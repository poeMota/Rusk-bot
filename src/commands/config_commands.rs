use crate::prelude::*;
use serenity::{self, all::Attachment};

pub async fn config_commands(ctx: &Context, guild: GuildId) {
    #[slash_command([])]
    async fn load_config(
        ctx: &Context,
        inter: CommandInteraction,
        file: Attachment,
        path: Option<String>,
    ) {
        inter.defer_ephemeral(&ctx.http).await.unwrap();

        let mut path_to_upload = DATA_PATH.clone();
        if let Some(p) = &path {
            path_to_upload = path_to_upload.join(p.strip_prefix("/").unwrap_or(p));
        }
        path_to_upload = path_to_upload.join(&file.filename);

        write_file(
            &path_to_upload,
            String::from_utf8_lossy(&file.download().await.unwrap()).to_string(),
        );

        inter
            .edit_response(
                &ctx.http,
                EditInteractionResponse::new().content(get_string("command-done-response", None)),
            )
            .await
            .unwrap();

        Logger::high(
            fetch_member(&inter.user.id).await.unwrap().display_name(),
            &format!(
                "uploaded config file \"{}{}\"",
                path.unwrap_or(String::new()),
                file.filename,
            ),
        )
        .await;
    }
}
