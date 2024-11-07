use crate::logger::Logger;
use once_cell::sync::Lazy;
use serenity::{model::application::CommandInteraction, prelude::*};
use std::{collections::HashMap, future::Future, pin::Pin, sync::Arc};
use tokio::sync::RwLock;

pub static COMMANDMANAGER: Lazy<Arc<RwLock<CommandManager>>> =
    Lazy::new(|| Arc::new(RwLock::new(CommandManager::new())));

pub struct CommandManager {
    commands_calls: HashMap<
        String,
        Arc<
            dyn Fn(
                    CommandInteraction,
                    Arc<Context>,
                ) -> Pin<
                    Box<
                        dyn Future<Output = Result<(), Box<dyn std::error::Error + Send + Sync>>>
                            + Send,
                    >,
                > + Send
                + Sync,
        >,
    >,
}

impl CommandManager {
    pub fn new() -> Self {
        CommandManager {
            commands_calls: HashMap::new(),
        }
    }

    pub fn add_command(
        &mut self,
        name: &str,
        command_call: Arc<
            dyn Fn(
                    CommandInteraction,
                    Arc<Context>,
                ) -> Pin<
                    Box<
                        dyn Future<Output = Result<(), Box<dyn std::error::Error + Send + Sync>>>
                            + Send,
                    >,
                > + Send
                + Sync,
        >,
    ) {
        self.commands_calls.insert(name.to_string(), command_call);
    }

    pub async fn call_command(
        &self,
        command_name: &str,
        command: CommandInteraction,
        ctx: Arc<Context>,
    ) {
        if let Some(command_fn) = self.commands_calls.get(command_name) {
            match command_fn(command.clone(), Arc::clone(&ctx)).await {
                Ok(_) => {
                    let member = command.member.clone();
                    let mut display_name = "Unknown".to_string();
                    let mut id = 0;

                    if let Some(mem) = member {
                        display_name = String::from(mem.display_name());
                        id = mem.user.id.get();
                    }

                    Logger::debug(
                        &format!("commands.{}", command_name),
                        &format!("command triggered by {} ({})", display_name, id),
                    )
                    .await;
                }
                Err(e) => {
                    Logger::error(
                        &format!("commands.{}", command_name),
                        &format!("command caused a panic with error: {}", e.to_string()),
                    )
                    .await
                }
            };
        } else {
            Logger::error(
                "com_man.call_command",
                &format!(
                    "command manager cannot find call for command: {}",
                    command_name
                ),
            )
            .await;
        }
    }
}
