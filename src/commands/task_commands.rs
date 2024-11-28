use crate::prelude::*;
use serenity;

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
}
