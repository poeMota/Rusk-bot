use crate::prelude::*;

pub async fn fun_commands(ctx: &Context, guild: GuildId) {
    #[slash_command([])]
    async fn bot_send(ctx: &Context, inter: CommandInteraction, message: String) {
        inter
            .create_response(
                &ctx.http,
                CreateInteractionResponse::Message(
                    CreateInteractionResponseMessage::new()
                        .content(get_string("commands-done-response", None)),
                ),
            )
            .await
            .unwrap();

        inter
            .get_response(&ctx.http)
            .await
            .unwrap()
            .delete(&ctx.http)
            .await
            .unwrap();

        inter
            .channel_id
            .send_message(&ctx.http, CreateMessage::new().content(message))
            .await
            .unwrap();
    }

    #[slash_command([])]
    async fn when(ctx: &Context, inter: CommandInteraction) {
        bot_send(ctx, inter, get_string("when-command-responce", None)).await;
    }
}
