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

pub static MEMBERSMANAGER: Lazy<Arc<RwLock<MembersManager>>> =
    Lazy::new(|| Arc::new(RwLock::new(MembersManager::new())));

#[derive(Deserialize, Debug)]
pub struct MembersManager {
    members: HashMap<UserId, ProjectMember>,
}

impl MembersManager {
    fn new() -> Self {
        /*subscribe_event::<OnMemberUpdateEvent>({
            Box::new(move |ev: &OnMemberUpdateEvent| MEMBERSMANAGER.try_read().unwrap().update(ev))
        });*/

        Self {
            members: HashMap::new(),
        }
    }

    pub async fn init(&mut self, database_name: &str) {
        let content = read_file(&DATA_PATH.join(format!("databases/{}", database_name)));
        let members: Vec<ProjectMember> = match content.as_str() {
            "" => Vec::new(),
            _ => serde_json::from_str(&content).unwrap(),
        };

        for member in members.iter() {
            let mut mem = member.clone();

            if let Err(e) = mem.fetch_member().await {
                Logger::error(
                    "mem_man.init",
                    &format!(
                        "error while fetching member with id {} - {}",
                        mem.id.get(),
                        e.to_string()
                    ),
                )
                .await;
                continue;
            }

            self.members.insert(mem.dis_member.user.id, mem);
        }

        Logger::debug(
            "mem_man.init",
            &format!("initialized from database \"databases/{}\"", database_name),
        )
        .await;
    }

    fn serialize(&self) {
        write_file(
            &DATA_PATH.join("databases/members.json"),
            serde_json::to_string(&self.members).unwrap(),
        );
    }

    #[allow(unused_variables)]
    pub fn update(&self, ev: &OnMemberUpdateEvent) {
        self.serialize();
    }

    // No need update after add empty member
    pub fn get(&mut self, member: Member) -> &ProjectMember {
        self.members
            .entry(member.user.id.clone())
            .or_insert_with(|| ProjectMember::new(member))
    }

    pub fn get_mut(&mut self, member: Member) -> &mut ProjectMember {
        self.members
            .entry(member.user.id.clone())
            .or_insert_with(|| ProjectMember::new(member))
    }
}

#[derive(Event)]
pub struct OnMemberUpdateEvent {
    pub member: ProjectMember,
}

#[derive(Deserialize, Serialize, Clone, Debug)]
pub struct ProjectMember {
    pub id: UserId,
    #[serde(default, skip_serializing)]
    pub dis_member: Member,
    #[serde(default)]
    pub in_tasks: Vec<u32>,
    #[serde(default)]
    pub done_tasks: Vec<String>,
    #[serde(default)]
    pub curation_tasks: Vec<String>,
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
    fn new(member: Member) -> Self {
        Self {
            id: member.user.id.clone(),
            dis_member: member,
            in_tasks: Vec::new(),
            done_tasks: Vec::new(),
            curation_tasks: Vec::new(),
            own_folder: None,
            score: 0,
            all_time_score: 0,
            warns: Vec::new(),
            notes: Vec::new(),
            shop_data: ShopData::default(),
        }
    }

    async fn fetch_member(&mut self) -> Result<(), serenity::Error> {
        self.dis_member = fetch_member(self.id.clone().get()).await?;
        Ok(())
    }

    fn update(&self) {
        OnMemberUpdateEvent {
            member: self.clone(),
        }
        .raise();
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
