use crate::prelude::*;
use serenity::{
    self,
    all::{CreateActionRow, CreateSelectMenu, CreateSelectMenuOption},
};
use tag::{tag::TaskTag, TageTypes};

impl TaskTag {
    pub async fn main_changer(&self) -> Vec<CreateActionRow> {
        let mut rows =
            get_params_buttons("tag-changer", Vec::from(["max-members", "score-modifier"]));

        rows.insert(
            0,
            CreateActionRow::SelectMenu(
                CreateSelectMenu::new(
                    "tag-changer:tag-type",
                    serenity::all::CreateSelectMenuKind::String {
                        options: Vec::from([
                            CreateSelectMenuOption::new(loc!("tag-types-base"), "base")
                                .default_selection(self.tag_type == Some(TageTypes::Base)),
                            CreateSelectMenuOption::new(loc!("tag-types-closedtask"), "closedtask")
                                .default_selection(self.tag_type == Some(TageTypes::ClosedTask)),
                            CreateSelectMenuOption::new(loc!("tag-types-inwork"), "inwork")
                                .default_selection(self.tag_type == Some(TageTypes::InWork)),
                        ]),
                    },
                )
                .placeholder(loc!("tag-changer-tag-type-placeholder")),
            ),
        );

        rows.insert(
            1,
            CreateActionRow::SelectMenu(
                CreateSelectMenu::new(
                    "tag-changer:ping-role",
                    serenity::all::CreateSelectMenuKind::Role {
                        default_roles: match self.ping_role {
                            Some(role) => Some(Vec::from([role])),
                            None => None,
                        },
                    },
                )
                .min_values(0)
                .max_values(1)
                .placeholder(loc!("tag-changer-ping-role-placeholder")),
            ),
        );

        let proj_man = project::PROJECTMANAGER.write().await;
        let projects = proj_man.projects();

        if !projects.is_empty() {
            let mut options = Vec::new();
            for project in projects {
                options.push(
                    CreateSelectMenuOption::new(project.clone(), project)
                        .default_selection(Some(project.clone()) == self.task_project),
                );
            }

            rows.insert(
                2,
                CreateActionRow::SelectMenu(
                    CreateSelectMenu::new(
                        "tag-changer:task-project",
                        serenity::all::CreateSelectMenuKind::String { options },
                    )
                    .min_values(0)
                    .max_values(1)
                    .placeholder(loc!("tag-changer-task-project-placeholder")),
                ),
            );
        }

        rows
    }
}
