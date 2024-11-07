use crate::config::CONFIG;
use crate::localization::get_string;
use crate::{
    config::{read_file, DATA_PATH},
    logger::Logger,
};
use once_cell::sync::Lazy;
use serde::Deserialize;
use serenity::{
    all::{GuildChannel, Timestamp, UserId},
    builder::{CreateButton, CreateEmbed, CreateMessage},
    model::{
        application::{ButtonStyle, ComponentInteraction},
        colour::Colour,
        guild::Member,
        id::ChannelId,
    },
    prelude::*,
};
use std::collections::HashMap;
use std::sync::Arc;
use walkdir::WalkDir;

pub static SHOPMANAGER: Lazy<Arc<RwLock<ShopManager>>> =
    Lazy::new(|| Arc::new(RwLock::new(ShopManager::new("shop".to_string()))));

#[derive(Debug, Deserialize)]
#[serde(tag = "type", rename_all = "lowercase")]
enum ReplacementOrPage {
    Replacement(ReplacementData),
    Page(Page),
}

#[derive(Debug, Deserialize)]
#[serde(tag = "type", rename_all = "lowercase")]
struct ReplacementData {
    name: String,
    value: Replacement,
}

#[derive(Debug, Deserialize)]
pub struct ShopManager {
    pages: Vec<Page>,
    replacements: HashMap<String, Replacement>,
    shop_path: String,
}

impl ShopManager {
    fn new(shop_path: String) -> Self {
        let mut shop = ShopManager {
            pages: Vec::new(),
            replacements: HashMap::new(),
            shop_path,
        };
        shop.collect_data();
        shop
    }

    fn collect_data(&mut self) {
        self.pages = Vec::new();
        self.replacements = HashMap::new();

        for entry in WalkDir::new(DATA_PATH.join(&self.shop_path)) {
            let entry = match entry {
                Ok(s) => s,
                Err(error) => {
                    println!("Error while reading shop prototypes: {}", error);
                    continue;
                }
            };

            if !entry.path().is_file() {
                continue;
            }

            let content: Vec<ReplacementOrPage> =
                serde_yaml::from_str(read_file(&entry.path().to_path_buf()).as_str()).expect(
                    format!(
                        "Error while parsing shop file: {}",
                        entry.file_name().to_str().unwrap()
                    )
                    .as_str(),
                );

            for cont in content.iter() {
                match cont {
                    ReplacementOrPage::Replacement(repl) => {
                        self.replacements.insert(
                            repl.name.clone(),
                            match repl.value {
                                Replacement::Str(ref string) => {
                                    Replacement::Str(get_string(string.as_str(), None))
                                }
                                _ => repl.value.clone(),
                            },
                        );
                    }
                    ReplacementOrPage::Page(page) => {
                        let mut page_clone = page.clone();
                        page_clone.convert();
                        self.pages.push(page_clone)
                    }
                }
            }
        }
    }

    fn convert_string(&self, string: String) -> Replacement {
        let mut out = Replacement::Str(get_string(string.as_str(), None));

        for (replacement, value) in self.replacements.iter() {
            if string.contains(replacement) {
                match value {
                    Replacement::Str(repl_string) => {
                        if let Replacement::Str(s) = out {
                            out = Replacement::Str(s.replace(replacement, &repl_string));
                        }
                    }
                    _ => {
                        return value.clone();
                    }
                }
            }
        }

        out
    }
}

#[derive(Debug, Deserialize, Clone)]
#[serde(untagged)]
enum Replacement {
    Str(String),
    Num(i64),
    Float(f64),
    Channel(GuildChannel),
    Member(Member),
}

pub struct Shop {
    current_page: i64,
    pages: Vec<Page>,
}

#[derive(Debug, Deserialize, Clone)]
pub struct Page {
    name: String,
    description: String,
    price: i64,
    #[serde(default)]
    access: Vec<String>,
    #[serde(rename = "notAcces", default)]
    not_acces: Vec<String>,
    #[serde(rename = "onBuy")]
    on_buy: Vec<ShopActions>,
}

impl Page {
    fn convert(&mut self) {
        self.name = get_string(self.name.as_str(), None);
        self.description = get_string(self.description.as_str(), None);
    }

