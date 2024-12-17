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
                        CreateInteractionResponseMessage::new().content(get_string(
                            "last-save-command-message",
                            Some(HashMap::from([(
                                "last_save",
                                match task.last_save.get() {
                                    Some(task_save) => task_save.clone(),
                                    None => get_string("task-no-last-save", None),
                                }
                                .as_str(),
                            )])),
                        )),
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
    async fn task_change(ctx: &Context, inter: CommandInteraction) {
        inter.defer_ephemeral(&ctx.http).await.unwrap();

        let task_man = task::TASKMANAGER.read().await;

        if let Some(task) = task_man.get_thread(inter.channel_id) {
            let mut mem_man = member::MEMBERSMANAGER.write().await;
            mem_man.get_mut(inter.user.id).await.unwrap().changed_task = Some(task.id as u32);

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
                        .content(get_string("task-command-not-in-task", None)),
                )
                .await
                .unwrap();
        }
    }

    #[slash_command([])]
    async fn task_close(ctx: &Context, inter: CommandInteraction) {
        let mut mem_man = member::MEMBERSMANAGER.write().await;
        let member = mem_man.get_mut(inter.user.id).await.unwrap();
        let mut task_man = task::TASKMANAGER.write().await;

        if let Some(task) = task_man.get_thread_mut(inter.channel_id) {
            member.changed_task = Some(task.id);
            drop(mem_man);

            task.ending_results = HashMap::new();
            if let Some(mentor) = task.mentor_id.get() {
                task.ending_results.insert(mentor.clone(), 2.0);
            }

            if task.ending_results.len() != task.members.get().len() {
                for member in task.members.get().iter() {
                    if &Some(member.clone()) == task.mentor_id.get() {
                        continue;
                    }

                    inter
                        .create_response(
                            &ctx.http,
                            CreateInteractionResponse::Message(
                                CreateInteractionResponseMessage::new()
                                    .components(Vec::from([task.closing_option(member).await]))
                                    .ephemeral(true),
                            ),
                        )
                        .await
                        .unwrap();
                    return;
                }
            }

            task.close(&ctx).await;

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
                            .content(get_string("task-command-not-in-task", None))
                            .ephemeral(true),
                    ),
                )
                .await
                .unwrap();
        }
    }

    #[slash_command([])]
    async fn ping(ctx: &Context, inter: CommandInteraction) {
        let mut task_man = task::TASKMANAGER.write().await;

        if let Some(task) = task_man.get_thread_mut(inter.channel_id) {
            inter
                .create_response(
                    &ctx.http,
                    CreateInteractionResponse::Message(
                        CreateInteractionResponseMessage::new().content(task.get_members_ping()),
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
}
