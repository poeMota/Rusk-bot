use crate::prelude::*;
use serenity::all::{CreateActionRow, CreateButton, CreateSelectMenu};

impl task::Task {
    pub async fn main_changer(&self) -> Vec<CreateActionRow> {
        let mut rows = get_params_buttons("task-changer", vec!["score", "max-members", "mentor"]);

        rows.insert(
            0,
            CreateActionRow::SelectMenu(
                CreateSelectMenu::new(
                    "task-changer:members",
                    serenity::all::CreateSelectMenuKind::User {
                        default_users: Some(self.members.get().clone()),
                    },
                )
                .max_values(*self.max_members.get() as u8)
                .placeholder(get_string("task-changer-members-placeholder", None)),
            ),
        );

        if self.finished {
            rows.push(CreateActionRow::Buttons(Vec::from([CreateButton::new(
                "task-changer:open",
            )
            .label(get_string("task-changer-open-button", None))
            .style(serenity::all::ButtonStyle::Success)])));
        } else {
            rows.push(CreateActionRow::Buttons(Vec::from([CreateButton::new(
                "task-changer:close",
            )
            .label(get_string("task-changer-close-button", None))
            .style(serenity::all::ButtonStyle::Danger)])));
        }

        rows
    }
}