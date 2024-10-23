use once_cell::sync::Lazy;
use serenity::{
    model::{self, application::CommandInteraction, id::GuildId},
    prelude::*,
};

use std::{
    collections::HashMap,
    future::Future,
    pin::Pin,
    sync::{Arc, Mutex},
};

pub static COMMANDMANAGER: Lazy<Arc<Mutex<CommandManager>>> =
    Lazy::new(|| Arc::new(Mutex::new(CommandManager::new())));

pub struct CommandManager {
    commands: HashMap<
        String,
        Box<
            dyn for<'a> Fn(GuildId, &'a Context) -> Pin<Box<dyn Future<Output = ()> + 'a>>
                + Send
                + Sync,
        >,
    >,
    commands_calls: HashMap<
        String,
        Box<
            dyn for<'a> Fn(CommandInteraction, Context) -> Pin<Box<dyn Future<Output = ()>>>
                + Send
                + Sync,
        >,
    >,
}

impl CommandManager {
    pub fn new() -> Self {
        CommandManager {
            commands: HashMap::new(),
            commands_calls: HashMap::new(),
        }
    }

    pub fn add_command(
        &mut self,
        name: &str,
        command_declaration: Box<
            dyn for<'a> Fn(
                    model::id::GuildId,
                    &'a Context,
                ) -> Pin<Box<dyn Future<Output = ()> + 'a>>
                + Send
                + Sync,
        >,
        command_call: Box<
            dyn for<'a> Fn(CommandInteraction, Context) -> Pin<Box<dyn Future<Output = ()>>>
                + Send
                + Sync,
        >,
    ) {
        self.commands.insert(name.to_string(), command_declaration);
        self.commands_calls.insert(name.to_string(), command_call);
    }

    pub async fn apply_commands(&self, guild: GuildId, ctx: &Context) {
        for (_, command) in self.commands.iter() {
            (command)(guild, ctx).await;
        }
    }

    pub async fn call_command(
        &self,
        command_name: &String,
        command: CommandInteraction,
        ctx: Context,
    ) {
        self.commands_calls.get(command_name).expect(stringify!(
            "Cannot found call converter for commad {}",
            command_name
        ))(command, ctx)
        .await;
    }
}
