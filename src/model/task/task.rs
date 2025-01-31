use crate::{
    model::{
        member::MEMBERSMANAGER,
        tag::{TageTypes, TAGSMANAGER},
    },
    prelude::*,
};
use once_cell::sync::Lazy;
use serde::{Deserialize, Serialize};
use serde_json;
use serenity::{
    all::{Colour, CreateActionRow, CreateEmbed, CreateSelectMenu, CreateSelectMenuOption},
    builder::{CreateMessage, EditThread},
    client::Context,
    model::{
        channel::GuildChannel,
        id::{ChannelId, RoleId, UserId},
        timestamp::Timestamp,
    },
};
use std::fs;
use std::{collections::HashMap, sync::Arc};
use tokio::sync::RwLock;
use walkdir::WalkDir;

pub static TASKMANAGER: Lazy<Arc<RwLock<TaskManager>>> =
    Lazy::new(|| Arc::new(RwLock::new(TaskManager::new())));

#[derive(Deserialize, Serialize, Clone, Debug)]
pub struct TaskOption<T: Clone> {
    base_value: T,
    modified_value: Option<T>,
    value_history: HashMap<Timestamp, T>,
}

impl<T: Clone> TaskOption<T> {
    pub fn new(value: T) -> Self {
        Self {
            base_value: value.clone(),
            modified_value: None,
            value_history: HashMap::from([(Timestamp::now(), value)]),
        }
    }

    pub fn get(&self) -> &T {
        if let Some(ref value) = self.modified_value {
            return value;
        }

        &self.base_value
    }

    pub fn get_mut(&mut self) -> &mut T {
        if let Some(ref mut value) = self.modified_value {
            return value;
        }

        &mut self.base_value
    }

    pub fn set_base(&mut self, value: T) {
        if let None = self.modified_value {
            self.value_history.insert(Timestamp::now(), value.clone());
        }

        self.base_value = value;
    }

    pub fn set(&mut self, value: T) {
        self.value_history.insert(Timestamp::now(), value.clone());
        self.modified_value = Some(value);
    }
}

#[derive(Deserialize, Debug)]
pub struct TaskManager {
    tasks: HashMap<u32, Task>,
    last_task_id: u32,
}

impl TaskManager {
    fn new() -> Self {
        Self {
            tasks: HashMap::new(),
            last_task_id: 0,
        }
    }

    pub async fn init(&mut self) {
        if !fs::exists(DATA_PATH.join("databases/tasks")).unwrap() {
            fs::create_dir_all(DATA_PATH.join("databases/tasks"))
                .expect("error while creating folder data/databases/tasks");
        }

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

            self.tasks.insert(task.id, task);
        }

        Logger::debug("tasks_man.init", "initialized from databases/tasks/*").await;
    }

    pub async fn new_task(
        &mut self,
        ctx: &Context,
        thread: &mut GuildChannel,
        project: String,
        waiter_role: Option<RoleId>,
    ) -> Result<u32, String> {
        let task = Task::new(
            &ctx,
            self.last_task_id + 1,
            project.clone(),
            waiter_role,
            thread,
        )
        .await?;
        self.last_task_id += 1;

        Logger::low(
            "tasks_man.new_task",
            &format!(
                "created new task {} for project {}",
                task.name.get(),
                project
            ),
        )
        .await;

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
        for task in self.tasks.values() {
            if task.thread_id == thread_id {
                return Some(task);
            }
        }

        None
    }

    pub fn get_thread_mut(&mut self, thread_id: ChannelId) -> Option<&mut Task> {
        for task in self.tasks.values_mut() {
            if task.thread_id == thread_id {
                return Some(task);
            }
        }

        None
    }

    pub fn get_by_project(&self, project: &String) -> Vec<&Task> {
        self.tasks
            .values()
            .filter(|task| &task.project == project)
            .collect()
    }
}

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct Task {
    pub id: u32,
    pub project: String,
    pub thread_id: ChannelId,
    pub finished: bool,
    pub name: TaskOption<String>,
    pub score: TaskOption<i64>,
    pub max_members: TaskOption<u32>,
    pub mentor_id: TaskOption<Option<UserId>>,
    pub members: TaskOption<Vec<UserId>>,
    pub start_date: Option<Timestamp>,
    pub end_date: TaskOption<Option<Timestamp>>,
    pub last_save: TaskOption<Option<String>>,
    #[serde(default, skip_serializing)]
    pub ending_results: HashMap<UserId, f64>,
}

