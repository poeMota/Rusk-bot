use crate::{command_manager::COMMANDMANAGER, config::CONFIG, localization::get_string};
use command_macro::command;
use serenity::{
    builder::{CreateInteractionResponse, CreateInteractionResponseMessage, CreateMessage},
    client::Context,
    model::{application::CommandInteraction, id::GuildId},
};

pub async fn fun_commands(ctx: Context, guild: GuildId) {
    #[command([])]
    async fn bot_send(ctx: Context, inter: CommandInteraction, message: String) {
        inter
            .create_response(
                &ctx.http,
                CreateInteractionResponse::Message(
                    CreateInteractionResponseMessage::new().content("**Done**"),
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

    #[command([])]
    async fn when(ctx: Context, inter: CommandInteraction) {
        bot_send(ctx, inter, get_string("when-command-responce", None)).await;
    }
}
