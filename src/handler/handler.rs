use crate::{command_manager::COMMANDMANAGER, config::CONFIG, logger::Logger};
use serenity::{
    all::async_trait,
    client::{Context, EventHandler},
    model::{application::Interaction, gateway::Ready},
};
use std::sync::Arc;

pub struct Handler;

#[async_trait]
impl EventHandler for Handler {
    #[allow(unused_variables)]
    async fn ready(&self, ctx: Context, data_about_bot: Ready) {
        let cfg = CONFIG.lock().await;

        if let Some(num) = cfg.log {
            Logger::set_log_channel(&ctx, num).await;

            Logger::debug(&ctx, "handler.ready", "the log channel is set").await;
        }

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
