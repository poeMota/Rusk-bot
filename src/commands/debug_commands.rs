use crate::{
    command_manager::COMMANDMANAGER, config::CONFIG, localization::get_string, logger::Logger,
};
use command_macro::slash_command;
use serenity::{
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
                        .content(get_string("commands-done-response", None))
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
            "shutdown bot...",
        )
        .await;
        std::process::exit(0);
    }
}
