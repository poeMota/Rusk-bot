use crate::model::member::ProjectMember;
use crate::{
    config::{read_file, DATA_PATH},
    prelude::*,
    shop::{Action, ShopActions},
};
use once_cell::sync::Lazy;
use serde::Deserialize;
use serenity::{
    all::GuildChannel,
    builder::CreateEmbed,
    model::{
        application::ComponentInteraction,
        colour::Colour,
        guild::{Member, Role},
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

#[derive(Debug, Deserialize, Clone)]
pub struct ShopData {
    pub current_page: i32,
    pub pages: Vec<Page>,
}

impl Default for ShopData {
    fn default() -> Self {
        Self {
            current_page: 0,
            pages: Vec::new(),
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
                                Replacement::Str(ref string) => Replacement::Str(loc!(string)),
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

    pub fn convert_string(&self, string: String) -> Replacement {
        let mut out = Replacement::Str(loc!(&string));

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
pub enum Replacement {
    Str(String),
    Num(i64),
    Float(f64),
    Channel(GuildChannel),
    Member(Member),
    Role(Role),
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
        self.name = loc!(&self.name);
        self.description = loc!(&self.description);

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

        member.change_score(-self.price).await;
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
            .title(loc!("shop-embed-title"))
            .description(loc!("shop-embed-description"))
            .color(Colour::ORANGE)
            .field(
                loc!(
                    "shop-embed-item",
                    "num" = format!("{}", member.shop_data.current_page + 1)
                ),
                &self.name,
                false,
            )
            .field(
                loc!("shop-embed-description-field"),
                &self.description,
                false,
            )
            .field(
                loc!("shop-embed-price"),
                format!("```{}```", self.price),
                true,
            )
            .field(
                loc!("shop-embed-page"),
                format!("```{}/{}```", member.shop_data.current_page + 1, max_pages),
                true,
            )
            .field(
                loc!("shop-embed-balance"),
                format!("```{}```", member.score),
                true,
            )
    }
}
