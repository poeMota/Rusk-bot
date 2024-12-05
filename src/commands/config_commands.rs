use std::fs;
use std::path::PathBuf;

use crate::prelude::*;
use serenity::{
    self,
    all::{
        Attachment, ComponentInteractionDataKind, CreateActionRow, CreateAttachment,
        CreateSelectMenu, CreateSelectMenuOption,
    },
};

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

    #[slash_command([])]
    async fn unload_config(ctx: &Context, inter: CommandInteraction) {
        inter.defer_ephemeral(&ctx.http).await.unwrap();

        inter
            .edit_response(
                &ctx.http,
                EditInteractionResponse::new()
                    .components(Vec::from([unload_component(DATA_PATH.clone()).await])),
            )
            .await
            .unwrap();
    }

    async fn unload_component(path: PathBuf) -> CreateActionRow {
        let mut options = Vec::new();

        for file in fs::read_dir(path).unwrap() {
            if let Ok(file) = file {
                if options.len() == 25 {
                    break;
                }

                if file.file_name().to_str().unwrap().starts_with(".") {
                    continue;
                }

                if file.path().is_dir() {
                    options.push(CreateSelectMenuOption::new(
                        format!("{}/", file.file_name().into_string().unwrap()),
                        format!("{}/", file.path().to_str().unwrap()),
                    ));
                } else {
                    options.push(CreateSelectMenuOption::new(
                        file.file_name().into_string().unwrap(),
                        file.path().to_str().unwrap(),
                    ));
                }
            }
        }

        CreateActionRow::SelectMenu(
            CreateSelectMenu::new(
                "unload-config:file",
                serenity::all::CreateSelectMenuKind::String { options },
            )
            .placeholder(get_string("unload-config-select-placeholder", None)),
        )
    }

    #[listen_component("unload-config:file")]
    async fn unload_config_response(ctx: &Context, inter: ComponentInteraction) {
        if let ComponentInteractionDataKind::StringSelect { values } = &inter.data.kind {
            for value in values.iter() {
                if value.ends_with("/") {
                    inter
                        .create_response(
                            &ctx.http,
                            CreateInteractionResponse::Message(
                                CreateInteractionResponseMessage::new()
                                    .components(Vec::from([
                                        unload_component(PathBuf::from(value)).await
                                    ]))
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
                                    .add_file(CreateAttachment::bytes(
                                        read_file(&DATA_PATH.join(&value)),
                                        value,
                                    ))
                                    .ephemeral(true),
                            ),
                        )
                        .await
                        .unwrap();

                    Logger::high(
                        "commands.unload_config",
                        &format!("unloaded config file: {}", value),
                    )
                    .await;
                }
            }
        }
    }
}
