use crate::{model::member::MEMBERSMANAGER, prelude::*};
use once_cell::sync::Lazy;
use serde::{Deserialize, Serialize};
use serde_json;
use serenity::{
    all::Colour,
    builder::{CreateEmbed, EditMessage},
    model::{
        guild::Member,
        id::{ChannelId, MessageId, RoleId},
    },
};
use std::fs;
use std::{collections::HashMap, sync::Arc};
use tokio::sync::RwLock;
use tokio::time::{sleep, Duration};
use walkdir::WalkDir;

pub static PROJECTMANAGER: Lazy<Arc<RwLock<ProjectManager>>> =
    Lazy::new(|| Arc::new(RwLock::new(ProjectManager::new())));

#[derive(Debug)]
pub struct ProjectManager {
    projects: HashMap<String, Project>,
}

impl ProjectManager {
    fn new() -> Self {
        Self {
            projects: HashMap::new(),
        }
    }

    pub async fn init(&mut self) {
        if !fs::exists(DATA_PATH.join("databases/projects")).unwrap() {
            fs::create_dir_all(DATA_PATH.join("databases/projects"))
                .expect("error while creating folder data/databases/projects");
        }

        for entry in WalkDir::new(DATA_PATH.join("databases/projects")) {
            let entry = match entry {
                Ok(s) => s,
                Err(error) => {
                    Logger::error(
                        "proj_man.init",
                        &format!("error with project data file: {}", error),
                    )
                    .await;
                    continue;
                }
            };

            if !entry.path().is_file() {
                continue;
            }

            let project: Project =
                match serde_yaml::from_str(read_file(&entry.path().to_path_buf()).as_str()) {
                    Ok(c) => c,
                    Err(e) => {
                        Logger::error(
                            "proj_man.init",
                            &format!(
                                "error while parsing project data file \"{}\": {}",
                                entry.file_name().to_str().unwrap(),
                                e.to_string()
                            ),
                        )
                        .await;
                        continue;
                    }
                };

            self.projects.insert(project.name.clone(), project);
        }

        Logger::debug("proj_man.init", "initialized from databases/projects/*").await;
    }

    pub async fn new_project(
        &mut self,
        name: String,
        max_tasks_per_user: u32,
        tasks_forum: ChannelId,
        waiter_role: Option<RoleId>,
        stat_channel: Option<ChannelId>,
    ) -> Result<(), String> {
        if !self.projects.contains_key(&name) {
            let project = Project {
                name,
                max_tasks_per_user,
                tasks_forum,
                waiter_role,
                stat_channel,
                stat_posts: HashMap::new(),
                associated_roles: Vec::new(),
            };

            Logger::high(
                "proj_man.new_project",
                &format!("created new project \"{}\"", project.name),
            )
            .await;

            project.update().await;
            self.projects.insert(project.name.clone(), project);
        } else {
            return Err(format!("project with name \"{}\" currently excist", name));
        }

        Ok(())
    }

    pub async fn delete(&mut self, name: &String) -> Option<Project> {
        if let Some(proj) = self.projects.remove(name) {
            Logger::high(
                "proj_man.delete",
                &format!("deleted project \"{}\"", proj.name()),
            )
            .await;

            return Some(proj);
        }

        None
    }

    pub async fn start_update_stat(ctx: Context) {
        tokio::spawn(async move {
            let timer = CONFIG.read().await.project_stat_update_duration;

            loop {
                let mut man = PROJECTMANAGER.write().await;

                for project in man.projects.values_mut() {
                    project.update_stat_post(&ctx).await;
                }

                drop(man);
                sleep(Duration::from_secs(timer)).await;
            }
        });
    }

    pub fn projects(&self) -> Vec<&String> {
        self.projects.keys().collect()
    }

    pub fn get(&self, name: &String) -> Option<&Project> {
        self.projects.get(name)
    }

    pub fn get_mut(&mut self, name: &String) -> Option<&mut Project> {
        self.projects.get_mut(name)
    }

    pub fn get_from_forum(&self, forum: &ChannelId) -> Option<&Project> {
        for project in self.projects.values() {
            if &project.tasks_forum == forum {
                return Some(project);
            }
        }
        None
    }

