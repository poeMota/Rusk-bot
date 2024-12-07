use crate::prelude::*;
use once_cell::sync::Lazy;
use serde::{Deserialize, Serialize};
use serde_json;
use serenity::{
    all::{Colour, CreateEmbed},
    model::id::{ChannelId, ForumTagId, RoleId},
};
use std::fs;
use std::{collections::HashMap, sync::Arc};
use tokio::sync::RwLock;
use walkdir::WalkDir;

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Copy)]
pub enum TageTypes {
    Base,
    ClosedTask,
    InWork,
}

impl ToString for TageTypes {
    fn to_string(&self) -> String {
        match self {
            Self::Base => String::from("base"),
            Self::ClosedTask => String::from("closedtask"),
            Self::InWork => String::from("inwork"),
        }
    }
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
        if !fs::exists(DATA_PATH.join("databases/tags")).unwrap() {
            fs::create_dir(DATA_PATH.join("databases/tags"))
                .expect("error while creating folder data/databases/tags");
        }

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

    pub fn get_by_type(
        &self,
        forum_id: &ChannelId,
        tag_type: TageTypes,
    ) -> Option<Vec<ForumTagId>> {
        let mut tags = Vec::new();
        for tag_id in self.tags_by_channel.get(&forum_id)? {
            if let Some(tag) = self.tags.get(tag_id) {
                if tag.tag_type == Some(tag_type) {
                    tags.push(tag.id);
                }
            }
        }
        Some(tags)
    }

    pub async fn new_tag(&mut self, tag_id: ForumTagId, forum_id: ChannelId) {
        let tag = TaskTag::new(tag_id, forum_id);
        self.tags.insert(tag.id, tag.clone());

        if !self.tags_by_channel.contains_key(&tag.forum_id) {
            self.tags_by_channel
                .insert(tag.forum_id.clone(), Vec::new());
        }
        self.tags_by_channel
            .get_mut(&tag.forum_id)
            .unwrap()
            .push(tag.id);

        tag.update().await;

        Logger::low(
            "tag_man.new_tag",
            &format!("registered new tag {} from channel {}", tag_id, forum_id),
        )
        .await;
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
    pub fn new(id: ForumTagId, forum_id: ChannelId) -> Self {
        Self {
            id,
            forum_id,
            tag_type: None,
            max_members: None,
            score_modifier: None,
            task_project: None,
            ping_role: None,
        }
    }

    async fn serialize(&self) {
        write_file(
            &DATA_PATH.join(format!("databases/tags/{}", self.id)),
            match serde_json::to_string(&self) {
                Ok(content) => content,
                Err(e) => {
                    Logger::error(
                        "tag.serialize",
                        &format!("cannot serialize tag {}: {}", self.id.get(), e.to_string()),
                    )
                    .await;
                    return;
                }
            },
        );
    }

    pub async fn update(&self) {
        self.serialize().await;
    }

    pub async fn set_tag_type(&mut self, tag_type: Option<TageTypes>) {
        let old = self.tag_type;

        self.tag_type = tag_type;
        self.update().await;

        Logger::medium(
            "tag.set_tag_type",
            &format!(
                "type of tag {} changed from {:?} to {:?}",
                self.id.get(),
                old,
                self.tag_type
            ),
        )
        .await;
    }

    pub async fn set_max_members(&mut self, max_members: Option<u32>) {
        let old = self.max_members;

        self.max_members = max_members;
        self.update().await;

        Logger::medium(
            "tag.set_max_members",
            &format!(
                "max members of tag {} changed from {:?} to {:?}",
                self.id.get(),
                old,
                self.max_members
            ),
        )
        .await;
    }

    pub async fn set_score_modifier(&mut self, score_modifier: Option<i64>) {
        let old = self.score_modifier;

        self.score_modifier = score_modifier;
        self.update().await;

        Logger::medium(
            "tag.set_score_modifier",
            &format!(
                "score modifier of tag {} changed from {:?} to {:?}",
                self.id.get(),
                old,
                self.score_modifier
            ),
        )
        .await;
    }

    pub async fn set_task_project(&mut self, task_project: Option<String>) {
        let old = self.task_project.clone();

        self.task_project = task_project;
        self.update().await;

        Logger::medium(
            "tag.set_task_project",
            &format!(
                "task project of tag {} changed from {:?} to {:?}",
                self.id.get(),
                old,
                self.task_project
            ),
        )
        .await;
    }

    pub async fn set_ping_role(&mut self, ping_role: Option<RoleId>) {
        let old = self.ping_role;

        self.ping_role = ping_role;
        self.update().await;

        Logger::medium(
            "tag.set_ping_role",
            &format!(
                "ping role of tag {} changed from {:?} to {:?}",
                self.id.get(),
                old,
                self.ping_role
            ),
        )
        .await;
    }

    pub fn to_embed(&self) -> CreateEmbed {
        let mut embed = CreateEmbed::new()
            .colour(Colour::DARK_GREY)
            .title(get_string("tag-embed-title", None))
            .field(
                get_string("tag-embed-id-name", None),
                format!("`{}`", self.id.get()),
                false,
            )
            .field(
                get_string("tag-embed-forum-id-name", None),
                format!("<#{}>", self.forum_id.get()),
                false,
            );

        if let Some(tag_type) = self.tag_type {
            embed = embed.field(
                get_string("tag-embed-tag-type-name", None),
                get_string(&format!("tag-types-{}", tag_type.to_string()), None),
                false,
            );
        }

        if let Some(max_members) = self.max_members {
            embed = embed.field(
                get_string("tag-embed-max-members-name", None),
                format!("`{}`", max_members),
                false,
            );
        }

        if let Some(score_modifier) = self.score_modifier {
            embed = embed.field(
                get_string("tag-embed-score-modifier-name", None),
                format!("`{}`", score_modifier),
                false,
            );
        }

        if let Some(task_project) = &self.task_project {
            embed = embed.field(
                get_string("tag-embed-task-project-name", None),
                format!("`{}`", task_project),
                false,
            );
        }

        if let Some(ping_role) = self.ping_role {
            embed = embed.field(
                get_string("tag-embed-ping-role-name", None),
                format!("<@&{}>", ping_role),
                false,
            );
        }

        embed
    }
}
