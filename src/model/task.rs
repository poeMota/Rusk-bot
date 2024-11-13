use crate::prelude::*;
use once_cell::sync::Lazy;
use serde::{Deserialize, Serialize};
use serde_json;
use serenity::{
    builder::CreateMessage,
    client::Context,
    model::{
        id::{ChannelId, UserId},
        timestamp::Timestamp,
    },
};
use std::{collections::HashMap, sync::Arc};
use tokio::sync::RwLock;
use walkdir::WalkDir;

pub static TASKMANAGER: Lazy<Arc<RwLock<TaskManager>>> =
    Lazy::new(|| Arc::new(RwLock::new(TaskManager::new())));

#[derive(Deserialize, Debug)]
pub struct TaskManager {
    tasks: HashMap<u32, Task>,
    tasks_by_thread: HashMap<ChannelId, u32>,
    last_task_id: u32,
}

impl TaskManager {
    fn new() -> Self {
        Self {
            tasks: HashMap::new(),
            tasks_by_thread: HashMap::new(),
            last_task_id: 0,
        }
    }

    pub async fn init(&mut self) {
        for entry in WalkDir::new(DATA_PATH.join("databases/tasks")) {
            let entry = match entry {
                Ok(s) => s,
                Err(error) => {
                    Logger::error(
                        "tasks_man.init",
                        &format!("error with task data file: {}", error),
                    )
                    .await;
                    continue;
                }
            };

            if !entry.path().is_file() {
                continue;
            }

            let task: Task =
                match serde_yaml::from_str(read_file(&entry.path().to_path_buf()).as_str()) {
                    Ok(c) => c,
                    Err(e) => {
                        Logger::error(
                            "tasks_man.init",
                            &format!(
                                "error while parsing task data file \"{}\": {}",
                                entry.file_name().to_str().unwrap(),
                                e.to_string()
                            ),
                        )
                        .await;
                        continue;
                    }
                };

            if self.last_task_id < task.id {
                self.last_task_id = task.id;
            }

            self.tasks.insert(task.id, task.clone());
            self.tasks_by_thread.insert(task.thread_id, task.id);
        }

        Logger::debug("tasks_man.init", "initialized from databases/tasks/*").await;
    }

    pub async fn new_task(&mut self, ctx: &Context, thread_id: ChannelId) -> Result<u32, String> {
        self.last_task_id += 1;
        let task = Task::new(&ctx, self.last_task_id, thread_id).await?;

        self.tasks_by_thread
            .insert(task.thread_id.clone(), self.last_task_id);
        self.tasks.insert(self.last_task_id, task);

        Ok(self.last_task_id)
    }

    pub fn get(&self, id: u32) -> Option<&Task> {
        self.tasks.get(&id)
    }

    pub fn get_mut(&mut self, id: u32) -> Option<&mut Task> {
        self.tasks.get_mut(&id)
    }

    pub fn get_thread(&self, thread_id: ChannelId) -> Option<&Task> {
        self.tasks.get(self.tasks_by_thread.get(&thread_id)?)
    }

    pub fn get_thread_mut(&mut self, thread_id: ChannelId) -> Option<&mut Task> {
        self.tasks
            .get_mut(self.tasks_by_thread.get_mut(&thread_id)?)
    }
}

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct Task {
    pub id: u32,
    pub thread_id: ChannelId,
    pub finished: bool,
    pub name: String,
    pub score: i64,
    pub max_members: u32,
    pub mentor_id: Option<UserId>,
    pub members: Vec<UserId>,
    pub start_date: Option<Timestamp>,
    pub last_save: Option<String>,
}

impl Task {
    async fn new(ctx: &Context, id: u32, thread_id: ChannelId) -> Result<Self, String> {
        let thread = fetch_channel(&ctx, thread_id)?;

        let mut instance = Self {
            id,
            thread_id,
            finished: false,
            name: thread.name.clone(),
            score: 0,
            max_members: 10000,
            mentor_id: None,
            members: Vec::new(),
            start_date: match thread.thread_metadata {
                Some(meta) => meta.create_timestamp,
                None => {
                    return Err("no found thread metadata, maybe its not a thread".to_string());
                }
            },
            last_save: None,
        };

        instance.fetch_tags(&ctx).await;
        Ok(instance)
    }

    fn serialize(&self) {
        write_file(
            &DATA_PATH.join(format!("databases/tasks/{}", self.id)),
            serde_json::to_string(&self).unwrap(),
        );
    }

    async fn fetch_tags(&mut self, ctx: &Context) {
        match fetch_channel(&ctx, self.thread_id) {
            Ok(thread) => {
                let tags_man = match TAGSMANAGER.try_read() {
                    Ok(man) => man,
                    Err(e) => {
                        Logger::error(
                            "task.fetch_tags",
                            &format!(
                                "cannot lock TAGSMANAGER for read because: {}",
                                e.to_string()
                            ),
                        )
                        .await;
                        return;
                    }
                };

                for dis_tag in thread.applied_tags.iter() {
                    if let Some(tag) = tags_man.get(dis_tag) {
                        if let Some(max_members) = tag.max_members {
                            self.max_members = max_members;
                        }

                        if let Some(score_modifier) = tag.score_modifier {
                            self.score = score_modifier;
                        }
                    }
                }
            }
            Err(e) => {
                Logger::error(
                    "task.fetch_tags",
                    &format!(
                        "cannot fetch task thread \"{}\", because: {}",
                        self.name,
                        e.to_string()
                    ),
                )
                .await;
                return;
            }
        }
    }

