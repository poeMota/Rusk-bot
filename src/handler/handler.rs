use crate::command_manager::COMMANDMANAGER;
use serenity::{
    all::async_trait,
    client::{Context, EventHandler},
    model::application::Interaction,
};
use std::sync::Arc;

pub struct Handler;

#[async_trait]
impl EventHandler for Handler {
    async fn interaction_create(&self, ctx: Context, interaction: Interaction) {
        if let Interaction::Command(ref command) = interaction {
            let command_man = COMMANDMANAGER.read().await;
            command_man
                .call_command(&command.data.name, command.clone(), Arc::new(ctx))
                .await;
        }
    }
}
