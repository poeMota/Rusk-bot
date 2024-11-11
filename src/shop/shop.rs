use crate::model::ProjectMember;
use crate::{
    config::{read_file, DATA_PATH},
    prelude::*,
};
use once_cell::sync::Lazy;
use serde::Deserialize;
use serenity::{
    all::{GuildChannel, Timestamp, UserId},
    builder::{CreateEmbed, CreateMessage},
    model::{application::ComponentInteraction, colour::Colour, guild::Member, id::ChannelId},
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

#[derive(Debug, Deserialize, Clone)]
pub struct ShopData {
    pub current_page: i32,
    pub pages: Vec<Page>,
    pub inter: Option<CommandInteraction>,
}

impl Default for ShopData {
    fn default() -> Self {
        Self {
            current_page: 0,
            pages: Vec::new(),
            inter: None,
        }
    }
}

#[derive(Debug, Deserialize)]
pub struct ShopManager {
    pages: Vec<Page>,
    replacements: HashMap<String, Replacement>,
    shop_path: String,
}

impl ShopManager {
    fn new(shop_path: String) -> Self {
        ShopManager {
            pages: Vec::new(),
            replacements: HashMap::new(),
            shop_path,
        }
    }

    pub async fn init(&mut self) {
        let mut pages = Vec::new();
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
                    ReplacementOrPage::Page(page) => pages.push(page.clone()),
                }
            }
        }

        for page in pages.iter_mut() {
            page.convert(&self).await.expect(&format!(
                "error with page \"{}\", full struct:\n{:#?}",
                page.name, page
            ));
        }

        self.pages = pages;

        Logger::debug(
            "shop_man.init",
            &format!("initialized from shop/* with {} pages", self.pages.len()),
        )
        .await;
    }

    fn convert_string(&self, string: String) -> Replacement {
        let mut out = Replacement::Str(get_string(string.as_str(), None));

        for (replacement, value) in self.replacements.iter() {
            if string.contains(&format!("<{}>", replacement)) {
                match value {
                    Replacement::Str(repl_string) => {
                        if let Replacement::Str(s) = out {
                            out = Replacement::Str(
                                s.replace(&format!("<{}>", replacement), &repl_string),
                            );
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

    pub async fn get_pages(&self, ctx: &Context, dis_member: &Member) -> Vec<Page> {
        let guild = match get_guild().to_guild_cached(&ctx) {
            Some(g) => g,
            None => {
                return Vec::new();
            }
        };

        let mut out_pages: Vec<Page> = Vec::new();

        'page_loop: for page in self.pages.iter() {
            for name in page.not_access.iter() {
                if let Some(role) = guild.role_by_name(&name) {
                    if dis_member.roles.contains(&role.id) {
                        continue 'page_loop;
                    }
                } else {
                    println!(
                        "cannot find role with name {} in \"{}\".notAccess",
                        name, page.name
                    )
                    /*
                    Logger::error(
                        "shop_man.get_pages",
                        &format!(
                            "cannot find role with name {} in \"{}\".notAccess",
                            name, page.name
                        ),
                    )
                    .await;
                    */
                }
            }

            if page.access.len() == 0 {
                out_pages.push(page.clone());
                continue 'page_loop;
            }

            for name in page.access.iter() {
                if let Some(role) = guild.role_by_name(&name) {
                    if dis_member.roles.contains(&role.id) {
                        out_pages.push(page.clone());
                        continue 'page_loop;
                    }
                } else {
                    println!(
                        "cannot find role with name {} in \"{}\".access",
                        name, page.name
                    )
                    /*
                    Logger::error(
                        "shop_man.get_pages",
                        &format!(
                            "cannot find role with name {} in \"{}\".access",
                            name, page.name
                        ),
                    )
                    .await;
                    */
                }
            }
        }
        out_pages
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
    Nothing,
}

impl Default for Replacement {
    fn default() -> Self {
        Replacement::Nothing
    }
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
    #[serde(rename = "notAccess", default)]
    not_access: Vec<String>,
    #[serde(rename = "onBuy")]
    on_buy: Vec<ShopActions>,
}

impl Page {
    async fn convert(&mut self, shop_man: &ShopManager) -> Result<(), String> {
        self.name = get_string(self.name.as_str(), None);
        self.description = get_string(self.description.as_str(), None);

        for action in self.on_buy.iter_mut() {
            match action {
                ShopActions::GiveRoles(act) => act.convert(&shop_man).await?,
                ShopActions::RemoveRoles(act) => act.convert(&shop_man).await?,
                ShopActions::SendMessage(act) => act.convert(&shop_man).await?,
                ShopActions::Mute(act) => act.convert(&shop_man).await?,
            }
        }

        Ok(())
    }

    pub async fn buy(&self, inter: &ComponentInteraction, member: &mut ProjectMember) {
        if member.score < self.price {
            return;
        }

        let dis_member = member.member().await.unwrap();

        for action in self.on_buy.iter() {
            match action {
                ShopActions::GiveRoles(give_roles) => match give_roles.call(inter.clone()).await {
                    Err(e) => {
                        Logger::error(
                            "shop.page.buy",
                            &format!(
                                "error while call giveRoles shop action in page \"{}\": {}",
                                &self.name, e
                            ),
                        )
                        .await;
                    }
                    _ => (),
                },
                ShopActions::RemoveRoles(remove_roles) => {
                    match remove_roles.call(inter.clone()).await {
                        Err(e) => {
                            Logger::error(
                                "shop.page.buy",
                                &format!(
                                    "error while call removeRoles shop action in page \"{}\": {}",
                                    &self.name, e
                                ),
                            )
                            .await;
                        }
                        _ => (),
                    }
                }
                ShopActions::SendMessage(send_message) => {
                    match send_message.call(inter.clone()).await {
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
                ShopActions::Mute(mute) => match mute.call(inter.clone()).await {
                    Err(e) => {
                        Logger::error(
                            "shop.page.buy",
                            &format!(
                                "error while call mute shop action in page \"{}\": {}",
                                &self.name, e
                            ),
                        )
                        .await;
                    }
                    _ => (),
                },
            }
        }

        member.change_score(-self.price);
        Logger::low(
            "shop.page.buy",
            &format!(
                "user {} score has been changed to {} and is now {}",
                dis_member.display_name(),
                -self.price,
                member.score
            ),
        )
        .await;

        Logger::medium(
            "shop.page.buy",
            &format!(
                "user {} has made a purchase \"{}\"",
                dis_member.display_name(),
                self.name
            ),
        )
        .await;
    }

    pub fn to_embed(&self, member: &ProjectMember, max_pages: i32) -> CreateEmbed {
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
                        format!("{}", member.shop_data.current_page + 1).as_str(),
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
                format!("```{}/{}```", member.shop_data.current_page + 1, max_pages),
                true,
            )
            .field(
                get_string("shop-embed-balance", None),
                format!("```{}```", member.score),
                true,
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
    async fn call(&self, inter: ComponentInteraction) -> Result<(), String>;

    async fn convert(&mut self, shop_man: &ShopManager) -> Result<(), String>;
}

#[derive(Debug, Deserialize, Clone)]
struct GiveRoles {
    #[serde(default)]
    member: Replacement,
    roles: Vec<String>,
}

impl Action for GiveRoles {
    async fn call(&self, inter: ComponentInteraction) -> Result<(), String> {
        let guild = match get_guild().to_partial_guild(get_http()).await {
            Ok(g) => g,
            Err(_) => return Err("failed to fetch guild from API".to_string()),
        };

        let member = match self.member.clone() {
            Replacement::Member(member) => member,
            Replacement::Nothing => inter.member.ok_or_else(|| "")?,
            _ => {
                return Err("kys".to_string());
            }
        };

        for role_name in self.roles.iter() {
            if let Some(role) = guild.role_by_name(role_name) {
                if let Err(e) = member.add_role(get_http(), role.id).await {
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

    async fn convert(&mut self, shop_man: &ShopManager) -> Result<(), String> {
        if let Replacement::Str(ref string) = self.member {
            self.member = shop_man.convert_string(string.clone());
        }

        if let Replacement::Nothing = self.member {
        } else {
            self.member = Replacement::Member(get_member(&self.member).await?);
        }

        // TODO: roles convert
        Ok(())
    }
}

#[derive(Debug, Deserialize, Clone)]
struct RemoveRoles {
    #[serde(default)]
    member: Replacement,
    roles: Vec<String>,
}

impl Action for RemoveRoles {
    async fn call(&self, inter: ComponentInteraction) -> Result<(), String> {
        let guild = match get_guild().to_partial_guild(get_http()).await {
            Ok(g) => g,
            Err(_) => return Err("failed to fetch guild from API".to_string()),
        };

        let member = match self.member.clone() {
            Replacement::Member(member) => member,
            Replacement::Nothing => inter.member.ok_or_else(|| "")?,
            _ => {
                return Err("kys".to_string());
            }
        };

        for role_name in self.roles.iter() {
            if let Some(role) = guild.role_by_name(role_name) {
                if let Err(e) = member.remove_role(get_http(), role.id).await {
                    return Err(format!(
                        "cannot remove role {} because - {}",
                        role_name,
                        e.to_string()
                    ));
                }
            }
        }
        Ok(())
    }

    async fn convert(&mut self, shop_man: &ShopManager) -> Result<(), String> {
        if let Replacement::Str(ref string) = self.member {
            self.member = shop_man.convert_string(string.clone());
        }

        if let Replacement::Nothing = self.member {
        } else {
            self.member = Replacement::Member(get_member(&self.member).await?);
        }

        // TODO: roles convert
        Ok(())
    }
}

#[derive(Debug, Deserialize, Clone)]
struct SendMessage {
    #[serde(default)]
    channel: Replacement,
    message: String,
}

impl Action for SendMessage {
    async fn call(&self, inter: ComponentInteraction) -> Result<(), String> {
        let replacements = HashMap::from([("AuthorPing", format!("<@{}>", inter.user.id))]);
        let mut message = self.message.clone();

        for (replacement, value) in replacements.iter() {
            if message.contains(&format!("<{}>", replacement)) {
                message = message.replace(&format!("<{}>", replacement), &value);
            }
        }

        let channel = match self.channel.clone() {
            Replacement::Channel(channel) => channel,
            Replacement::Nothing => match get_guild().channels(get_http()).await {
                Ok(channels) => match channels.get(&inter.channel_id) {
                    Some(channel) => channel.clone(),
                    None => {
                        return Err("cannot found channel from interaction".to_string());
                    }
                },
                Err(_) => {
                    return Err("cannot get guild channels, wtf".to_string());
                }
            },
            _ => {
                return Err("kys".to_string());
            }
        };

        match channel
            .send_message(get_http(), CreateMessage::new().content(message))
            .await
        {
            Ok(_) => Ok(()),
            Err(e) => Err(format!(
                "cannot send message in SendMessage action: {}",
                e.to_string()
            )),
        }
    }

    async fn convert(&mut self, shop_man: &ShopManager) -> Result<(), String> {
        self.message = match shop_man.convert_string(self.message.clone()) {
            Replacement::Str(string) => string,
            _ => {
                return Err(
                    "message field in sendMessage must be string or string replacement".to_string(),
                )
            }
        };
        if let Replacement::Str(ref string) = self.channel {
            self.channel = shop_man.convert_string(string.clone());
        }

        if let Replacement::Nothing = self.channel {
        } else {
            self.channel = Replacement::Channel(get_channel(&self.channel).await?);
        }

        Ok(())
    }
}

#[derive(Debug, Deserialize, Clone)]
struct Mute {
    #[serde(default)]
    member: Replacement,
    duration: i64,
}

impl Action for Mute {
    async fn call(&self, inter: ComponentInteraction) -> Result<(), String> {
        match Timestamp::from_unix_timestamp(Timestamp::now().unix_timestamp() + self.duration) {
            Ok(time) => {
                let mut member = match self.member.clone() {
                    Replacement::Member(mem) => mem,
                    Replacement::Nothing => {
                        inter.member.ok_or_else(|| "member field if not specified if mute action and cannot take member from interaction".to_string())?
                    },
                    _ => {
                        return Err("kys".to_string());
                    }
                };

                match member
                    .disable_communication_until_datetime(get_http(), time)
                    .await
                {
                    Ok(_) => Ok(()),
                    Err(e) => Err(format!(
                        "error while disabling member communication in mute action: {}",
                        e.to_string()
                    )
                    .to_string()),
                }
            }
            Err(_) => Err("invalid duration given in mute action".to_string()),
        }
    }

    async fn convert(&mut self, shop_man: &ShopManager) -> Result<(), String> {
        if let Replacement::Str(ref string) = self.member {
            self.member = shop_man.convert_string(string.clone());
        }

        if let Replacement::Nothing = self.member {
        } else {
            self.member = Replacement::Member(get_member(&self.member).await?);
        }

        Ok(())
    }
}

async fn get_member(content: &Replacement) -> Result<Member, String> {
    let http = get_http();
    let guild = match get_guild().to_partial_guild(&http).await {
        Ok(g) => g,
        Err(_) => return Err("Failed to fetch guild from API".to_string()),
    };

    match content {
        Replacement::Num(num) => match guild.member(&http, UserId::new(*num as u64)).await {
            Ok(member) => Ok(member.clone()),
            Err(e) => Err(e.to_string()),
        },
        Replacement::Member(member) => Ok(member.clone()),
        _ => Err("uncompatible type to convert into member".to_string()),
    }
}

async fn get_channel(content: &Replacement) -> Result<GuildChannel, String> {
    let http = get_http();
    let guild = match get_guild().to_partial_guild(&http).await {
        Ok(g) => g,
        Err(_) => return Err("Failed to fetch guild from API".to_string()),
    };

    match content {
        Replacement::Num(num) => {
            match guild
                .channels(&http)
                .await
                .map_err(|e| e.to_string())?
                .get(&ChannelId::new(*num as u64))
            {
                Some(channel) => Ok(channel.clone()),
                None => Err(format!("channel with {} id not found", num)),
            }
        }
        Replacement::Channel(channel) => Ok(channel.clone()),
        _ => Err("uncompatible type to convert into channel".to_string()),
    }
}