    fn update(&self) {
        self.serialize();
    }

    pub fn set_mentor(&mut self, mentor_id: Option<UserId>) {
        self.mentor_id = mentor_id;
        self.update();
    }

    pub fn set_last_save(&mut self, last_save: Option<String>) {
        self.last_save = last_save;
        self.update();
    }

    pub async fn set_max_members(&mut self, ctx: &Context, max_members: u32) {
        let old_max_members = self.max_members;

        self.max_members = max_members;
        self.update();

        if self.members.len() >= self.max_members as usize {
            let thread = match fetch_channel(&ctx, self.thread_id) {
                Ok(thread) => thread,
                Err(e) => {
                    Logger::error(
                        "task.set_max_members",
                        &format!(
                            "cannot fetch thread of task \"{}\", because: {}",
                            self.name,
                            e.to_string()
                        ),
                    )
                    .await;
                    return;
                }
            };

            match thread
                .send_message(
                    &ctx.http,
                    CreateMessage::new().content(get_string("task-members-filled", None)),
                )
                .await
            {
                Ok(_) => (),
                Err(e) => {
                    Logger::error(
                        "task.set_max_members",
                        &format!("cannot send message about filled task: {}", e.to_string()),
                    )
                    .await;
                }
            }
        } else if self.members.len() < self.max_members as usize
            && self.members.len() >= old_max_members as usize
        {
            let thread = match fetch_channel(&ctx, self.thread_id) {
                Ok(thread) => thread,
                Err(e) => {
                    Logger::error(
                        "task.set_max_members",
                        &format!(
                            "cannot fetch thread of task \"{}\", because: {}",
                            self.name,
                            e.to_string()
                        ),
                    )
                    .await;
                    return;
                }
            };

            match thread
                .send_message(
                    &ctx.http,
                    CreateMessage::new().content(get_string("task-members-unfilled", None)),
                )
                .await
            {
                Ok(_) => (),
                Err(e) => {
                    Logger::error(
                        "task.set_max_members",
                        &format!(
                            "cannot send message about unfilling task: {}",
                            e.to_string()
                        ),
                    )
                    .await;
                }
            }
        }
    }

    pub fn set_score(&mut self, score: i64) {
        self.score = score;
        self.update();
    }

    pub fn get_members_ping(&self) -> String {
        let mut ping = String::new();
        for member in self.members.iter() {
            ping = format!("{} <@{}>", ping, member.get());
        }

        ping
    }

    pub async fn remove_member(&mut self, ctx: &Context, member: UserId) {
        self.members
            .remove(match self.members.iter().position(|x| x == &member) {
                Some(index) => index,
                None => {
                    return ();
                }
            });

        if Some(member) == self.mentor_id {
            self.mentor_id = None;
        }

        self.update();

        if self.members.len() + 1 == self.max_members as usize {
            let thread = match fetch_channel(&ctx, self.thread_id) {
                Ok(thread) => thread,
                Err(e) => {
                    Logger::error(
                        "task.remove_member",
                        &format!(
                            "cannot fetch thread of task \"{}\", because: {}",
                            self.name,
                            e.to_string()
                        ),
                    )
                    .await;
                    return;
                }
            };

            match thread
                .send_message(
                    &ctx.http,
                    CreateMessage::new().content(get_string("task-members-unfilled", None)),
                )
                .await
            {
                Ok(_) => (),
                Err(e) => {
                    Logger::error(
                        "task.remove_member",
                        &format!(
                            "cannot send message about unfilling task: {}",
                            e.to_string()
                        ),
                    )
                    .await;
                }
            }
        }
    }

    pub async fn add_member(&mut self, ctx: &Context, member: UserId) {
        if self.members.len() < self.max_members as usize && !self.members.contains(&member) {
            self.members.push(member);
            self.update();

            if self.members.len() == self.max_members as usize {
                let thread = match fetch_channel(&ctx, self.thread_id) {
                    Ok(thread) => thread,
                    Err(e) => {
                        Logger::error(
                            "task.add_member",
                            &format!(
                                "cannot fetch thread of task \"{}\", because: {}",
                                self.name,
                                e.to_string()
                            ),
                        )
                        .await;
                        return;
                    }
                };

                match thread
                    .send_message(
                        &ctx.http,
                        CreateMessage::new().content(get_string("task-members-filled", None)),
                    )
                    .await
                {
                    Ok(_) => (),
                    Err(e) => {
                        Logger::error(
                            "task.add_member",
                            &format!("cannot send message about filled task: {}", e.to_string()),
                        )
                        .await;
                    }
                }
            }
        }
    }
}
