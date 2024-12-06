use std::collections::HashMap;

use crate::model::task::{Task, TaskOption};
use serde::Deserialize;
use serenity::{
    self,
    all::{ChannelId, Timestamp, UserId},
};

#[derive(Deserialize, Clone)]
pub struct OldTask {
    pub id: Option<u32>,
    pub project: Option<String>,
    #[serde(rename = "score_modifier")]
    score: i64,
    name: String,
    #[serde(rename = "thread")]
    thread_id: ChannelId,
    #[serde(rename = "brigadire")]
    mentor_id: Option<UserId>,
    pub members: Vec<UserId>,
    start_date: String,
    last_save: Option<String>,
    max_members: u32,
    _max_members: Option<u32>,
}

impl Into<Task> for OldTask {
    fn into(self) -> Task {
        Task {
            id: self.id.unwrap(),
            project: self.project.unwrap(),
            thread_id: self.thread_id,
            finished: false,
            name: TaskOption::new(self.name),
            score: TaskOption::new(self.score),
            max_members: match self._max_members {
                Some(max) => TaskOption::new(max),
                None => TaskOption::new(self.max_members),
            },
            mentor_id: TaskOption::new(self.mentor_id),
            members: TaskOption::new(self.members),
            start_date: Some(Timestamp::parse(&format!("{}T00:00:00Z", self.start_date)).unwrap()),
            end_date: TaskOption::new(None),
            last_save: TaskOption::new(self.last_save),
            ending_results: HashMap::new(),
        }
    }
}
