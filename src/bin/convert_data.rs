use converter::{OldProject, OldProjectMember};
use member::ProjectMember;
use project::Project;
use serenity::all::UserId;
use std::{collections::HashMap, fs};
use tag::TaskTag;
use task::Task;
use task_bot::prelude::*;
use tokio;

#[tokio::main]
async fn main() {
    let mut task_members: HashMap<UserId, HashMap<String, Vec<u32>>> = HashMap::new();

    if fs::exists(DATA_PATH.join("projects.json")).unwrap() {
        let content = read_file(&DATA_PATH.join("projects.json"));
        let projects: HashMap<String, OldProject> = match serde_json::from_str(&content) {
            Ok(m) => m,
            Err(e) => panic!("error while converting projects.json: {}", e),
        };

        for (name, mut project) in projects {
            project.name = Some(name);

            for (id, mut tag) in project.tags.clone() {
                tag.id = Some(id);
                tag.forum_id = Some(project.tasks_forum.clone());

                let new_tag: TaskTag = tag.into();
                new_tag.update();
            }

            for (id, mut task) in project.tasks.clone() {
                task.id = Some(id);
                task.project = project.name.clone();

                for member in task.members.clone() {
                    if !task_members.contains_key(&member) {
                        task_members.insert(member.clone(), HashMap::new());
                    }

                    let in_tasks = task_members.get_mut(&member).unwrap();

                    if !in_tasks.contains_key(&task.project.clone().unwrap()) {
                        in_tasks.insert(task.project.clone().unwrap(), Vec::new());
                    }

                    in_tasks
                        .get_mut(&task.project.clone().unwrap())
                        .unwrap()
                        .push(id);
                }

                let new_task: Task = task.into();
                new_task.update();
            }

            let new_project: Project = project.into();
            new_project.update().await;
        }

        println!("Converted projects.json");
    }

    if fs::exists(DATA_PATH.join("members_database.json")).unwrap() {
        let content = read_file(&DATA_PATH.join("members_database.json"));
        let members: HashMap<UserId, OldProjectMember> = match serde_json::from_str(&content) {
            Ok(m) => m,
            Err(e) => panic!("error while converting members_database.json: {}", e),
        };

        for (user, mut old_member) in members {
            old_member.id = Some(user);

            let mut new_member: ProjectMember = old_member.into();
            new_member.in_tasks = task_members
                .get(&new_member.id)
                .unwrap_or(&HashMap::new())
                .clone();

            new_member.update();
        }

        println!("Converted members_database.json");
    }

    println!("Databases converted successfully!");
}
