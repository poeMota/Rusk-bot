use crate::prelude::*;
use serenity::all::{CreateActionRow, CreateButton, CreateSelectMenu};

impl task::Task {
    pub async fn main_changer(&self) -> Vec<CreateActionRow> {
        let mut rows = get_params_buttons("task-changer", vec!["score", "max-members"]);

        if self.finished {
            rows = Vec::new();
        }

        if !self.finished {
            rows.insert(
                0,
                CreateActionRow::SelectMenu(
                    CreateSelectMenu::new(
                        "task-changer:members",
                        serenity::all::CreateSelectMenuKind::User {
                            default_users: Some(self.members.get().clone()),
                        },
                    )
                    .min_values(0)
                    .max_values(*self.max_members.get() as u8)
                    .placeholder(loc!("task-changer-members-placeholder")),
                ),
            );

            rows.insert(
                1,
                CreateActionRow::SelectMenu(
                    CreateSelectMenu::new(
                        "task-changer:mentor",
                        serenity::all::CreateSelectMenuKind::User {
                            default_users: match self.mentor_id.get() {
                                Some(mentor) => Some(Vec::from([mentor.clone()])),
                                None => None,
                            },
                        },
                    )
                    .min_values(0)
                    .placeholder(loc!("task-changer-mentor-placeholder")),
                ),
            );
        }

        if self.finished {
            rows.push(CreateActionRow::Buttons(Vec::from([CreateButton::new(
                "task-changer:open",
            )
            .label(loc!("task-changer-open-button"))
            .style(serenity::all::ButtonStyle::Success)])));
        } else {
            rows.push(CreateActionRow::Buttons(Vec::from([CreateButton::new(
                "task-changer:close",
            )
            .label(loc!("task-changer-close-button"))
            .style(serenity::all::ButtonStyle::Danger)])));
        }

        rows
    }
}