    async fn buy(&self, ctx: Context, inter: ComponentInteraction) {
        for action in self.on_buy.iter() {
            match action {
                ShopActions::GiveRoles(give_roles) => {
                    match give_roles.call(ctx.clone(), inter.clone()).await {
                        Err(e) => {
                            Logger::error(
                                "shop.page.buy",
                                &format!(
                                    "error while call giveRoles shop action in page \"{}\": {}",
                                    &self.name, e
                                ),
                            )
                            .await
                        }
                        _ => (),
                    }
                }
                ShopActions::RemoveRoles(remove_roles) => {
                    match remove_roles.call(ctx.clone(), inter.clone()).await {
                        Err(e) => {
                            Logger::error(
                                "shop.page.buy",
                                &format!(
                                    "error while call removeRoles shop action in page \"{}\": {}",
                                    &self.name, e
                                ),
                            )
                            .await
                        }
                        _ => (),
                    }
                }
                ShopActions::SendMessage(send_message) => {
                    match send_message.call(ctx.clone(), inter.clone()).await {
                        Err(e) => {
                            Logger::error(
                                "shop.page.buy",
                                &format!(
                                    "error while call sendMessage shop action in page \"{}\": {}",
                                    &self.name, e
                                ),
                            )
                            .await
                        }
                        _ => (),
                    }
                }
                ShopActions::Mute(mute) => match mute.call(ctx.clone(), inter.clone()).await {
                    Err(e) => {
                        Logger::error(
                            "shop.page.buy",
                            &format!(
                                "error while call mute shop action in page \"{}\": {}",
                                &self.name, e
                            ),
                        )
                        .await
                    }
                    _ => (),
                },
            }
        }
    }

    pub fn to_message_bulder(&self, current_page: i32, max_pages: i32) -> CreateMessage {
        CreateMessage::new()
            .embed(
                CreateEmbed::new()
                    .title(get_string("shop-embed-title", None))
                    .description(get_string("shop-embed-description", None))
                    .color(match CONFIG.try_read().unwrap().shop_embed_color {
                        Some(color) => color,
                        None => Colour::ORANGE.0,
                    })
                    .field(
                        get_string(
                            "shop-embed-item",
                            Some(HashMap::from([(
                                "num",
                                format!("{}", current_page).as_str(),
                            )])),
                        ),
                        &self.name,
                        false,
                    )
                    .field(
                        get_string("shop-embed-description-field", None),
                        &self.description,
                        false,
                    )
                    .field(
                        get_string("shop-embed-price", None),
                        format!("```{}```", self.price),
                        true,
                    )
                    .field(
                        get_string("shop-embed-page", None),
                        format!("```{}/{}```", current_page, max_pages),
                        true,
                    )
                    .field(
                        get_string("shop-embed-balance", None),
                        format!("```{}```", "TODO member score"),
                        true,
                    ),
            )
            .button(
                CreateButton::new("previous")
                    .emoji('◀')
                    .style(ButtonStyle::Secondary),
            )
            .button(
                CreateButton::new("buy")
                    .emoji('🛒')
                    .style(ButtonStyle::Success),
            )
            .button(
                CreateButton::new("next")
                    .emoji('▶')
                    .style(ButtonStyle::Secondary),
            )
    }
}

#[derive(Debug, Deserialize, Clone)]
#[serde(tag = "type", rename_all = "camelCase")]
enum ShopActions {
    GiveRoles(GiveRoles),
    RemoveRoles(RemoveRoles),
    SendMessage(SendMessage),
    Mute(Mute),
}

trait Action {
    async fn call(&self, ctx: Context, inter: ComponentInteraction) -> Result<(), String>;
}

#[derive(Debug, Deserialize, Clone)]
struct GiveRoles {
    #[serde(default)]
    member: StringOrNum,
    roles: Vec<String>,
}

impl Action for GiveRoles {
    async fn call(&self, ctx: Context, inter: ComponentInteraction) -> Result<(), String> {
        if let Some(guild) = inter.guild_id {
            let guild = guild
                .to_guild_cached(&ctx.cache)
                .expect("cannot get cached guild from GuildId");

            match get_member(&inter, &ctx, &self.member).await {
                Ok(member) => {
                    for role_name in self.roles.iter() {
                        if let Some(role) = guild.role_by_name(role_name) {
                            if let Err(e) = member.add_role(&ctx.http, role.id).await {
                                return Err(format!(
                                    "cannot give role {} because - {}",
                                    role_name,
                                    e.to_string()
                                ));
                            }
                        }
                    }
                    Ok(())
                }
                Err(e) => Err(e),
            }
        } else {
            Err("cannot get guild from shop interaction, wtf".to_string())
        }
    }
}

#[derive(Debug, Deserialize, Clone)]
struct RemoveRoles {
    member: StringOrNum,
    roles: Vec<String>,
}

impl Action for RemoveRoles {
    async fn call(&self, ctx: Context, inter: ComponentInteraction) -> Result<(), String> {
        if let Some(guild) = inter.guild_id {
            let guild = guild
                .to_guild_cached(&ctx.cache)
                .expect("cannot get cached guild from GuildId");

            match get_member(&inter, &ctx, &self.member).await {
                Ok(member) => {
                    for role_name in self.roles.iter() {
                        if let Some(role) = guild.role_by_name(role_name) {
                            member.remove_role(&ctx.http, role.id).await.unwrap();
                        }
                    }
                    Ok(())
                }
                Err(e) => Err(e),
            }
        } else {
            Err("cannot get guild from shop interaction, wtf".to_string())
        }
    }
}

