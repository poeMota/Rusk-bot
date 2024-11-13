use crate::prelude::*;
use once_cell::sync::Lazy;
use serde::{Deserialize, Serialize};
use serde_json;
use serenity::model::id::{ChannelId, ForumTagId, RoleId};
use std::{collections::HashMap, sync::Arc};
use tokio::sync::RwLock;
use walkdir::WalkDir;

#[derive(Debug, Clone, Deserialize, Serialize)]
pub enum TageTypes {
    Base,
    FrozenTask,
    EndedTask,
    ClosedTask,
}

pub static TAGSMANAGER: Lazy<Arc<RwLock<TagsManager>>> =
    Lazy::new(|| Arc::new(RwLock::new(TagsManager::new())));

#[derive(Debug, Deserialize)]
pub struct TagsManager {
    tags: HashMap<ForumTagId, TaskTag>,
    tags_by_channel: HashMap<ChannelId, Vec<ForumTagId>>,
}

impl TagsManager {
    fn new() -> Self {
        Self {
            tags: HashMap::new(),
            tags_by_channel: HashMap::new(),
        }
    }

    pub async fn init(&mut self) {
        for entry in WalkDir::new(DATA_PATH.join("databases/tags")) {
            let entry = match entry {
                Ok(s) => s,
                Err(error) => {
                    Logger::error(
                        "tags_man.init",
                        &format!("error with tag data file: {}", error),
                    )
                    .await;
                    continue;
                }
            };

            if !entry.path().is_file() {
                continue;
            }

            let tag: TaskTag =
                match serde_yaml::from_str(read_file(&entry.path().to_path_buf()).as_str()) {
                    Ok(c) => c,
                    Err(e) => {
                        Logger::error(
                            "tags_man.init",
                            &format!(
                                "error while parsing tag data file \"{}\": {}",
                                entry.file_name().to_str().unwrap(),
                                e.to_string()
                            ),
                        )
                        .await;
                        continue;
                    }
                };

            self.tags.insert(tag.id, tag.clone());

            if !self.tags_by_channel.contains_key(&tag.forum_id) {
                self.tags_by_channel
                    .insert(tag.forum_id.clone(), Vec::new());
            }
            self.tags_by_channel
                .get_mut(&tag.forum_id)
                .unwrap()
                .push(tag.id);
        }

        Logger::debug("tags_man.init", "initialized from databases/tags/*").await;
    }

    pub fn get(&self, id: &ForumTagId) -> Option<&TaskTag> {
        self.tags.get(&id)
    }

    pub fn get_mut(&mut self, id: &ForumTagId) -> Option<&mut TaskTag> {
        self.tags.get_mut(&id)
    }

    pub fn get_forum_tags(&self, forum_id: &ChannelId) -> Option<Vec<&TaskTag>> {
        let mut tags = Vec::new();
        for tag_id in self.tags_by_channel.get(&forum_id)? {
            tags.push(self.tags.get(tag_id)?);
        }
        Some(tags)
    }

    pub fn register_new_tag(&mut self, tag: TaskTag) {
        self.tags.insert(tag.id, tag.clone());

        if !self.tags_by_channel.contains_key(&tag.forum_id) {
            self.tags_by_channel
                .insert(tag.forum_id.clone(), Vec::new());
        }
        self.tags_by_channel
            .get_mut(&tag.forum_id)
            .unwrap()
            .push(tag.id);

        tag.update();
    }
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct TaskTag {
    pub id: ForumTagId,
    pub forum_id: ChannelId,
    pub tag_type: Option<TageTypes>,
    pub max_members: Option<u32>,
    pub score_modifier: Option<i64>,
    pub task_project: Option<String>,
    pub ping_role: Option<RoleId>,
}

impl TaskTag {
    fn serialize(&self) {
        write_file(
            &DATA_PATH.join(format!("databases/tags/{}", self.id)),
            serde_json::to_string(&self).unwrap(),
        );
    }

    fn update(&self) {
        self.serialize();
    }
}
