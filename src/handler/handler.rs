use crate::{command_manager::COMMANDMANAGER, commands::*, config::CONFIG, logger::Logger};
use serenity::{
    all::async_trait,
    client::{Context, EventHandler},
    http::Http,
    model::{application::Interaction, gateway::Ready, id::GuildId},
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
        clear_guild_commands(&ctx.http, &guild_id).await;

        fun_commands(ctx.clone(), guild_id).await;

        Logger::debug("handler.ready", "bot is ready").await;
    }

    async fn interaction_create(&self, ctx: Context, interaction: Interaction) {
        if let Interaction::Command(ref command) = interaction {
            let command_man = COMMANDMANAGER.read().await;
            command_man
                .call_command(&command.data.name, command.clone(), Arc::new(ctx))
                .await;
        }
    }
}

async fn clear_guild_commands(http: &Http, guild_id: &GuildId) {
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
