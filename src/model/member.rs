use crate::{
    config::{read_file, write_file, DATA_PATH},
    events::*,
    prelude::*,
    shop::ShopData,
};
use event_macro::Event;
use once_cell::sync::Lazy;
use serde::{Deserialize, Serialize};
use serde_json;
use serenity::{
    builder::CreateEmbed,
    client::Context,
    model::{colour::Colour, guild::Member, id::UserId},
};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use walkdir::WalkDir;

pub static MEMBERSMANAGER: Lazy<Arc<RwLock<MembersManager>>> =
    Lazy::new(|| Arc::new(RwLock::new(MembersManager::new())));

#[derive(Deserialize, Debug)]
pub struct MembersManager {
    members: HashMap<UserId, ProjectMember>,
}

impl MembersManager {
    fn new() -> Self {
        Self {
            members: HashMap::new(),
        }
    }

    pub async fn init(&mut self) {
        for entry in WalkDir::new(DATA_PATH.join("databases/members")) {
            let entry = match entry {
                Ok(s) => s,
                Err(error) => {
                    Logger::error(
                        "mem_man.init",
                        &format!("Error with member data file: {}", error),
                    )
                    .await;
                    continue;
                }
            };

            if !entry.path().is_file() {
                continue;
            }

            let member: ProjectMember =
                match serde_yaml::from_str(read_file(&entry.path().to_path_buf()).as_str()) {
                    Ok(c) => c,
                    Err(_) => {
                        Logger::error(
                            "mem_man.init",
                            &format!(
                                "Error while parsing member data file: {}",
                                entry.file_name().to_str().unwrap()
                            ),
                        )
                        .await;
                        continue;
                    }
                };

            self.members.insert(member.id.clone(), member);
        }

        Logger::debug("mem_man.init", "initialized from databases/members/*").await;
    }

    fn serialize(&self) {
        write_file(
            &DATA_PATH.join("databases/members.json"),
            serde_json::to_string(&self.members).unwrap(),
        );
    }

    pub fn update(&self) {
        self.serialize();
    }

    // No need update after add empty member
    pub async fn get(&mut self, id: UserId) -> Result<&ProjectMember, serenity::Error> {
        Ok(self.members.entry(id.clone()).or_insert_with({
            let member = ProjectMember::new(id).await?;
            || member
        }))
    }

    pub async fn get_mut(&mut self, id: UserId) -> Result<&mut ProjectMember, serenity::Error> {
        Ok(self.members.entry(id.clone()).or_insert_with({
            let member = ProjectMember::new(id).await?;
            || member
        }))
    }
}

#[derive(Deserialize, Serialize, Clone, Debug)]
pub struct ProjectMember {
    pub id: UserId,
    #[serde(default, skip_serializing)]
    pub dis_member: Member,
    #[serde(default)]
    pub in_tasks: HashMap<String, u32>,
    #[serde(default)]
    pub done_tasks: HashMap<String, String>,
    #[serde(default)]
    pub curation_tasks: HashMap<String, String>,
    pub own_folder: Option<String>,
    #[serde(default)]
    pub score: i64,
    #[serde(default)]
    pub all_time_score: i64,
    #[serde(default)]
    pub warns: Vec<String>,
    #[serde(default)]
    pub notes: Vec<String>,
    #[serde(default, skip_serializing)]
    pub shop_data: ShopData,
}

impl ProjectMember {
    async fn new(id: UserId) -> Result<Self, serenity::Error> {
        let content = read_file(&DATA_PATH.join(format!("databases/members/{}", id.get())));

        let mut instance: ProjectMember = match content.as_str() {
            "" => Self {
                id: id.clone(),
                dis_member: fetch_member(id.get()).await?,
                in_tasks: HashMap::new(),
                done_tasks: HashMap::new(),
                curation_tasks: HashMap::new(),
                own_folder: None,
                score: 0,
                all_time_score: 0,
                warns: Vec::new(),
                notes: Vec::new(),
                shop_data: ShopData::default(),
            },
            _ => serde_json::from_str(&content).unwrap(),
        };

        instance.fetch_member().await?;
        Ok(instance)
    }

    async fn fetch_member(&mut self) -> Result<(), serenity::Error> {
        self.dis_member = fetch_member(self.id.clone().get()).await?;
        Ok(())
    }

    fn serialize(&self) {
        write_file(
            &DATA_PATH.join(format!("databases/members/{}", self.id.get())),
            serde_json::to_string(&self).unwrap(),
        );
    }

    fn update(&self) {
        self.serialize();
    }

    pub fn change_score(&mut self, score: i64) {
        self.score += score;
        self.update();
    }

    pub fn change_folder(&mut self, folder: Option<String>) {
        self.own_folder = folder;
        self.update();
    }

    pub fn to_embed(&self, ctx: &Context) -> CreateEmbed {
        let mut embed = CreateEmbed::new()
            .title(get_string(
                "member-stat-embed-title",
                Some(HashMap::from([("member", self.dis_member.display_name())])),
            ))
            .color(match get_guild().to_guild_cached(&ctx.cache) {
                Some(guild) => match guild.member_highest_role(&self.dis_member) {
                    Some(color) => color.colour,
                    None => Colour::LIGHT_GREY,
                },
                None => Colour::LIGHT_GREY,
            });
        // TODO
        embed
    }
}
