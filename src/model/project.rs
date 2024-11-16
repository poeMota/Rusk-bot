use crate::prelude::*;
use once_cell::sync::Lazy;
use serde::{Deserialize, Serialize};
use serde_json;
use serenity::{
    builder::{CreateEmbed, EditMessage},
    model::{
        guild::Member,
        id::{ChannelId, MessageId, RoleId},
    },
};
use std::{collections::HashMap, sync::Arc};
use tokio::sync::RwLock;
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
        ctx: &Context,
        name: String,
        max_tasks_per_user: u32,
        tasks_forum: ChannelId,
        waiter_role: Option<RoleId>,
        stat_channel: Option<ChannelId>,
    ) -> Result<(), String> {
        if !self.projects.contains_key(&name) {
            let mut project = Project {
                name,
                max_tasks_per_user,
                tasks_forum,
                waiter_role,
                stat_channel,
                stat_posts: HashMap::new(),
                associated_roles: Vec::new(),
            };

            project.update(&ctx).await;
            self.projects.insert(project.name.clone(), project);
        } else {
            return Err(format!("project with name \"{}\" currently excist", name));
        }

        Ok(())
    }

    pub fn get(&self, name: &String) -> Option<&Project> {
        self.projects.get(name)
    }

    pub fn get_mut(&mut self, name: &String) -> Option<&mut Project> {
        self.projects.get_mut(name)
    }

    pub async fn update_from_member(&mut self, ctx: &Context, member: &Member) {
        for proj in self.projects.values_mut() {
            if proj.member_in_project(&member) {
                proj.update_stat_post(&ctx).await;
            }
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Project {
    name: String,
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

    async fn update(&mut self, ctx: &Context) {
        self.write().await;
        self.update_stat_post(&ctx).await;
    }

    pub fn member_in_project(&self, member: &Member) -> bool {
        for role in member.roles.iter() {
            if self.associated_roles.contains(&role) {
                return true;
            }
        }
        false
    }

    pub async fn add_role(&mut self, ctx: &Context, role: RoleId) {
        if !self.associated_roles.contains(&role) {
            self.associated_roles.push(role);
            self.update(&ctx).await;
        }
    }

    pub async fn remove_role(&mut self, ctx: &Context, role: RoleId) {
        if self.associated_roles.contains(&role) {
            self.associated_roles.remove(
                match self.associated_roles.iter().position(|x| x == &role) {
                    Some(index) => index,
                    None => {
                        return ();
                    }
                },
            );

            self.update(&ctx).await;
        }
    }

    async fn update_stat_post(&mut self, ctx: &Context) {
        if let Some(stat_channel) = self.stat_channel {
            let stat_channel = fetch_channel(&ctx, stat_channel).unwrap();
            let embeds = self.get_stat_embeds(&ctx).await;

            for (role, embed) in embeds.iter() {
                match self.stat_posts.get(&role) {
                    Some(msg) => {
                        let mut post = stat_channel.message(&ctx.http, msg).await.unwrap();

                        post.edit(&ctx.http, EditMessage::new().embed(embed.clone()))
                            .await
                            .unwrap();
                    }
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
            for role in member.roles.iter().rev() {
                if self.associated_roles.contains(&role) {
                    if !fields.contains_key(role) {
                        fields.insert(role.clone(), Vec::new());
                    }

                    match mem_man
                        .get(member.user.id)
                        .await
                        .unwrap()
                        .to_project_stat(member.display_name().to_string(), &self.name)
                    {
                        Ok((name, value, inline)) => {
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
}