#[derive(Debug, Deserialize, Clone)]
struct SendMessage {
    #[serde(default)]
    channel: StringOrNum,
    message: String,
}

impl Action for SendMessage {
    async fn call(&self, ctx: Context, inter: ComponentInteraction) -> Result<(), String> {
        match get_channel(&inter, &ctx, &self.channel).await {
            Ok(channel) => {
                match channel
                    .send_message(&ctx.http, CreateMessage::new().content(&self.message))
                    .await
                {
                    Ok(_) => Ok(()),
                    Err(_) => Err("cannot send message in SendMessage action".to_string()),
                }
            }
            Err(e) => Err(e),
        }
    }
}

#[derive(Debug, Deserialize, Clone)]
struct Mute {
    #[serde(default)]
    member: StringOrNum,
    duration: i64,
}

impl Action for Mute {
    async fn call(&self, ctx: Context, inter: ComponentInteraction) -> Result<(), String> {
        match Timestamp::from_unix_timestamp(Timestamp::now().unix_timestamp() + self.duration) {
            Ok(time) => match get_member(&inter, &ctx, &self.member).await {
                Ok(mut member) => match member
                    .disable_communication_until_datetime(&ctx, time)
                    .await
                {
                    Ok(_) => Ok(()),
                    Err(e) => Err(format!(
                        "error while disabling member communication in mute action: {}",
                        e.to_string()
                    )
                    .to_string()),
                },
                Err(e) => Err(e),
            },
            Err(_) => Err("invalid duration given in mute action".to_string()),
        }
    }
}

#[derive(Debug, Deserialize, Clone)]
#[serde(untagged)]
enum StringOrNum {
    Str(String),
    Num(u64),
    Nothing,
}

impl Default for StringOrNum {
    fn default() -> Self {
        StringOrNum::Nothing
    }
}

async fn get_member(
    inter: &ComponentInteraction,
    ctx: &Context,
    content: &StringOrNum,
) -> Result<Member, String> {
    if let Some(guild) = inter.guild_id {
        let guild = guild
            .to_guild_cached(&ctx.cache)
            .expect("cannot get cached guild from GuildId");

        match content {
            StringOrNum::Str(string) => {
                let shop_man = SHOPMANAGER.read().await;
                match shop_man.convert_string(string.clone()) {
                    Replacement::Num(num) => {
                        match guild.member(&ctx.http, UserId::new(num as u64)).await {
                            Ok(member) => Ok(member.as_ref().clone()),
                            Err(e) => Err(e.to_string()),
                        }
                    }
                    Replacement::Member(member) => Ok(member),
                    _ => Err("member field in Mute action must be number or member".to_string()),
                }
            }
            StringOrNum::Num(num) => match guild.member(&ctx.http, UserId::new(*num)).await {
                Ok(member) => Ok(member.as_ref().clone()),
                Err(e) => Err(e.to_string()),
            },
            StringOrNum::Nothing => {
                panic!(
                    "member object can only be retrieved by number (id) or Member replacement tag"
                )
            }
        }
    } else {
        Err("cannot get guild from shop interaction, wtf".to_string())
    }
}

async fn get_channel(
    inter: &ComponentInteraction,
    ctx: &Context,
    content: &StringOrNum,
) -> Result<GuildChannel, String> {
    if let Some(guild) = inter.guild_id {
        let guild = guild
            .to_guild_cached(&ctx.cache)
            .expect("cannot get cached guild from GuildId");

        match content {
            StringOrNum::Num(num) => {
                match guild
                    .channels(&ctx.http)
                    .await
                    .unwrap()
                    .get(&ChannelId::new(*num))
                {
                    Some(channel) => Ok(channel.clone()),
                    None => Err(format!("channel with {} id not found", num)),
                }
            }
            StringOrNum::Str(string) => {
                let shop_man = SHOPMANAGER.read().await;
                match shop_man.convert_string(string.clone()) {
                    Replacement::Num(num) => {
                        match guild
                            .channels(&ctx.http)
                            .await
                            .unwrap()
                            .get(&ChannelId::new(num as u64))
                        {
                            Some(channel) => Ok(channel.clone()),
                            None => Err(format!(
                                "channel with {} id not found (id taken from replacement)",
                                num
                            )),
                        }
                    }
                    Replacement::Channel(channel) => Ok(channel),
                    _ => {
                        Err("uncompatible replacement type for channel in page pototype"
                            .to_string())
                    }
                }
            }
            StringOrNum::Nothing => match guild.channels(&ctx.http).await {
                Ok(channels) => match channels.get(&inter.channel_id) {
                    Some(channel) => Ok(channel.clone()),
                    None => Err("cannot found channel from interaction".to_string()),
                },
                Err(_) => Err("cannot get guild channels, wtf".to_string()),
            },
        }
    } else {
        Err("cannot get guild from shop interaction, wtf".to_string())
    }
}
