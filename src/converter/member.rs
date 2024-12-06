use std::collections::HashMap;

use crate::{
    model::member::{NotesHistory, ProjectMember, TaskHistory},
    shop::ShopData,
};
use serde::Deserialize;
use serenity::{
    self,
    all::{Timestamp, UserId},
};

#[allow(dead_code)]
#[derive(Debug, Deserialize)]
pub struct OldProjectMember {
    pub id: Option<UserId>,
    #[serde(rename = "выполненные заказы")]
    done_tasks: Option<HashMap<String, Vec<String>>>,
    #[serde(rename = "курирование заказов")]
    mentor_tasks: Option<HashMap<String, Vec<String>>>,
    #[serde(rename = "личная папка")]
    own_folder: Option<String>,
    #[serde(rename = "очки")]
    score: Option<i64>,
    #[serde(rename = "очки за всё время")]
    all_time_score: Option<i64>,
    #[serde(rename = "@предупреждения")]
    warns: Option<Vec<String>>,
    #[serde(rename = "@заметки")]
    notes: Option<Vec<String>>,
    #[serde(rename = "@последняя активность")]
    last_activity: Option<HashMap<String, String>>,
    #[serde(rename = "@сикей")]
    ckey: Option<String>,
}

impl Into<ProjectMember> for OldProjectMember {
    fn into(self) -> ProjectMember {
        let mut done_tasks = HashMap::new();
        let mut mentor_tasks = HashMap::new();
        let mut last_activity = HashMap::new();
        let mut warns = Vec::new();
        let mut notes = Vec::new();

        for (proj, tasks) in self.done_tasks.unwrap_or(HashMap::new()).iter() {
            if !done_tasks.contains_key(proj) {
                done_tasks.insert(proj.clone(), Vec::new());
            }

            for task in tasks {
                done_tasks
                    .get_mut(proj)
                    .unwrap()
                    .push(TaskHistory::OldFormat(task.clone()));
            }
        }

        for (proj, tasks) in self.mentor_tasks.unwrap_or(HashMap::new()).iter() {
            if !mentor_tasks.contains_key(proj) {
                mentor_tasks.insert(proj.clone(), Vec::new());
            }

            for task in tasks {
                mentor_tasks
                    .get_mut(proj)
                    .unwrap()
                    .push(TaskHistory::OldFormat(task.clone()));
            }
        }

        for (proj, time) in self.last_activity.unwrap_or(HashMap::new()).iter() {
            if let Ok(stamp) = Timestamp::parse(&format!("{}T00:00:00Z", time)) {
                last_activity.insert(proj.clone(), stamp);
            }
        }

        for note in self.notes.unwrap_or(Vec::new()).iter() {
            let user = UserId::new(
                note.split(" ")
                    .collect::<Vec<&str>>()
                    .get(0)
                    .unwrap()
                    .strip_prefix("<@")
                    .unwrap()
                    .strip_suffix(">")
                    .unwrap()
                    .parse::<u64>()
                    .unwrap(),
            );

            let timestamp = Timestamp::parse(&format!(
                "{}T00:00:00Z",
                note.split("): **")
                    .collect::<Vec<&str>>()
                    .get(0)
                    .unwrap()
                    .split("> (")
                    .collect::<Vec<&str>>()
                    .last()
                    .unwrap()
            ))
            .unwrap();

            let text = note
                .split("): **")
                .collect::<Vec<&str>>()
                .last()
                .unwrap()
                .strip_suffix("**")
                .unwrap()
                .to_string();

            notes.push(NotesHistory::Current((user, timestamp, text)));
        }

        for warn in self.warns.unwrap_or(Vec::new()).iter() {
            let user = UserId::new(
                warn.split(" ")
                    .collect::<Vec<&str>>()
                    .get(0)
                    .unwrap()
                    .strip_prefix("<@")
                    .unwrap()
                    .strip_suffix(">")
                    .unwrap()
                    .parse::<u64>()
                    .unwrap(),
            );

            let timestamp = Timestamp::parse(&format!(
                "{}T00:00:00Z",
                warn.split("): **П.")
                    .collect::<Vec<&str>>()
                    .get(0)
                    .unwrap()
                    .split("> (")
                    .collect::<Vec<&str>>()
                    .last()
                    .unwrap()
            ))
            .unwrap();

            let text = warn
                .split("): **")
                .collect::<Vec<&str>>()
                .last()
                .unwrap()
                .strip_suffix("**")
                .unwrap()
                .to_string();

            warns.push(NotesHistory::Current((user, timestamp, text)));
        }

        ProjectMember {
            id: self.id.unwrap(),
            in_tasks: HashMap::new(),
            done_tasks,
            mentor_tasks,
            own_folder: self.own_folder,
            score: self.score.unwrap_or(0),
            all_time_score: self.all_time_score.unwrap_or(0),
            last_activity,
            warns,
            notes,
            shop_data: ShopData {
                current_page: 0,
                pages: Vec::new(),
            },
            changed_member: None,
            changed_task: None,
            changed_project: None,
            changed_tag: None,
            changed_sub_post: None,
        }
    }
}
