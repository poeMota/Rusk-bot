use crate::{
    config::{read_file, write_file, DATA_PATH},
    prelude::*,
    shop::ShopData,
};
use once_cell::sync::Lazy;
use serde::{de::value, Deserialize, Serialize};
use serde_json;
use serenity::{
    builder::CreateEmbed,
    client::Context,
    model::{colour::Colour, guild::Member, id::UserId, timestamp::Timestamp},
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
                        &format!("error with member data file: {}", error),
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
                    Err(e) => {
                        Logger::error(
                            "mem_man.init",
                            &format!(
                                "error while parsing member data file \"{}\": {}",
                                entry.file_name().to_str().unwrap(),
                                e.to_string()
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

#[derive(Deserialize, Serialize, Clone, Debug, PartialEq)]
pub enum TaskHistory {
    Current(HashMap<Timestamp, u32>),
    OldFormat(String),
}

#[derive(Deserialize, Serialize, Clone, Debug)]
pub struct ProjectMember {
    pub id: UserId,
    #[serde(default)]
    pub in_tasks: Vec<u32>,
    #[serde(default)]
    pub done_tasks: HashMap<String, Vec<TaskHistory>>,
    #[serde(default)]
    pub mentor_tasks: HashMap<String, Vec<TaskHistory>>,
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

        Ok(match content.as_str() {
            "" => Self {
                id: id.clone(),
                in_tasks: Vec::new(),
                done_tasks: HashMap::new(),
                mentor_tasks: HashMap::new(),
                own_folder: None,
                score: 0,
                all_time_score: 0,
                warns: Vec::new(),
                notes: Vec::new(),
                shop_data: ShopData::default(),
            },
            _ => serde_json::from_str(&content)?,
        })
    }

    pub async fn member(&self) -> Result<Member, serenity::Error> {
        Ok(fetch_member(self.id.get()).await?)
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

    pub fn leave_task(&mut self, id: u32) {
        if self.in_tasks.contains(&id) {
            self.in_tasks
                .remove(match self.in_tasks.iter().position(|x| x == &id) {
                    Some(index) => index,
                    None => {
                        return ();
                    }
                });
            self.update();
        }
    }

    pub fn join_task(&mut self, id: u32) {
        if !self.in_tasks.contains(&id) {
            self.in_tasks.push(id);
            self.update();
        }
    }

    pub fn add_done_task(&mut self, project_name: &String, task: u32) {
        if !self.done_tasks.contains_key(project_name) {
            self.done_tasks.insert(project_name.clone(), Vec::new());
        }

        self.done_tasks
            .get_mut(project_name)
            .unwrap()
            .push(TaskHistory::Current(HashMap::from([(
                Timestamp::now(),
                task,
            )])));
        self.update();
    }

    pub fn add_mentor_task(&mut self, project_name: &String, task: u32) {
        if !self.mentor_tasks.contains_key(project_name) {
            self.mentor_tasks.insert(project_name.clone(), Vec::new());
        }

        self.mentor_tasks
            .get_mut(project_name)
            .unwrap()
            .push(TaskHistory::Current(HashMap::from([(
                Timestamp::now(),
                task,
            )])));
        self.update();
    }

    pub async fn to_embed(&self, ctx: &Context) -> CreateEmbed {
        let dis_member = self.member().await.unwrap();

        let mut embed = CreateEmbed::new()
            .title(get_string(
                "member-stat-embed-title",
                Some(HashMap::from([("member", dis_member.display_name())])),
            ))
            .color(match get_guild().to_guild_cached(&ctx.cache) {
                Some(guild) => match guild.member_highest_role(&dis_member) {
                    Some(color) => color.colour,
                    None => Colour::LIGHT_GREY,
                },
                None => Colour::LIGHT_GREY,
            });

        let task_man = TASKMANAGER.try_read().unwrap();
        if !self.in_tasks.is_empty() {
            embed = embed.field(
                get_string(
                    "member-stat-embed-in-tasks-name",
                    Some(HashMap::from([(
                        "num",
                        self.in_tasks.len().to_string().as_str(),
                    )])),
                ),
                {
                    let mut value = String::new();
                    for id in self.in_tasks.iter() {
                        if let Some(task) = task_man.get(*id) {
                            value = format!(
                                "{}{} <#{}>\n",
                                value,
                                match id == self.in_tasks.last().unwrap() {
                                    true => "╠︎",
                                    false => "╚",
                                },
                                task.thread_id.get()
                            );
                        }
                    }
                    value
                },
                false,
            );
        }

        if !self.done_tasks.is_empty() {
            embed = embed.field(
                get_string(
                    "member-stat-embed-done-tasks-name",
                    Some(HashMap::from([(
                        "num",
                        self.done_tasks.len().to_string().as_str(),
                    )])),
                ),
                {
                    let mut value = String::new();
                    for (proj, tasks) in self.done_tasks.iter() {
                        if !tasks.is_empty() {
                            value = format!("{}{} ({})\n", value, proj, tasks.len());

                            for task in tasks.iter() {
                                value = format!(
                                    "{}{} {}\n",
                                    value,
                                    match task == tasks.last().unwrap() {
                                        true => "╠︎",
                                        false => "╚",
                                    },
                                    match task {
                                        TaskHistory::Current(map) => {
                                            let mut value_2 = String::new();
                                            for (time, id) in map.iter() {
                                                value_2 = format!(
                                                    "{}<t:{}:D> <#{}>\n",
                                                    value_2,
                                                    time.timestamp(),
                                                    task_man.get(*id).unwrap().thread_id.get()
                                                );
                                            }
                                            value_2
                                        }
                                        TaskHistory::OldFormat(string) => string.clone(),
                                    }
                                )
                            }
                        }
                    }
                    value
                },
                false,
            );
        }

        if !self.mentor_tasks.is_empty() {
            embed = embed.field(
                get_string(
                    "member-stat-embed-mentor-tasks-name",
                    Some(HashMap::from([(
                        "num",
                        self.mentor_tasks.len().to_string().as_str(),
                    )])),
                ),
                {
                    let mut value = String::new();
                    for (proj, tasks) in self.mentor_tasks.iter() {
                        if !tasks.is_empty() {
                            value = format!("{}{} ({})\n", value, proj, tasks.len());

                            for task in tasks.iter() {
                                value = format!(
                                    "{}{} {}\n",
                                    value,
                                    match task == tasks.last().unwrap() {
                                        true => "╠︎",
                                        false => "╚",
                                    },
                                    match task {
                                        TaskHistory::Current(map) => {
                                            let mut value_2 = String::new();
                                            for (time, id) in map.iter() {
                                                value_2 = format!(
                                                    "{}<t:{}:D> <#{}>\n",
                                                    value_2,
                                                    time.timestamp(),
                                                    task_man.get(*id).unwrap().thread_id.get()
                                                );
                                            }
                                            value_2
                                        }
                                        TaskHistory::OldFormat(string) => string.clone(),
                                    }
                                )
                            }
                        }
                    }
                    value
                },
                false,
            );
        }

        if let Some(ref folder) = self.own_folder {
            embed = embed.field(
                get_string("member-stat-embed-folder-name", None),
                folder,
                false,
            );
        }

        embed
            .field(
                get_string("member-stat-embed-score-name", None),
                self.score.to_string(),
                false,
            )
            .field(
                get_string("member-stat-embed-all-time-score-name", None),
                self.all_time_score.to_string(),
                false,
            )
    }
}
