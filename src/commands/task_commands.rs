use std::collections::HashMap;

use crate::prelude::*;
use serenity::{
    self,
    all::{Colour, CreateEmbed},
};

pub async fn task_commands(ctx: &Context, guild: GuildId) {
    #[slash_command([])]
    async fn task_info(ctx: &Context, inter: CommandInteraction) {
        let task_man = task::TASKMANAGER.read().await;

        if let Some(task) = task_man.get_thread(inter.channel_id) {
            inter
                .create_response(
                    &ctx.http,
                    CreateInteractionResponse::Message(
                        CreateInteractionResponseMessage::new()
                            .embed(task.to_embed())
                            .ephemeral(true),
                    ),
                )
                .await
                .unwrap();
        } else {
            inter
                .create_response(
                    &ctx.http,
                    CreateInteractionResponse::Message(
                        CreateInteractionResponseMessage::new()
                            .content(get_string("task-command-not-in-task", None))
                            .ephemeral(true),
                    ),
                )
                .await
                .unwrap();
        }
    }

    #[slash_command([])]
    async fn last_save(ctx: &Context, inter: CommandInteraction, path: Option<String>) {
        let mut task_man = task::TASKMANAGER.write().await;

        if let Some(task) = task_man.get_thread_mut(inter.channel_id) {
            if let Some(save) = path {
                task.set_last_save(&ctx, Some(save)).await;
            }

            inter
                .create_response(
                    &ctx.http,
                    CreateInteractionResponse::Message(
                        CreateInteractionResponseMessage::new()
                            .content(get_string(
                                "last-save-command-message",
                                Some(HashMap::from([(
                                    "last_save",
                                    match task.last_save.get() {
                                        Some(task_save) => task_save.clone(),
                                        None => get_string("task-no-last-save", None),
                                    }
                                    .as_str(),
                                )])),
                            ))
                            .ephemeral(true),
                    ),
                )
                .await
                .unwrap();
        } else {
            inter
                .create_response(
                    &ctx.http,
                    CreateInteractionResponse::Message(
                        CreateInteractionResponseMessage::new()
                            .content(get_string("task-command-not-in-task", None))
                            .ephemeral(true),
                    ),
                )
                .await
                .unwrap();
        }
    }

    #[slash_command([])]
    async fn become_mentor(ctx: &Context, inter: CommandInteraction) {
        let mut task_man = task::TASKMANAGER.write().await;

        if let Some(task) = task_man.get_thread_mut(inter.channel_id) {
            if let None = task.mentor_id.get() {
                if task.set_mentor(&ctx, Some(inter.user.id), false).await {
                    inter
                        .create_response(
                            &ctx.http,
                            CreateInteractionResponse::Message(
                                CreateInteractionResponseMessage::new()
                                    .content(get_string("command-done-response", None))
                                    .ephemeral(true),
                            ),
                        )
                        .await
                        .unwrap();
                } else {
                    inter
                        .create_response(
                            &ctx.http,
                            CreateInteractionResponse::Message(
                                CreateInteractionResponseMessage::new()
                                    .content(get_string(
                                        "become-mentor-command-max-members-error",
                                        None,
                                    ))
                                    .ephemeral(true),
                            ),
                        )
                        .await
                        .unwrap();
                }
            } else {
                inter
                    .create_response(
                        &ctx.http,
                        CreateInteractionResponse::Message(
                            CreateInteractionResponseMessage::new()
                                .content(get_string("become-mentor-command-mentor-exist", None))
                                .ephemeral(true),
                        ),
                    )
                    .await
                    .unwrap();
            }
        } else {
            inter
                .create_response(
                    &ctx.http,
                    CreateInteractionResponse::Message(
                        CreateInteractionResponseMessage::new()
                            .content(get_string("task-command-not-in-task", None))
                            .ephemeral(true),
                    ),
                )
                .await
                .unwrap();
        }
    }

    #[slash_command([])]
    async fn task_change(ctx: &Context, inter: CommandInteraction, id: i64) {
        inter.defer_ephemeral(&ctx.http).await.unwrap();

        let task_man = task::TASKMANAGER.read().await;

        if let Some(task) = task_man.get(id as u32) {
            let mut mem_man = member::MEMBERSMANAGER.write().await;
            mem_man.get_mut(inter.user.id).await.unwrap().changed_task = Some(id as u32);

            inter
                .edit_response(
                    &ctx.http,
                    EditInteractionResponse::new()
                        .embed(
                            CreateEmbed::new()
                                .title(get_string("task-changer-embed-title", None))
                                .description(get_string(
                                    "task-changer-embed-description",
                                    Some(HashMap::from([("task", task.name.get().as_str())])),
                                ))
                                .color(Colour::BLUE),
                        )
                        .components(task.main_changer().await),
                )
                .await
                .unwrap();
        } else {
            inter
                .edit_response(
                    &ctx.http,
                    EditInteractionResponse::new()
                        .content(get_string("task-change-command-not-found", None)),
                )
                .await
                .unwrap();
        }
    }

    // TODO
    #[slash_command([])]
    async fn close(ctx: &Context, inter: CommandInteraction) {
        let task_man = task::TASKMANAGER.read().await;

        if let Some(_task) = task_man.get_thread(inter.channel_id) {
        } else {
            inter
                .create_response(
                    &ctx.http,
                    CreateInteractionResponse::Message(
                        CreateInteractionResponseMessage::new()
                            .content(get_string("task-command-not-in-task", None))
                            .ephemeral(true),
                    ),
                )
                .await
                .unwrap();
        }
    }
}
