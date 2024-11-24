use crate::{
    commands::*, config::CONFIG, model::project::PROJECTMANAGER, prelude::*,
    shop::shop_component_listeners,
};
use serenity::{
    all::async_trait,
    client::{Context, EventHandler},
    http::Http,
    model::{application::Interaction, event::GuildMemberUpdateEvent, gateway::Ready, id::GuildId},
};
use std::sync::Arc;

pub struct Handler;

#[async_trait]
impl EventHandler for Handler {
    #[allow(unused_variables)]
    async fn ready(&self, ctx: Context, data_about_bot: Ready) {
        if let Some(num) = CONFIG.try_read().unwrap().log {
            Logger::set_log_channel(&ctx, num).await;
        }

        let guild_id = GuildId::new(CONFIG.try_read().unwrap().guild);

        // TODO: better commands sync
        //_clear_guild_commands(&ctx.http, &guild_id).await;

        fun_commands(&ctx, guild_id).await;
        debug_commands(&ctx, guild_id).await;
        shop_commands(&ctx, guild_id).await;
        member_commands(&ctx, guild_id).await;
        project_commands(&ctx, guild_id).await;

        shop_component_listeners().await;

        Logger::debug("handler.ready", "bot is ready").await;
    }

    async fn interaction_create(&self, ctx: Context, interaction: Interaction) {
        let command_man = match COMMANDMANAGER.try_read() {
            Ok(man) => man,
            Err(_) => {
                Logger::error(
                    "handler.interaction_create",
                    "error while try_read COMMANDMANAGER, maybe deadlock, trying await...",
                )
                .await;
                COMMANDMANAGER.read().await
            }
        };

        match interaction {
            Interaction::Command(ref command) => {
                command_man
                    .call_command(&command.data.name, command, Arc::new(ctx))
                    .await;
            }
            Interaction::Component(ref component) => {
                command_man
                    .call_component(&component.data.custom_id, component, Arc::new(ctx))
                    .await;
            }
            _ => (),
        }
    }

    #[allow(unused_variables)]
    async fn guild_member_update(
        &self,
        ctx: Context,
        old_if_available: Option<Member>,
        new: Option<Member>,
        event: GuildMemberUpdateEvent,
    ) {
        let mut roles_diff = Vec::new();

        if let Some(ref new) = new {
            if let Some(ref old) = old_if_available {
                for role in new.roles.iter() {
                    if !old.roles.contains(&role) {
                        roles_diff.push(role.clone());
                    }
                }

                for role in old.roles.iter() {
                    if !new.roles.contains(&role) {
                        roles_diff.push(role.clone());
                    }
                }
            } else {
                roles_diff = new.roles.clone();
            }
        }

        if !roles_diff.is_empty() {
            let mut proj_mem = match PROJECTMANAGER.try_write() {
                Ok(man) => man,
                Err(_) => {
                    Logger::error(
                        "handler.guild_member_update",
                        "error while try_write PROJECTMANAGER, maybe deadlock, trying await...",
                    )
                    .await;
                    PROJECTMANAGER.write().await
                }
            };

            proj_mem.update_from_roles(&ctx, &roles_diff).await;
        } else if let Some(_) = event.nick {
            if let Some(new) = new {
                let mut proj_mem = match PROJECTMANAGER.try_write() {
                    Ok(man) => man,
                    Err(_) => {
                        Logger::error(
                            "handler.guild_member_update",
                            "error while try_write PROJECTMANAGER, maybe deadlock, trying await...",
                        )
                        .await;
                        PROJECTMANAGER.write().await
                    }
                };

                proj_mem.update_from_member(&ctx, &new).await;
            }
        }
    }
}

async fn _clear_guild_commands(http: &Http, guild_id: &GuildId) {
    match http.get_guild_commands(guild_id.clone()).await {
        Ok(commands) => {
            for command in commands {
                if Logger::if_ok(
                    "handler.clear_guild_commands",
                    &format!("error while clear guild command {}", command.name),
                    http.delete_guild_command(guild_id.clone(), command.id)
                        .await,
                )
                .await
                {
                    Logger::debug(
                        "handler.clear_guild_commands",
                        &format!("deleted interaction command: {}", command.name),
                    )
                    .await;
                }
            }
        }
        Err(e) => {
            Logger::error(
                "handler.clear_guild_commands",
                &format!("error while getting guild commands: {}", e.to_string()),
            )
            .await;
        }
    }
}
