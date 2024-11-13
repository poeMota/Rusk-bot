use crate::prelude::*;
use once_cell::sync::Lazy;
use serde::{Deserialize, Serialize};
use serde_json;
use serenity::model::{
    guild::Member,
    id::{ChannelId, MessageId, RoleId},
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

    pub fn new_project(
        &mut self,
        name: String,
        max_tasks_per_user: u32,
        tasks_forum: ChannelId,
        waiter_role: Option<RoleId>,
        stat_channel: Option<ChannelId>,
    ) {
        let project = Project {
            name,
            max_tasks_per_user,
            tasks_forum,
            waiter_role,
            stat_channel,
            stat_posts: HashMap::new(),
            associated_roles: Vec::new(),
        };

        project.update();
        self.projects.insert(project.name.clone(), project);
    }

    pub fn get(&self, name: String) -> Option<&Project> {
        self.projects.get(&name)
    }

    pub fn get_mut(&mut self, name: String) -> Option<&mut Project> {
        self.projects.get_mut(&name)
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

    fn serialize(&self) {
        write_file(
            &DATA_PATH.join(format!("databases/projects/{}", self.name())),
            serde_json::to_string(&self).unwrap(),
        );
    }

    fn update(&self) {
        self.serialize();
    }

    fn member_in_project(&self, member: Member) -> bool {
        for role in member.roles.iter() {
            if self.associated_roles.contains(&role) {
                return true;
            }
        }
        false
    }

    fn add_role(&mut self, role: RoleId) {
        if self.associated_roles.contains(&role) {
            self.associated_roles.push(role);
            self.update();
        }
    }

    fn remove_role(&mut self, role: RoleId) {
        if self.associated_roles.contains(&role) {
            self.associated_roles.remove(
                match self.associated_roles.iter().position(|x| x == &role) {
                    Some(index) => index,
                    None => {
                        return ();
                    }
                },
            );

            self.update();
        }
    }
}
