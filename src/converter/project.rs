use std::collections::HashMap;

use crate::model::project::Project;
use serde::Deserialize;
use serenity::{
    self,
    all::{ChannelId, ForumTagId, MessageId, RoleId},
};

use super::{tag::OldTaskTag, task::OldTask};

#[derive(Deserialize)]
pub struct OldProject {
    pub name: Option<String>,
    #[serde(rename = "max_brigades_per_user")]
    max_tasks_per_user: u32,
    #[serde(rename = "forum")]
    pub tasks_forum: ChannelId,
    waiter_role: Option<RoleId>,
    #[serde(rename = "stat_post")]
    stat_posts: HashMap<RoleId, MessageId>,
    stat_channel: Option<ChannelId>,
    associated_roles: Vec<RoleId>,
    pub tags: HashMap<ForumTagId, OldTaskTag>,
    pub tasks: HashMap<u32, OldTask>,
}

impl Into<Project> for OldProject {
    fn into(self) -> Project {
        Project {
            name: self.name.unwrap(),
            max_tasks_per_user: self.max_tasks_per_user,
            tasks_forum: self.tasks_forum,
            waiter_role: self.waiter_role,
            stat_posts: self.stat_posts,
            stat_channel: self.stat_channel,
            associated_roles: self.associated_roles,
        }
    }
}
