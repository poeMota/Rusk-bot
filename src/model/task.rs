use crate::prelude::*;
use once_cell::sync::Lazy;
use serde::{Deserialize, Serialize};
use serde_json;
use serenity::{
    builder::{CreateMessage, EditThread},
    client::Context,
    model::{
        channel::GuildChannel,
        id::{ChannelId, RoleId, UserId},
        timestamp::Timestamp,
    },
};
use std::{collections::HashMap, fmt::format, sync::Arc};
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
        let task = Task::new(&ctx, self.last_task_id + 1, thread_id).await?;
        self.last_task_id += 1;

        Logger::low(
            "tasks_man.new_task",
            &format!("created new task {}", task.name),
        )
        .await;

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
    pub end_date: Option<Timestamp>,
    pub last_save: Option<String>,
}

impl Task {
    async fn new(ctx: &Context, id: u32, thread_id: ChannelId) -> Result<Self, String> {
        let mut thread = fetch_channel(&ctx, thread_id)?;

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
            end_date: None,
            last_save: None,
        };

        if let Some(tags) = TAGSMANAGER
            .try_read()
            .map_err(|e| e.to_string())?
            .get_by_type(&thread.parent_id.unwrap(), TageTypes::InWork)
        {
            let mut new_tags = thread.applied_tags.clone();
            new_tags.extend(tags.iter());

            thread
                .edit_thread(&ctx.http, EditThread::new().applied_tags(new_tags))
                .await
                .map_err(|e| format!("cannot change thread tags, {}", e.to_string()))?
        }

        if let Some(ping_msg) = instance.get_roles_ping(&thread, None) {
            thread
                .send_message(&ctx.http, CreateMessage::new().content(ping_msg))
                .await
                .map_err(|e| {
                    format!(
                        "cannot send ping message in task \"{}\", {}",
                        instance.name,
                        e.to_string()
                    )
                })?;
        }

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

    pub async fn close(&mut self, ctx: &Context) {
        let mut mem_man = match MEMBERSMANAGER.try_write() {
            Ok(man) => man,
            Err(_) => {
                Logger::error("task.close", "cannot lock MEMBERSMANAGER for write").await;
                return;
            }
        };

        for member_id in self.members.iter() {
            let member = mem_man.get_mut(member_id.clone()).await.unwrap();

            member.leave_task(self.id);
            member.change_score(self.score);
            // TODO
        }
        drop(mem_man);

        self.finished = true;
        self.members.clear();
        self.mentor_id = None;
        self.end_date = Some(Timestamp::now());
        self.update();

        let mut thread = match fetch_channel(&ctx, self.thread_id) {
            Ok(thread) => thread,
            Err(e) => {
                Logger::error(
                    "task.close",
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
                CreateMessage::new().content(get_string("task-closed", None)),
            )
            .await
        {
            Ok(_) => (),
            Err(e) => {
                Logger::error(
                    "task.close",
                    &format!(
                        "cannot send message about closing task \"{}\": {}",
                        self.name,
                        e.to_string()
                    ),
                )
                .await;
            }
        }

        if let Some(tags) = TAGSMANAGER
            .try_read()
            .expect("task.close")
            .get_by_type(&thread.parent_id.unwrap(), TageTypes::ClosedTask)
        {
            match thread
                .edit_thread(&ctx.http, EditThread::new().applied_tags(tags).locked(true))
                .await
            {
                Ok(_) => (),
                Err(e) => {
                    Logger::error(
                        "task.close",
                        &format!(
                            "cannot change thread tags and lock thread, {}",
                            e.to_string()
                        ),
                    )
                    .await
                }
            }
        }
    }

    pub async fn set_mentor(&mut self, ctx: &Context, mentor_id: Option<UserId>) {
        self.mentor_id = mentor_id;
        self.update();

        if let Some(id) = self.mentor_id {
            if !self.members.contains(&id) {
                self.add_member(&ctx, id).await;
            }
        }

        let thread = match fetch_channel(&ctx, self.thread_id) {
            Ok(thread) => thread,
            Err(e) => {
                Logger::error(
                    "task.set_mentor",
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
                CreateMessage::new().content(match self.mentor_id {
                    Some(id) => get_string(
                        "task-mentor-changed",
                        Some(HashMap::from([(
                            "mentor_id",
                            id.get().to_string().as_str(),
                        )])),
                    ),
                    None => get_string("task-no-more-mentor", None),
                }),
            )
            .await
        {
            Ok(_) => (),
            Err(e) => {
                Logger::error(
                    "task.set_mentor",
                    &format!(
                        "cannot send message about changing mentor of task \"{}\": {}",
                        self.name,
                        e.to_string()
                    ),
                )
                .await;
            }
        }
    }

    pub async fn set_last_save(&mut self, ctx: &Context, last_save: Option<String>) {
        self.last_save = last_save;
        self.update();

        if self.members.len() >= self.max_members as usize {
            let thread = match fetch_channel(&ctx, self.thread_id) {
                Ok(thread) => thread,
                Err(e) => {
                    Logger::error(
                        "task.set_last_save",
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
                    CreateMessage::new().content(match self.last_save {
                        Some(ref save) => get_string(
                            "task-last-save",
                            Some(HashMap::from([("save", save.as_str())])),
                        ),
                        None => get_string(
                            "task-last-save",
                            Some(HashMap::from([(
                                "save",
                                get_string("task-lask-save-not-specified", None).as_str(),
                            )])),
                        ),
                    }),
                )
                .await
            {
                Ok(_) => (),
                Err(e) => {
                    Logger::error(
                        "task.set_last_save",
                        &format!(
                            "cannot send message about changing last save of task \"{}\": {}",
                            self.name,
                            e.to_string()
                        ),
                    )
                    .await;
                }
            }
        }
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

    pub fn get_roles_ping(
        &self,
        thread: &GuildChannel,
        waiter_role: Option<RoleId>,
    ) -> Option<String> {
        let mut ping = String::new();

        if let Some(role_id) = waiter_role {
            ping = format!("<@&{}>", role_id.get());
        }

        let tags_man = TAGSMANAGER.try_read().expect("task.get_roles_ping");
        for tag in thread.applied_tags.iter() {
            if let Some(tag) = tags_man.get(tag) {
                if let Some(ping_role) = tag.ping_role {
                    ping = format!("{} <@&{}>", ping, ping_role.get());
                }
            }
        }

        if ping == String::new() {
            return None;
        }
        Some(ping)
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
            self.set_mentor(&ctx, None).await;
        }

        match MEMBERSMANAGER.try_write().as_mut() {
            Ok(man) => {
                if let Ok(mem) = man.get_mut(member.clone()).await {
                    mem.leave_task(self.id);
                }
            }
            Err(_) => {
                Logger::error("task.remove_member", "cannot lock MEMBERSMANAGER for write").await;
                return;
            }
        };

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

            match MEMBERSMANAGER.try_write().as_mut() {
                Ok(man) => {
                    if let Ok(mem) = man.get_mut(member.clone()).await {
                        mem.join_task(self.id);
                    }
                }
                Err(_) => {
                    Logger::error("task.add_member", "cannot lock MEMBERSMANAGER for write").await;
                    return;
                }
            };

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
