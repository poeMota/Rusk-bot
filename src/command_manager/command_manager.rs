use crate::logger::Logger;
use once_cell::sync::Lazy;
use serenity::{
    model::application::{CommandInteraction, ComponentInteraction, ModalInteraction},
    prelude::*,
};
use std::{collections::HashMap, future::Future, pin::Pin, sync::Arc};
use tokio::sync::RwLock;

pub static COMMANDMANAGER: Lazy<Arc<RwLock<CommandManager>>> =
    Lazy::new(|| Arc::new(RwLock::new(CommandManager::new())));

pub struct CommandManager {
    commands: HashMap<
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
    components: HashMap<
        String,
        Arc<
            dyn Fn(
                    ComponentInteraction,
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
    modals: HashMap<
        String,
        Arc<
            dyn Fn(
                    ModalInteraction,
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
            commands: HashMap::new(),
            components: HashMap::new(),
            modals: HashMap::new(),
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
        self.commands.insert(name.to_string(), command_call);
    }

    pub fn add_modal(
        &mut self,
        modal_id: &str,
        modal_call: Arc<
            dyn Fn(
                    ModalInteraction,
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
        self.modals.insert(modal_id.to_string(), modal_call);
    }

    pub fn add_component(
        &mut self,
        comp_id: &str,
        component_call: Arc<
            dyn Fn(
                    ComponentInteraction,
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
        self.components.insert(comp_id.to_string(), component_call);
    }

    pub async fn call_command(
        &self,
        command_name: &str,
        command: &CommandInteraction,
        ctx: Arc<Context>,
    ) {
        if let Some(command_fn) = self.commands.get(command_name) {
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

    pub async fn call_component(
        &self,
        component_id: &str,
        component: &ComponentInteraction,
        ctx: Arc<Context>,
    ) {
        if let Some(command_fn) = self.components.get(component_id) {
            match command_fn(component.clone(), Arc::clone(&ctx)).await {
                Ok(_) => {
                    let member = component.member.clone();
                    let mut display_name = "Unknown".to_string();
                    let mut id = 0;

                    if let Some(mem) = member {
                        display_name = String::from(mem.display_name());
                        id = mem.user.id.get();
                    }

                    Logger::debug(
                        &format!("components.{}", component_id),
                        &format!("component triggered by {} ({})", display_name, id),
                    )
                    .await;
                }
                Err(e) => {
                    Logger::error(
                        &format!("components.{}", component_id),
                        &format!("component caused a panic with error: {}", e.to_string()),
                    )
                    .await
                }
            };
        } else {
            Logger::error(
                "com_man.call_component",
                &format!(
                    "command manager cannot find call for component: {}",
                    component_id
                ),
            )
            .await;
        }
    }

    pub async fn call_modal(&self, modal_id: &str, modal: &ModalInteraction, ctx: Arc<Context>) {
        if let Some(command_fn) = self.modals.get(modal_id) {
            match command_fn(modal.clone(), Arc::clone(&ctx)).await {
                Ok(_) => {
                    let member = modal.member.clone();
                    let mut display_name = "Unknown".to_string();
                    let mut id = 0;

                    if let Some(mem) = member {
                        display_name = String::from(mem.display_name());
                        id = mem.user.id.get();
                    }

                    Logger::debug(
                        &format!("modals.{}", modal_id),
                        &format!("modal triggered by {} ({})", display_name, id),
                    )
                    .await;
                }
                Err(e) => {
                    Logger::error(
                        &format!("modals.{}", modal_id),
                        &format!("modal caused a panic with error: {}", e.to_string()),
                    )
                    .await
                }
            };
        } else {
            Logger::error(
                "com_man.call_modal",
                &format!("command manager cannot find call for modal: {}", modal_id),
            )
            .await;
        }
    }
}
