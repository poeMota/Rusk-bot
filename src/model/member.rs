use crate::{
    bot::fetch_member,
    config::{read_file, write_file, DATA_PATH},
    events::*,
    logger::Logger,
};
use event_macro::Event;
use once_cell::sync::Lazy;
use serde::{Deserialize, Serialize};
use serde_json;
use serenity::model::{guild::Member, id::UserId};
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
        Self {
            members: HashMap::new(),
        }
    }

    pub async fn init(&mut self, database_name: &str) {
        let content = read_file(&DATA_PATH.join(format!("databases/{}", database_name)));
        let members: Vec<ProjectMember> = serde_json::from_str(&content).unwrap();

        self.members = HashMap::new();
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

        subscribe_event::<OnMemberUpdateEvent>(Box::new(move |ev: &OnMemberUpdateEvent| {
            MEMBERSMANAGER.try_read().unwrap().update(ev)
        }));
    }

    fn serialize(&self) {
        write_file(
            &DATA_PATH.join("databases/members.json"),
            serde_json::to_string(&self.members).unwrap(),
        );
    }

    #[allow(unused_variables)]
    fn update(&self, ev: &OnMemberUpdateEvent) {
        self.serialize();
    }
}

#[derive(Event)]
pub struct OnMemberUpdateEvent {
    pub member: Member,
}

#[derive(Deserialize, Serialize, Clone, Debug)]
struct ProjectMember {
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
    pub curent_shop_page: i32,
}

impl ProjectMember {
    async fn fetch_member(&mut self) -> Result<(), serenity::Error> {
        self.dis_member = fetch_member(self.id.clone().get()).await?;
        Ok(())
    }
}