impl Task {
    async fn new(
        ctx: &Context,
        id: u32,
        project: String,
        waiter_role: Option<RoleId>,
        thread: &mut GuildChannel,
    ) -> Result<Self, String> {
        let mut instance = Self {
            id,
            project,
            thread_id: thread.id,
            finished: false,
            name: TaskOption::new(thread.name.clone()),
            score: TaskOption::new(0),
            max_members: TaskOption::new(10000),
            mentor_id: TaskOption::new(None),
            members: TaskOption::new(Vec::new()),
            start_date: match thread.thread_metadata {
                Some(meta) => meta.create_timestamp,
                None => {
                    return Err("no found thread metadata, maybe its not a thread".to_string());
                }
            },
            end_date: TaskOption::new(None),
            last_save: TaskOption::new(None),
            ending_results: HashMap::new(),
        };

        if let Some(tags) = TAGSMANAGER
            .try_read()
            .map_err(|e| e.to_string())?
            .get_by_type(&thread.parent_id.unwrap(), TageTypes::InWork)
        {
            let mut new_tags = thread.applied_tags.clone();
            for tag in tags {
                if !new_tags.contains(&tag) {
                    new_tags.push(tag);
                }
            }

            thread
                .edit_thread(&ctx.http, EditThread::new().applied_tags(new_tags))
                .await
                .map_err(|e| format!("cannot change thread tags, {}", e.to_string()))?
        }

        if let Some(ping_msg) = instance.get_roles_ping(&thread, waiter_role) {
            thread
                .send_message(&ctx.http, CreateMessage::new().content(ping_msg))
                .await
                .map_err(|e| {
                    format!(
                        "cannot send ping message in task \"{}\", {}",
                        instance.name.get(),
                        e.to_string()
                    )
                })?;
        }

        instance.fetch_tags(&thread).await;
        instance.serialize().await;
        Ok(instance)
    }

    async fn serialize(&self) {
        write_file(
            &DATA_PATH.join(format!("databases/tasks/{}", self.id)),
            match serde_json::to_string(&self) {
                Ok(content) => content,
                Err(e) => {
                    Logger::error(
                        "task.serialize",
                        &format!(
                            "cannot serialize task {} \"{}\": {}",
                            self.id,
                            self.name.get(),
                            e.to_string()
                        ),
                    )
                    .await;
                    return;
                }
            },
        );
    }

    pub async fn fetch_tags(&mut self, thread: &GuildChannel) {
        let tags_man = TAGSMANAGER.read().await;
        let mut max_members = 10000;
        let mut score_modifier = 0;

        for dis_tag in thread.applied_tags.iter() {
            if let Some(tag) = tags_man.get(dis_tag) {
                if let Some(num) = tag.max_members {
                    max_members = num;
                }

                if let Some(num) = tag.score_modifier {
                    score_modifier = num;
                }

                if let Some(project) = &tag.task_project {
                    self.project = project.clone();
                }
            }
        }

        self.max_members.set_base(max_members);
        self.score.set_base(score_modifier);

        Logger::debug(
            "task.fetch_tags",
            &format!("updated tags of task \"{}\"", self.name.get()),
        )
        .await;
    }

    pub async fn update(&self) {
        self.serialize().await;
    }

