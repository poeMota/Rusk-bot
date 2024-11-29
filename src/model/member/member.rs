use crate::{
    config::{read_file, write_file, DATA_PATH},
    model::task::{Task, TASKMANAGER},
    prelude::*,
    shop::ShopData,
};
use once_cell::sync::Lazy;
use serde::{Deserialize, Serialize};
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

            let content = read_file(&entry.path().to_path_buf());

            if content == String::new() {
                continue;
            }

            let member: ProjectMember = match serde_json::from_str(content.as_str()) {
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

impl TaskHistory {
    pub async fn get(&self) -> String {
        let task_man = match TASKMANAGER.try_read() {
            Ok(man) => man,
            Err(_) => {
                Logger::error(
                    "taskhistory.get",
                    "error while try_read TASKMANAGER, maybe deadlock, trying await...",
                )
                .await;
                TASKMANAGER.read().await
            }
        };

        match self {
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
    }
}

#[derive(Deserialize, Serialize, Clone, Debug, PartialEq)]
pub enum NotesHistory {
    Current((UserId, Timestamp, String)),
    OldFormat(String),
}

#[derive(Deserialize, Serialize, Clone, Debug)]
pub struct ProjectMember {
    pub id: UserId,
    #[serde(default)]
    pub in_tasks: HashMap<String, Vec<u32>>,
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
    pub last_activity: HashMap<String, Timestamp>,
    #[serde(default)]
    pub warns: Vec<NotesHistory>,
    #[serde(default)]
    pub notes: Vec<NotesHistory>,
    #[serde(default, skip_serializing)]
    pub shop_data: ShopData,
    #[serde(default, skip_serializing)]
    pub changed_member: Option<UserId>,
    #[serde(default, skip_serializing)]
    pub changed_task: Option<u64>,
}

impl ProjectMember {
    async fn new(id: UserId) -> Result<Self, serenity::Error> {
        let content = read_file(&DATA_PATH.join(format!("databases/members/{}", id.get())));

        Ok(match content.as_str() {
            "" => Self {
                id: id.clone(),
                in_tasks: HashMap::new(),
                done_tasks: HashMap::new(),
                mentor_tasks: HashMap::new(),
                own_folder: None,
                score: 0,
                all_time_score: 0,
                last_activity: HashMap::new(),
                warns: Vec::new(),
                notes: Vec::new(),
                shop_data: ShopData::default(),
                changed_member: None,
                changed_task: None,
            },
            _ => serde_json::from_str(&content)?,
        })
    }

    pub async fn member(&self) -> Result<Member, serenity::Error> {
        Ok(fetch_member(&self.id).await?)
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

    pub async fn change_score(&mut self, score: i64) {
        self.score += score;
        if score > 0 {
            self.all_time_score += score;
        }
        self.update();

        let dis_member = self.member().await.unwrap();
        Logger::debug(
            "member.change_score",
            &format!(
                "score of member {} changed by {}",
                dis_member.display_name(),
                score.to_string()
            ),
        )
        .await;
    }

    pub async fn change_folder(&mut self, folder: Option<String>) {
        let old_folder = self.own_folder.clone();

        self.own_folder = folder;
        self.update();

        let dis_member = self.member().await.unwrap();
        Logger::debug(
            &format!("member.{}", dis_member.display_name()),
            &format!(
                "own folder changed from {:?} to {:?}",
                old_folder, self.own_folder
            ),
        )
        .await;
    }

    pub async fn leave_task(&mut self, task: &Task) {
        if let Some(tasks) = self.in_tasks.get_mut(&task.project) {
            if tasks.contains(&task.id) {
                tasks.remove(match tasks.iter().position(|x| x == &task.id) {
                    Some(index) => index,
                    None => {
                        return ();
                    }
                });

                if tasks.is_empty() {
                    self.in_tasks.remove(&task.project);
                }

                self.update();

                let dis_member = self.member().await.unwrap();
                Logger::debug(
                    "members.leave_task",
                    &format!(
                        "{} ({}) leaved form task \"{}\"",
                        dis_member.display_name(),
                        self.id.get(),
                        task.id
                    ),
                )
                .await;
            }
        }
    }

    pub async fn join_task(&mut self, task: &Task) {
        if let Some(tasks) = self.in_tasks.get_mut(&task.project) {
            if !tasks.contains(&task.id) {
                tasks.push(task.id);
                self.update();
            }
        } else {
            self.in_tasks
                .insert(task.project.clone(), Vec::from([task.id]));
            self.update();
        }

        let dis_member = self.member().await.unwrap();
        Logger::debug(
            "member.join_task",
            &format!(
                "{} ({}) joined to task \"{}\"",
                dis_member.display_name(),
                self.id.get(),
                task.id
            ),
        )
        .await;
    }

    pub async fn add_done_task(&mut self, project_name: &String, task: u32) {
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

        self.update_last_activity(&project_name).await;
        self.update();

        let dis_member = self.member().await.unwrap();
        Logger::debug(
            "member.add_done_task",
            &format!(
                "{} ({}) added done task {}",
                dis_member.display_name(),
                self.id.get(),
                task
            ),
        )
        .await;
    }

    pub async fn remove_done_task(&mut self, project_name: &String, task_index: usize) {
        let member = self.member().await.unwrap();

        if let Some(tasks) = self.done_tasks.get_mut(project_name) {
            Logger::debug(
                "member.remove_done_task",
                &format!(
                    "task \"{}\" deleted from done tasks of member {} ({})",
                    match tasks.get(task_index) {
                        Some(task) => task.get().await,
                        None => String::from("Not Found"),
                    },
                    member.display_name(),
                    self.id.get().to_string()
                ),
            )
            .await;

            tasks.remove(task_index);
            if tasks.is_empty() {
                self.done_tasks.remove(project_name);
            }

            self.update();
        }
    }

    pub async fn add_mentor_task(&mut self, project_name: &String, task: u32) {
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

        self.update_last_activity(&project_name).await;
        self.update();

        let member = self.member().await.unwrap();
        Logger::debug(
            "member.add_mentor_task",
            &format!(
                "added mentor task {} to member {} ({})",
                task,
                member.display_name(),
                self.id.get()
            ),
        )
        .await;
    }

    pub async fn remove_mentor_task(&mut self, project_name: &String, task_index: usize) {
        let member = self.member().await.unwrap();

        if let Some(tasks) = self.mentor_tasks.get_mut(project_name) {
            Logger::debug(
                "member.remove_mentor_task",
                &format!(
                    "task \"{}\" deleted from mentor tasks of member {} ({})",
                    match tasks.get(task_index) {
                        Some(task) => task.get().await,
                        None => String::from("Not Found"),
                    },
                    member.display_name(),
                    self.id.get().to_string()
                ),
            )
            .await;

            tasks.remove(task_index);
            if tasks.is_empty() {
                self.mentor_tasks.remove(project_name);
            }

            self.update();
        }
    }

    pub async fn add_custom_done_task(&mut self, project: &String, task: TaskHistory) {
        if let TaskHistory::OldFormat(ref string) = task {
            let member = self.member().await.unwrap();
            Logger::medium(
                "member.add_custom_done_task",
                &format!(
                    "added custom task \"{}\" to done tasks of member {} ({})",
                    string,
                    member.display_name(),
                    self.id.get()
                ),
            )
            .await;

            if let None = self.done_tasks.get(project) {
                self.done_tasks.insert(project.clone(), Vec::new());
            }

            self.done_tasks.get_mut(project).unwrap().push(task);
            self.update();
        }
    }

    pub async fn add_custom_mentor_task(&mut self, project: &String, task: TaskHistory) {
        if let TaskHistory::OldFormat(ref string) = task {
            let member = self.member().await.unwrap();
            Logger::medium(
                "member.add_custom_mentor_task",
                &format!(
                    "added custom task \"{}\" to mentor tasks of member {} ({})",
                    string,
                    member.display_name(),
                    self.id.get()
                ),
            )
            .await;

            if let None = self.mentor_tasks.get(project) {
                self.mentor_tasks.insert(project.clone(), Vec::new());
            }

            self.mentor_tasks.get_mut(project).unwrap().push(task);
            self.update();
        }
    }

    pub async fn add_note(&mut self, user: UserId, note: String) {
        Logger::medium(
            "member.add_note",
            &format!(
                "user {} issued a note **\"{}\"** to a user {}",
                user.get(),
                &note,
                self.id.get()
            ),
        )
        .await;

        self.notes
            .push(NotesHistory::Current((user, Timestamp::now(), note)));
        self.update();
    }

    pub async fn remove_note(&mut self, user: UserId, index: usize) {
        Logger::low(
            "member.remove_note",
            &format!(
                "user {} deleted a note **\"{}\"** to a user {}",
                user.get(),
                match self.notes.get(index) {
                    Some(note) => {
                        match note {
                            NotesHistory::Current((_, _, string)) => string.clone(),
                            NotesHistory::OldFormat(string) => string.clone(),
                        }
                    }
                    None => String::from("Not Found"),
                },
                self.id.get()
            ),
        )
        .await;

        self.notes.remove(index);
        self.update();
    }

    pub async fn add_warn(&mut self, user: UserId, warn: String) {
        Logger::high(
            "member.add_warn",
            &format!(
                "user {} issued a warn **\"{}\"** to a user {}",
                user.get(),
                &warn,
                self.id.get()
            ),
        )
        .await;

        self.warns
            .push(NotesHistory::Current((user, Timestamp::now(), warn)));
        self.update();
    }

    pub async fn remove_warn(&mut self, user: UserId, index: usize) {
        Logger::low(
            "member.remove_warn",
            &format!(
                "user {} deleted a warn **\"{}\"** to a user {}",
                user.get(),
                match self.warns.get(index) {
                    Some(warn) => {
                        match warn {
                            NotesHistory::Current((_, _, string)) => string.clone(),
                            NotesHistory::OldFormat(string) => string.clone(),
                        }
                    }
                    None => String::from("Not Found"),
                },
                self.id.get()
            ),
        )
        .await;

        self.warns.remove(index);
        self.update();
    }

    pub async fn update_last_activity(&mut self, project_name: &String) {
        self.last_activity
            .insert(project_name.clone(), Timestamp::now());
        self.update();

        Logger::debug(
            &format!("members.{}", self.id.get().to_string().as_str()),
            &format!("updated last activity for project \"{}\"", project_name),
        )
        .await;
    }

    pub fn to_project_stat(
        &self,
        member_name: String,
        project_name: &String,
    ) -> Result<(String, String, bool), String> {
        Ok((
            member_name,
            format!(
                r#"
                ╠︎ **{}:** {}
                ╠︎ **{}:** {}
                ╠︎ **{}:** {}
                ╠︎ **{}:** {}
                ╚ **{}:** {}
                "#,
                get_string("member-project-stat-done-tasks-name", None),
                match self.done_tasks.get(project_name) {
                    Some(tasks) => tasks.len(),
                    None => 0,
                },
                get_string("member-project-stat-mentor-tasks-name", None),
                match self.mentor_tasks.get(project_name) {
                    Some(tasks) => tasks.len(),
                    None => 0,
                },
                get_string("member-project-stat-in-tasks-name", None),
                match self.in_tasks.get(project_name) {
                    Some(tasks) => {
                        let mut value = String::new();
                        let task_man = TASKMANAGER
                            .try_read()
                            .map_err(|e| format!("cannot lock TASKMANAGER, {}", e.to_string()))?;

                        for task in tasks.iter() {
                            value = format!(
                                "{}\n╠︎ <#{}>",
                                value,
                                task_man.get(*task).unwrap().thread_id.get()
                            );
                        }
                        value
                    }
                    None => get_string("member-project-stat-no-in-tasks", None),
                },
                get_string("member-project-stat-last-activity-name", None),
                match self.last_activity.get(project_name) {
                    Some(activity) => format!("<t:{}:R>", activity.timestamp()),
                    None => get_string("member-project-stat-no-last-activity", None),
                },
                get_string("member-project-stat-score-name", None),
                self.score,
            ),
            true,
        ))
    }

    pub async fn to_embed(&self, ctx: &Context, show_secret: bool) -> CreateEmbed {
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

        let task_man = match TASKMANAGER.try_read() {
            Ok(man) => man,
            Err(_) => {
                Logger::error(
                    "member.to_embed",
                    "error while try_read TASKMANAGER, maybe deadlock, trying await...",
                )
                .await;
                TASKMANAGER.read().await
            }
        };

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
                    for (proj, tasks) in self.in_tasks.iter() {
                        if !tasks.is_empty() {
                            value = format!("{}**{} ({}):**\n", value, proj, tasks.len());

                            for id in tasks.iter() {
                                if let Some(task) = task_man.get(*id) {
                                    value = format!(
                                        "{}{} <#{}>\n",
                                        value,
                                        match id == tasks.last().unwrap() {
                                            false => "╠︎",
                                            true => "╚",
                                        },
                                        task.thread_id.get()
                                    );
                                }
                            }
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
                            value = format!("{}**{} ({})**\n", value, proj, tasks.len());

                            for task in tasks.iter() {
                                value = format!(
                                    "{}{} {}\n",
                                    value,
                                    match task == tasks.last().unwrap() {
                                        false => "╠︎",
                                        true => "╚",
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
                            value = format!("{}**{} ({})**\n", value, proj, tasks.len());

                            for task in tasks.iter() {
                                value = format!(
                                    "{}{} {}\n",
                                    value,
                                    match task == tasks.last().unwrap() {
                                        false => "╠︎",
                                        true => "╚",
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
                format!("`{}`", folder),
                false,
            );
        }

        embed = embed
            .field(
                get_string("member-stat-embed-score-name", None),
                format!("`{}`", self.score),
                false,
            )
            .field(
                get_string("member-stat-embed-all-time-score-name", None),
                format!("`{}`", self.all_time_score),
                false,
            );

        if show_secret {
            if !self.last_activity.is_empty() {
                embed = embed.field(
                    get_string("member-stat-embed-last-activity-name", None),
                    {
                        let mut value = String::new();
                        let mut i = 0;

                        for (proj, time) in &self.last_activity {
                            i += 1;
                            value = format!(
                                "{}{} **{}**: <t:{}:R>\n",
                                value,
                                match i == self.last_activity.len() {
                                    false => "╠︎",
                                    true => "╚",
                                },
                                proj,
                                time.timestamp()
                            );
                        }
                        value
                    },
                    false,
                );
            }

            if !self.notes.is_empty() {
                embed = embed.field(
                    get_string(
                        "member-stat-embed-notes-name",
                        Some(HashMap::from([(
                            "num",
                            self.notes.len().to_string().as_str(),
                        )])),
                    ),
                    {
                        let mut value = String::new();

                        for note in &self.notes {
                            value = format!(
                                "{}{} {}\n",
                                value,
                                match note == self.notes.last().unwrap() {
                                    false => "╠︎",
                                    true => "╚",
                                },
                                match note {
                                    NotesHistory::OldFormat(string) => string.clone(),
                                    NotesHistory::Current((user, time, string)) => {
                                        format!(
                                            "<@{}> <t:{}:D>: {}",
                                            user.get(),
                                            time.timestamp(),
                                            string
                                        )
                                    }
                                }
                            );
                        }
                        value
                    },
                    false,
                );
            }

            if !self.warns.is_empty() {
                embed = embed.field(
                    get_string(
                        "member-stat-embed-warns-name",
                        Some(HashMap::from([(
                            "num",
                            self.warns.len().to_string().as_str(),
                        )])),
                    ),
                    {
                        let mut value = String::new();

                        for warn in &self.warns {
                            value = format!(
                                "{}{} {}\n",
                                value,
                                match warn == self.warns.last().unwrap() {
                                    false => "╠︎",
                                    true => "╚",
                                },
                                match warn {
                                    NotesHistory::OldFormat(string) => string.clone(),
                                    NotesHistory::Current((user, time, string)) => {
                                        format!(
                                            "<@{}> <t:{}:D>: {}",
                                            user.get(),
                                            time.timestamp(),
                                            string
                                        )
                                    }
                                }
                            );
                        }
                        value
                    },
                    false,
                );
            }
        }

        embed
    }
}
