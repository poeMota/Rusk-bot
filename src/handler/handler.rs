use crate::{
    command_manager::COMMANDMANAGER, commands::*, config::CONFIG, logger::Logger,
    model::MEMBERSMANAGER, shop::SHOPMANAGER,
};
use serenity::{
    all::async_trait,
    builder::{CreateButton, CreateInteractionResponse, EditInteractionResponse},
    client::{Context, EventHandler},
    http::Http,
    model::{
        application::{ButtonStyle, ComponentInteractionDataKind, Interaction},
        gateway::Ready,
        id::GuildId,
    },
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
        //clear_guild_commands(&ctx.http, &guild_id).await;

        fun_commands(ctx.clone(), guild_id).await;
        debug_commands(ctx.clone(), guild_id).await;
        shop_commands(ctx.clone(), guild_id).await;

        Logger::debug("handler.ready", "bot is ready").await;
    }

    async fn interaction_create(&self, ctx: Context, interaction: Interaction) {
        match interaction {
            Interaction::Command(ref command) => {
                let command_man = COMMANDMANAGER.try_read().unwrap();
                command_man
                    .call_command(&command.data.name, command.clone(), Arc::new(ctx))
                    .await;
            }
            Interaction::Component(ref component) => {
                if let ComponentInteractionDataKind::Button = component.data.kind {
                    let mut mem_man = MEMBERSMANAGER.try_write().unwrap();
                    let member = mem_man.get_mut(component.user.id.clone()).await.unwrap();

                    let shop_man = SHOPMANAGER.try_read().unwrap();
                    member.shop_data.pages = shop_man.get_pages(&ctx, member).await;

                    component.defer_ephemeral(&ctx.http).await.unwrap();

                    match component.data.custom_id.as_str() {
                        "previous" => {
                            member.shop_data.current_page -= 1;
                            if member.shop_data.current_page < 0 {
                                member.shop_data.current_page =
                                    member.shop_data.pages.len() as i32 - 1;
                            }
                        }
                        "next" => {
                            member.shop_data.current_page += 1;
                            if member.shop_data.current_page
                                > member.shop_data.pages.len() as i32 - 1
                            {
                                member.shop_data.current_page = 0;
                            }
                        }
                        "buy" => {
                            if let Some(page) = member
                                .shop_data
                                .pages
                                .get(member.shop_data.current_page as usize)
                                .cloned()
                            {
                                page.buy(&ctx, component, member).await;
                            }
                        }
                        _ => (),
                    }

                    component
                        .edit_response(
                            &ctx.http,
                            EditInteractionResponse::new()
                                .embed(
                                    member
                                        .shop_data
                                        .pages
                                        .get(member.shop_data.current_page as usize)
                                        .unwrap()
                                        .to_embed(&member, member.shop_data.pages.len() as i32),
                                )
                                .button(
                                    CreateButton::new("previous")
                                        .emoji('◀')
                                        .style(ButtonStyle::Secondary),
                                )
                                .button(
                                    CreateButton::new("buy")
                                        .emoji('🛒')
                                        .style(ButtonStyle::Success),
                                )
                                .button(
                                    CreateButton::new("next")
                                        .emoji('▶')
                                        .style(ButtonStyle::Secondary),
                                ),
                        )
                        .await
                        .unwrap();
                }
            }
            _ => (),
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