    pub fn get_mut_from_forum(&mut self, forum: &ChannelId) -> Option<&mut Project> {
        for project in self.projects.values_mut() {
            if &project.tasks_forum == forum {
                return Some(project);
            }
        }
        None
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Project {
    pub name: String,
    pub max_tasks_per_user: u32,
    pub tasks_forum: ChannelId,
    pub waiter_role: Option<RoleId>,
    pub stat_posts: HashMap<RoleId, MessageId>,
    pub stat_channel: Option<ChannelId>,
    pub associated_roles: Vec<RoleId>,
}

impl Project {
    pub fn name(&self) -> &String {
        &self.name
    }

    async fn write(&self) {
        write_file(
            &DATA_PATH.join(format!("databases/projects/{}", self.name())),
            match serde_json::to_string(&self) {
                Ok(content) => content,
                Err(e) => {
                    Logger::error(
                        "project.write",
                        &format!("error with project \"{}\", {}", self.name, e.to_string()),
                    )
                    .await;
                    return;
                }
            },
        );
    }

    pub async fn update(&self) {
        self.write().await;
    }

    pub async fn set_max_task_per_user(&mut self, max_tasks: u32) {
        let old = self.max_tasks_per_user;

        self.max_tasks_per_user = max_tasks;
        self.update().await;

        Logger::high(
            "project.set_max_task_per_user",
            &format!(
                "max tasks per user of project \"{}\" changed from {} to {}",
                self.name(),
                old,
                self.max_tasks_per_user
            ),
        )
        .await;
    }

    pub async fn set_tasks_forum(&mut self, forum: ChannelId) {
        let old = self.tasks_forum.get();
        self.tasks_forum = forum;
        self.update().await;

        Logger::high(
            "project.set_tasks_forum",
            &format!(
                "task forum of project \"{}\" changed from {} to {}",
                self.name(),
                old,
                self.tasks_forum.get()
            ),
        )
        .await;
    }

    pub async fn set_waiter_role(&mut self, waiter: Option<RoleId>) {
        let old = self.waiter_role.clone();
        self.waiter_role = waiter;
        self.update().await;

        Logger::high(
            "project.set_waiter_role",
            &format!(
                "waiter role of project \"{}\" changed from {:?} to {:?}",
                self.name(),
                old,
                self.waiter_role
            ),
        )
        .await;
    }

    pub async fn set_stat_channel(&mut self, channel: Option<ChannelId>) {
        let old = self.stat_channel.clone();
        self.stat_channel = channel;
        self.update().await;

        Logger::high(
            "project.set_stat_channel",
            &format!(
                "stat channel of project \"{}\" changed from {:?} to {:?}",
                self.name(),
                old,
                self.stat_channel
            ),
        )
        .await;
    }

    pub fn member_in_project(&self, member: &Member) -> bool {
        for role in member.roles.iter() {
            if self.associated_roles.contains(&role) {
                return true;
            }
        }
        false
    }

    pub async fn add_role(&mut self, role: RoleId) {
        if !self.associated_roles.contains(&role) {
            self.associated_roles.push(role);
            self.update().await;

            Logger::high(
                "project.add_role",
                &format!(
                    "added associated role {} to project \"{}\"",
                    role.get(),
                    self.name(),
                ),
            )
            .await;
        }
    }

    pub async fn remove_role(&mut self, role: RoleId) {
        if self.associated_roles.contains(&role) {
            self.associated_roles.remove(
                match self.associated_roles.iter().position(|x| x == &role) {
                    Some(index) => index,
                    None => {
                        return ();
                    }
                },
            );

            self.update().await;

            Logger::high(
                "project.remove_role",
                &format!(
                    "removed associated role {} from project \"{}\"",
                    role.get(),
                    self.name(),
                ),
            )
            .await;
        }
    }

    async fn update_stat_post(&mut self, ctx: &Context) {
        if let Some(stat_channel) = self.stat_channel {
            let stat_channel = match fetch_channel(&ctx, stat_channel) {
                Ok(channel) => channel,
                Err(e) => {
                    Logger::error(
                        "project.update_stat_post",
                        &format!(
                            "cannot fetch stat channel {} of project \"{}\": {}",
                            stat_channel.get(),
                            self.name(),
                            e
                        ),
                    )
                    .await;
                    return;
                }
            };

            let embeds = self.get_stat_embeds(&ctx).await;

            for (role, embed) in embeds.iter() {
                match self.stat_posts.get(&role) {
                    Some(msg) => match stat_channel.message(&ctx.http, msg).await {
                        Ok(mut post) => {
                            post.edit(&ctx.http, EditMessage::new().embed(embed.clone()))
                                .await
                                .unwrap();
                        }
                        Err(_) => {
                            let stat_msg = stat_channel
                                .send_message(&ctx.http, CreateMessage::new().embed(embed.clone()))
                                .await
                                .unwrap();

                            self.stat_posts.insert(role.clone(), stat_msg.id);
                            self.write().await;
                        }
                    },
                    None => {
                        let stat_msg = stat_channel
                            .send_message(&ctx.http, CreateMessage::new().embed(embed.clone()))
                            .await
                            .unwrap();

                        self.stat_posts.insert(role.clone(), stat_msg.id);
                        self.write().await;
                    }
                }
            }

            for (role, msg) in self.stat_posts.clone().iter() {
                if !self.associated_roles.contains(&role) {
                    if let Ok(message) = stat_channel.message(&ctx.http, msg).await {
                        message.delete(&ctx.http).await.unwrap();
                    }
                    self.stat_posts.remove(role);
                }
            }

            Logger::debug(&format!("projects.{}", self.name), "updated stat post").await;
        }
    }

    async fn get_stat_embeds(&self, ctx: &Context) -> HashMap<RoleId, CreateEmbed> {
        let mut fields = HashMap::new();
        let mut mem_man = match MEMBERSMANAGER.try_write() {
            Ok(man) => man,
            Err(_) => {
                Logger::error(
                    "project.get_stat_embeds",
                    "error while try_write MEMBERSMANAGER, maybe deadlock, trying await...",
                )
                .await;
                MEMBERSMANAGER.write().await
            }
        };

        let guild = get_guild();
        let roles = guild.roles(&ctx.http).await.unwrap();

        for member in guild.members(&ctx.http, None, None).await.unwrap() {
            match get_highest_role_in(&ctx, member.user.id, &self.associated_roles).await {
                Ok(highest) => {
                    if let Some(role) = highest {
                        match mem_man
                            .get(member.user.id)
                            .await
                            .unwrap()
                            .to_project_stat(member.display_name().to_string(), &self.name)
                        {
                            Ok((name, value, inline)) => {
                                if !fields.contains_key(&role) {
                                    fields.insert(role, Vec::new());
                                }

                                fields.get_mut(&role).unwrap().push((name, value, inline));
                            }
                            Err(e) => {
                                Logger::error(
                                    "project.get_stat_embeds",
                                    &format!(
                                        "cannot get member {} stat post, {}",
                                        member.display_name(),
                                        e
                                    ),
                                )
                                .await;
                            }
                        }

                        break;
                    }
                }
                Err(e) => {
                    Logger::error(
                        "project.get_stat_embeds",
                        &format!(
                            "cannot get highest role of member {} ({}) in project \"{}\": {}",
                            member.display_name(),
                            member.user.id.get(),
                            self.name,
                            e
                        ),
                    )
                    .await
                }
            }
        }

        let mut embeds = HashMap::new();
        for role in self.associated_roles.iter() {
            embeds.insert(
                role.clone(),
                CreateEmbed::new()
                    .title(format!("{}", roles.get(role).unwrap().name))
                    .color(roles.get(role).unwrap().colour)
                    .fields(match fields.get(&role) {
                        Some(f) => f.clone(),
                        None => vec![(String::new(), String::new(), false)],
                    }),
            );
        }
        embeds
    }

    pub async fn to_embed(&self) -> CreateEmbed {
        let mut embed = CreateEmbed::new()
            .title(get_string(
                "project-embed-title",
                Some(HashMap::from([("project", self.name().as_str())])),
            ))
            .colour(Colour::MAGENTA);

        embed = embed.field(
            get_string("project-embed-max-tasks-per-user-name", None),
            self.max_tasks_per_user.to_string(),
            false,
        );

        if let Some(role) = &self.waiter_role {
            embed = embed.field(
                get_string("project-embed-waiter-role-name", None),
                &format!("<@&{}>", role.get()),
                false,
            );
        }

        embed = embed.field(
            get_string("project-embed-task-forum-name", None),
            &format!("<#{}>", self.tasks_forum.get()),
            false,
        );

        if let Some(channel) = &self.stat_channel {
            embed = embed.field(
                get_string("project-embed-stat-channel-name", None),
                &format!("<#{}>", channel.get()),
                false,
            );
        }

        if !self.associated_roles.is_empty() {
            embed = embed.field(
                get_string(
                    "project-embed-associated-roles-name",
                    Some(HashMap::from([(
                        "num",
                        self.associated_roles.len().to_string().as_str(),
                    )])),
                ),
                {
                    let mut value = String::new();
                    for role in self.associated_roles.iter() {
                        value = format!(
                            "{}{} <@&{}>\n",
                            value,
                            match role == self.associated_roles.last().unwrap() {
                                false => "╠︎",
                                true => "╚",
                            },
                            role.get()
                        );
                    }

                    value
                },
                false,
            );
        }

        embed
    }
}
