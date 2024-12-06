use crate::model::tag::{TageTypes, TaskTag};
use serde::Deserialize;
use serenity::{
    self,
    all::{ChannelId, ForumTagId, RoleId},
};

#[derive(Deserialize, Clone)]
pub struct OldTaskTag {
    pub id: Option<ForumTagId>,
    pub forum_id: Option<ChannelId>,
    #[serde(rename = "type")]
    tag_type: String,
    max_members: Option<u32>,
    score_modifier: Option<i64>,
    ping_role: Option<RoleId>,
}

impl Into<TaskTag> for OldTaskTag {
    fn into(self) -> TaskTag {
        TaskTag {
            id: self.id.unwrap(),
            forum_id: self.forum_id.unwrap(),
            tag_type: match self.tag_type.as_str() {
                "ended_tag" => Some(TageTypes::ClosedTask),
                "in_work_tag" => Some(TageTypes::InWork),
                _ => Some(TageTypes::Base),
            },
            max_members: self.max_members,
            score_modifier: self.score_modifier,
            task_project: None,
            ping_role: self.ping_role,
        }
    }
}
