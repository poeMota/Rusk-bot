use crate::prelude::*;
use project::project::Project;
use serenity::{
    self,
    all::{ChannelType, CreateActionRow, CreateSelectMenu},
};

impl Project {
    pub async fn main_changer(&self) -> Vec<CreateActionRow> {
        let mut rows = get_params_buttons("project-changer", Vec::from(["max-tasks-per-user"]));

        rows.push(CreateActionRow::SelectMenu(
            CreateSelectMenu::new(
                "project-changer:tasks-forum",
                serenity::all::CreateSelectMenuKind::Channel {
                    channel_types: Some(Vec::from([ChannelType::Forum])),
                    default_channels: None,
                },
            )
            .placeholder(get_string("project-changer-tasks-forum-placeholder", None))
            .min_values(0)
            .max_values(1),
        ));

        rows.push(CreateActionRow::SelectMenu(
            CreateSelectMenu::new(
                "project-changer:waiter-role",
                serenity::all::CreateSelectMenuKind::Role {
                    default_roles: match self.waiter_role {
                        Some(role) => Some(Vec::from([role])),
                        None => None,
                    },
                },
            )
            .placeholder(get_string("project-changer-waiter-role-placeholder", None))
            .min_values(0)
            .max_values(1),
        ));

        rows.push(CreateActionRow::SelectMenu(
            CreateSelectMenu::new(
                "project-changer:stat-channel",
                serenity::all::CreateSelectMenuKind::Channel {
                    channel_types: Some(Vec::from([ChannelType::Text])),
                    default_channels: match self.stat_channel {
                        Some(channel) => Some(Vec::from([channel])),
                        None => None,
                    },
                },
            )
            .placeholder(get_string("project-changer-stat-channel-placeholder", None))
            .min_values(0)
            .max_values(1),
        ));

        rows.push(CreateActionRow::SelectMenu(
            CreateSelectMenu::new(
                "project-changer:associated-roles",
                serenity::all::CreateSelectMenuKind::Role {
                    default_roles: Some(self.associated_roles.clone()),
                },
            )
            .placeholder(get_string(
                "project-changer-associated-roles-placeholder",
                None,
            ))
            .min_values(0)
            .max_values(20),
        ));

        rows
    }
}