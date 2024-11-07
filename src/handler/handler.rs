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
        let guild_id = GuildId::new(CONFIG.read().await.guild);
        clear_guild_commands(&ctx.http, &guild_id).await;

        if let Some(num) = CONFIG.read().await.log {
            Logger::set_log_channel(&ctx, num).await;

            Logger::debug(&ctx, "handler.ready", "the log channel is set").await;
        }

        fun_commands(ctx.clone(), guild_id).await;

        Logger::debug(&ctx, "handler.ready", "bot is ready").await;
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
    if let Ok(commands) = http.get_guild_commands(guild_id.clone()).await {
        for command in commands {
            if let Err(why) = http
                .delete_guild_command(guild_id.clone(), command.id)
                .await
            {
                println!(
                    "Error while clear guild command {}: {:?}",
                    command.name, why
                );
            }
        }
    }
}