    pub async fn close(&mut self, ctx: &Context) {
        if self.finished {
            return;
        }

        let mut mem_man = match MEMBERSMANAGER.try_write() {
            Ok(man) => man,
            Err(_) => {
                Logger::error("task.close", "cannot lock MEMBERSMANAGER for write").await;
                return;
            }
        };

        for member_id in self.members.get().iter() {
            if let Ok(member) = mem_man.get_mut(member_id.clone()).await {
                member.leave_task(&self).await;

                let end_score = self.ending_results.get(member_id).unwrap_or(&1.0).round() as i64;

                member.change_score(end_score).await;

                if end_score > 0 {
                    if &Some(member_id.clone()) != self.mentor_id.get() {
                        member.add_done_task(&self.project, self.id).await;
                    } else {
                        member.add_mentor_task(&self.project, self.id).await;
                    }
                }
            }
        }
        drop(mem_man);

        self.finished = true;
        self.members.get_mut().clear();
        self.mentor_id.set(None);
        self.end_date.set(Some(Timestamp::now()));
        self.update().await;

        Logger::low(
            "task.close",
            &format!("task \"{}\" closed", self.name.get()),
        )
        .await;

        let mut thread = match fetch_thread(&ctx, self.thread_id) {
            Ok(thread) => thread,
            Err(e) => {
                Logger::error(
                    "task.close",
                    &format!(
                        "cannot fetch thread of task \"{}\", because: {}",
                        self.name.get(),
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
                        self.name.get(),
                        e.to_string()
                    ),
                )
                .await;
            }
        }

        match thread
            .edit_thread(
                &ctx.http,
                EditThread::new()
                    .applied_tags(
                        TAGSMANAGER
                            .try_read()
                            .expect("task.close")
                            .get_by_type(&thread.parent_id.unwrap(), TageTypes::ClosedTask)
                            .unwrap_or(Vec::new()),
                    )
                    .locked(true),
            )
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

    pub async fn open(&mut self, ctx: &Context) {
        if !self.finished {
            return;
        }

        self.finished = false;
        self.end_date.set(None);
        self.update().await;

        Logger::low(
            "task.open",
            &format!("task \"{}\" reopened", self.name.get()),
        )
        .await;

        let mut thread = match fetch_thread(&ctx, self.thread_id) {
            Ok(thread) => thread,
            Err(e) => {
                Logger::error(
                    "task.open",
                    &format!(
                        "cannot fetch thread of task \"{}\", because: {}",
                        self.name.get(),
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
                CreateMessage::new().content(get_string("task-opened", None)),
            )
            .await
        {
            Ok(_) => (),
            Err(e) => {
                Logger::error(
                    "task.open",
                    &format!(
                        "cannot send message about opening task \"{}\": {}",
                        self.name.get(),
                        e.to_string()
                    ),
                )
                .await;
            }
        }

        let tags = TAGSMANAGER
            .read()
            .await
            .get_by_type(&thread.parent_id.unwrap(), TageTypes::ClosedTask)
            .unwrap_or(Vec::new());

        match thread
            .edit_thread(
                &ctx.http,
                EditThread::new()
                    .applied_tags(
                        thread
                            .applied_tags
                            .iter()
                            .filter(|x| !tags.contains(x))
                            .cloned(),
                    )
                    .locked(false),
            )
            .await
        {
            Ok(_) => (),
            Err(e) => {
                Logger::error(
                    "task.open",
                    &format!(
                        "cannot change thread tags and unlock thread, {}",
                        e.to_string()
                    ),
                )
                .await
            }
        }
    }

    pub async fn set_mentor(
        &mut self,
        ctx: &Context,
        mentor_id: Option<UserId>,
        ignore_limits: bool,
    ) -> bool {
        if self.finished {
            return false;
        }

        if let Some(id) = mentor_id {
            if !self.members.get().contains(&id) {
                if !self.add_member(&ctx, id.clone(), ignore_limits).await {
                    return false;
                }
            }
        }

        self.mentor_id.set(mentor_id);
        self.update().await;

        Logger::medium(
            "task.set_mentor",
            &format!(
                "task \"{}\" mentor now is {:?}",
                self.name.get(),
                self.mentor_id.get()
            ),
        )
        .await;

        let thread = match fetch_thread(&ctx, self.thread_id) {
            Ok(thread) => thread,
            Err(e) => {
                Logger::error(
                    "task.set_mentor",
                    &format!(
                        "cannot fetch thread of task \"{}\", because: {}",
                        self.name.get(),
                        e.to_string()
                    ),
                )
                .await;
                return false;
            }
        };

        match thread
            .send_message(
                &ctx.http,
                CreateMessage::new().content(match self.mentor_id.get() {
                    Some(id) => get_string(
                        "task-mentor-changed",
                        Some(HashMap::from([("mentor", id.get().to_string().as_str())])),
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
                        self.name.get(),
                        e.to_string()
                    ),
                )
                .await;
            }
        }

        true
    }

    pub async fn set_last_save(&mut self, ctx: &Context, last_save: Option<String>) {
        self.last_save.set(last_save);
        self.update().await;

        Logger::low(
            "task.set_last_save",
            &format!(
                "task \"{}\" last save now is {:?}",
                self.name.get(),
                self.last_save.get()
            ),
        )
        .await;

        if self.members.get().len() >= *self.max_members.get() as usize {
            let thread = match fetch_thread(&ctx, self.thread_id) {
                Ok(thread) => thread,
                Err(e) => {
                    Logger::error(
                        "task.set_last_save",
                        &format!(
                            "cannot fetch thread of task \"{}\", because: {}",
                            self.name.get(),
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
                    CreateMessage::new().content(match self.last_save.get() {
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
                            self.name.get(),
                            e.to_string()
                        ),
                    )
                    .await;
                }
            }
        }
    }

    pub async fn set_max_members(&mut self, ctx: &Context, max_members: u32) {
        let old_max_members = *self.max_members.get();

        self.max_members.set(max_members);
        self.update().await;

        Logger::medium(
            "task.set_max_members",
            &format!(
                "task \"{}\" max members changed to {}",
                self.name.get(),
                self.max_members.get()
            ),
        )
        .await;

        let thread = match fetch_thread(&ctx, self.thread_id) {
            Ok(thread) => thread,
            Err(e) => {
                Logger::error(
                    "task.set_max_members",
                    &format!(
                        "cannot fetch thread of task \"{}\", because: {}",
                        self.name.get(),
                        e.to_string()
                    ),
                )
                .await;
                return;
            }
        };

        if self.members.get().len() >= *self.max_members.get() as usize {
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
        } else if self.members.get().len() < *self.max_members.get() as usize
            && self.members.get().len() >= old_max_members as usize
        {
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

        match thread
            .send_message(
                &ctx.http,
                CreateMessage::new().content(get_string(
                    "task-max-members-change",
                    Some(HashMap::from([("num", max_members.to_string().as_str())])),
                )),
            )
            .await
        {
            Ok(_) => (),
            Err(e) => {
                Logger::error(
                    "task.set_max_members",
                    &format!(
                        "cannot send message about changing task max members: {}",
                        e.to_string()
                    ),
                )
                .await;
            }
        }
    }

    pub async fn set_score(&mut self, ctx: &Context, score: i64) {
        let old_score = *self.score.get();

        if old_score == score {
            return;
        }

        self.score.set(score);
        self.update().await;

        Logger::medium(
            "task.set_score",
            &format!(
                "task \"{}\" score changed from {} to {}",
                self.name.get(),
                old_score,
                self.score.get()
            ),
        )
        .await;

        let thread = match fetch_thread(&ctx, self.thread_id) {
            Ok(thread) => thread,
            Err(e) => {
                Logger::error(
                    "task.set_score",
                    &format!(
                        "cannot fetch thread of task \"{}\", because: {}",
                        self.name.get(),
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
                CreateMessage::new().content(get_string(
                    "task-score-changed",
                    Some(HashMap::from([
                        ("old", old_score.to_string().as_str()),
                        ("new", self.score.get().to_string().as_str()),
                    ])),
                )),
            )
            .await
        {
            Ok(_) => (),
            Err(e) => {
                Logger::error(
                    "task.set_score",
                    &format!(
                        "cannot send message about changing task score: {}",
                        e.to_string()
                    ),
                )
                .await;
            }
        }
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
        for member in self.members.get().iter() {
            ping = format!("{} <@{}>", ping, member.get());
        }

        if ping == String::new() {
            ping = get_string("task-no-ping", None);
        }

        ping
    }

    pub async fn remove_member(&mut self, ctx: &Context, member: UserId) {
        let members = self.members.get().clone();

        self.members
            .get_mut()
            .remove(match members.iter().position(|x| x == &member) {
                Some(index) => index,
                None => {
                    return ();
                }
            });

        if &Some(member) == self.mentor_id.get() {
            self.set_mentor(&ctx, None, false).await;
        }

        match MEMBERSMANAGER.try_write().as_mut() {
            Ok(man) => {
                if let Ok(mem) = man.get_mut(member.clone()).await {
                    mem.leave_task(&self).await;
                }
            }
            Err(_) => {
                Logger::error("task.remove_member", "cannot lock MEMBERSMANAGER for write").await;
                return;
            }
        };

        self.update().await;

        Logger::medium(
            "task.remove_member",
            &format!(
                "member {} removed from task \"{}\" members",
                member.get(),
                self.name.get()
            ),
        )
        .await;

        if self.members.get().len() + 1 == *self.max_members.get() as usize {
            let thread = match fetch_thread(&ctx, self.thread_id) {
                Ok(thread) => thread,
                Err(e) => {
                    Logger::error(
                        "task.remove_member",
                        &format!(
                            "cannot fetch thread of task \"{}\", because: {}",
                            self.name.get(),
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

    pub async fn add_member(&mut self, ctx: &Context, member: UserId, ignore_limits: bool) -> bool {
        if self.members.get().len() < *self.max_members.get() as usize {
            if self.members.get().contains(&member) {
                return true;
            }

            if self.finished {
                return false;
            }

            if let Some(project) = project::PROJECTMANAGER.read().await.get(&self.project) {
                if !project.member_in_project(&match fetch_member(&member).await {
                    Ok(m) => m,
                    Err(e) => {
                        Logger::error(
                            "task.add_member",
                            &format!(
                                "cannot fetch member by id {}: {}",
                                member.get(),
                                e.to_string()
                            ),
                        )
                        .await;
                        return false;
                    }
                }) {
                    return false;
                }
            }

            let mut mem_man = MEMBERSMANAGER.write().await;
            if let Ok(mem) = mem_man.get_mut(member.clone()).await {
                if (mem.in_tasks.get(&self.project).unwrap_or(&Vec::new()).len()
                    < match project::PROJECTMANAGER.read().await.get(&self.project) {
                        Some(project) => project.max_tasks_per_user as usize,
                        None => usize::max_value(),
                    })
                    || ignore_limits
                {
                    mem.join_task(&self).await;
                } else {
                    return false;
                }
            }

            self.members.get_mut().push(member);
            self.update().await;

            Logger::low(
                "task.add_member",
                &format!(
                    "member {} added to task \"{}\" members",
                    member.get(),
                    self.name.get()
                ),
            )
            .await;

            let thread = match fetch_thread(&ctx, self.thread_id) {
                Ok(thread) => thread,
                Err(e) => {
                    Logger::error(
                        "task.add_member",
                        &format!(
                            "cannot fetch thread of task \"{}\", because: {}",
                            self.name.get(),
                            e.to_string()
                        ),
                    )
                    .await;
                    return false;
                }
            };

            match thread
                .send_message(
                    &ctx.http,
                    CreateMessage::new().content(get_string(
                        "task-join-message",
                        Some(HashMap::from([(
                            "member",
                            member.get().to_string().as_str(),
                        )])),
                    )),
                )
                .await
            {
                Ok(_) => (),
                Err(e) => {
                    Logger::error(
                        "task.add_member",
                        &format!("cannot send message about joining task: {}", e.to_string()),
                    )
                    .await;
                }
            }

            if self.members.get().len() == *self.max_members.get() as usize {
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

            return true;
        }

        false
    }

    pub async fn closing_option(&self, member: &UserId) -> CreateActionRow {
        let cfg = CONFIG.read().await;

        let mut options = Vec::new();
        let mut index = 0;

        for (opt, ratio) in cfg.task_ratings.iter() {
            options.push(CreateSelectMenuOption::new(
                format!("{} (x{})", get_string(opt, None), ratio),
                format!(
                    "{}:::{}:::{}",
                    member.get(),
                    index,
                    *self.score.get() as f64 * ratio
                ),
            ));
            index += 1;
        }

        serenity::all::CreateActionRow::SelectMenu(
            CreateSelectMenu::new(
                "task-close:member-score",
                serenity::all::CreateSelectMenuKind::String {
                    options: options.clone(),
                },
            )
            .placeholder(match fetch_member(&member).await {
                Ok(mem) => mem.display_name().to_string(),
                Err(_) => format!("Unknown ({})", member.get()),
            }),
        )
    }

    pub fn to_embed(&self) -> CreateEmbed {
        let mut fields = Vec::new();

        fields.push((
            get_string("task-embed-id-name", None),
            format!("`{}`", self.id),
            false,
        ));

        if let Some(date) = self.start_date {
            fields.push((
                get_string("task-embed-start-date-name", None),
                format!("<t:{}:f>", date.timestamp()),
                false,
            ));
        }

        fields.push((
            get_string("task-embed-score-name", None),
            format!("`{}`", self.score.get()),
            false,
        ));

        fields.push((
            get_string("task-embed-last-save-name", None),
            format!(
                "`{}`",
                match self.last_save.get() {
                    Some(save) => save.clone(),
                    None => get_string("task-embed-no-last-save", None),
                }
            ),
            false,
        ));

        if let Some(mentor) = self.mentor_id.get() {
            fields.push((
                get_string("task-embed-mentor-name", None),
                format!("- <@{}>", mentor.get()),
                false,
            ));
        }

        let mut members_text = String::new();
        for member in self.members.get().iter() {
            members_text = format!("{}- <@{}>\n", members_text, member.get());
        }

        fields.push((
            get_string(
                "task-embed-members-name",
                Some(HashMap::from([
                    ("current", self.members.get().len().to_string().as_str()),
                    ("max", self.max_members.get().to_string().as_str()),
                ])),
            ),
            members_text,
            false,
        ));

        CreateEmbed::new()
            .title(get_string(
                "task-embed-title",
                Some(HashMap::from([("task", self.name.get().as_str())])),
            ))
            .color(Colour::ORANGE)
            .fields(fields)
    }
}
