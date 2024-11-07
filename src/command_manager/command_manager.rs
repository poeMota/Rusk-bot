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
            command_fn(command, Arc::clone(&ctx)).await.unwrap();
        } else {
            println!("Cannot find command: {}", command_name);
        }
    }
}
