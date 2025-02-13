use crate::prelude::*;
use command_macro::slash_command;
use serenity::{
    all::CreateAttachment,
    builder::{CreateInteractionResponse, CreateInteractionResponseMessage},
    client::Context,
    model::{application::CommandInteraction, id::GuildId},
};

pub async fn debug_commands(ctx: &Context, guild: GuildId) {
    #[slash_command([])]
    async fn shutdown(ctx: &Context, inter: CommandInteraction) {
        inter
            .create_response(
                &ctx.http,
                CreateInteractionResponse::Message(
                    CreateInteractionResponseMessage::new()
                        .content(loc!("command-done-response"))
                        .ephemeral(true),
                ),
            )
            .await
            .unwrap();

        Logger::high(
            match inter.member {
                Some(ref mem) => mem.display_name(),
                None => "Unknown",
            },
            "shutdowning bot...",
        )
        .await;
        std::process::exit(0);
    }

    #[slash_command([])]
    async fn task_print(ctx: &Context, inter: CommandInteraction) {
        let task_man = task::TASKMANAGER.read().await;

        if let Some(task) = task_man.get_thread(inter.channel_id) {
            inter
                .create_response(
                    &ctx.http,
                    CreateInteractionResponse::Message(
                        CreateInteractionResponseMessage::new()
                            .add_file(CreateAttachment::bytes(
                                format!("{:#?}", task),
                                task.id.to_string(),
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
                            .content(loc!("task-command-not-in-task"))
                            .ephemeral(true),
                    ),
                )
                .await
                .unwrap();
        }
    }
}
