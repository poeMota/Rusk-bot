use std::collections::HashMap;

use crate::{connect::*, prelude::*};
use serenity::{
    self,
    all::{
        ComponentInteractionDataKind, CreateActionRow, CreateAttachment, CreateSelectMenu,
        CreateSelectMenuOption,
    },
};

pub async fn save_commands(ctx: &Context, guild: GuildId) {
    #[slash_command([])]
    async fn save(ctx: &Context, inter: CommandInteraction, path: Option<String>) {
        inter.defer_ephemeral(&ctx.http).await.unwrap();

        let mut mem_man = member::MEMBERSMANAGER.write().await;
        let member = mem_man.get(inter.user.id).await.unwrap();

        if let Some(folder) = &member.own_folder {
            if let Some(p) = path {
                if p.ends_with("/") {
                    inter
                        .edit_response(
                            &ctx.http,
                            EditInteractionResponse::new()
                                .content(get_string("unload-save-menu-title", None))
                                .components(Vec::from([save_menu_component(
                                    format!("{}/{}", folder, p),
                                    format!("{}/{}", folder, p),
                                )
                                .await])),
                        )
                        .await
                        .unwrap();
                } else {
                    match unload_content(format!("{}/{}", folder, p)).await {
                        Ok(data) => {
                            inter
                                .edit_response(
                                    &ctx.http,
                                    EditInteractionResponse::new().new_attachment(
                                        CreateAttachment::bytes(
                                            data,
                                            p.split("/").last().unwrap_or("unknown.yml"),
                                        ),
                                    ),
                                )
                                .await
                                .unwrap();

                            Logger::low(
                                "commands.save",
                                &format!(
                                    "unloaded save {}/{} by {} ({}",
                                    folder,
                                    p,
                                    inter.user.display_name(),
                                    inter.user.id.get()
                                ),
                            )
                            .await;
                        }
                        Err(e) => match e {
                            ConnectionError::InvalidUrl(url) => {
                                inter
                                    .edit_response(
                                        &ctx.http,
                                        EditInteractionResponse::new().content(get_string(
                                            "invalid-url",
                                            Some(HashMap::from([("path", url.as_str())])),
                                        )),
                                    )
                                    .await
                                    .unwrap();
                            }
                            ConnectionError::NotAllowedUrl(_) => {
                                inter
                                    .edit_response(
                                        &ctx.http,
                                        EditInteractionResponse::new()
                                            .content(get_string("not-allowed-url", None)),
                                    )
                                    .await
                                    .unwrap();
                            }
                            _ => {
                                Logger::error(
                                    "commands.save",
                                    &format!("error while connecting: {:?}", e),
                                )
                                .await;

                                inter
                                    .edit_response(
                                        &ctx.http,
                                        EditInteractionResponse::new()
                                            .content(get_string("save-unload-error", None)),
                                    )
                                    .await
                                    .unwrap();
                            }
                        },
                    }
                }
            } else {
                inter
                    .edit_response(
                        &ctx.http,
                        EditInteractionResponse::new()
                            .content(get_string("unload-save-menu-title", None))
                            .components(Vec::from([save_menu_component(
                                format!("{}/", folder),
                                format!("{}/", folder),
                            )
                            .await])),
                    )
                    .await
                    .unwrap();
            }
        } else {
            inter
                .edit_response(
                    &ctx.http,
                    EditInteractionResponse::new()
                        .content(get_string("save-command-folder-not-linked", None)),
                )
                .await
                .unwrap();
        }
    }

    #[slash_command([])]
    async fn save_plus(ctx: &Context, inter: CommandInteraction, path: Option<String>) {
        inter.defer_ephemeral(&ctx.http).await.unwrap();

        if let Some(p) = path {
            if p.ends_with("/") {
                inter
                    .edit_response(
                        &ctx.http,
                        EditInteractionResponse::new()
                            .content(get_string("unload-save-menu-title", None))
                            .components(Vec::from([save_menu_component(p.clone(), p).await])),
                    )
                    .await
                    .unwrap();
            } else {
                match unload_content(p.clone()).await {
                    Ok(data) => {
                        inter
                            .edit_response(
                                &ctx.http,
                                EditInteractionResponse::new().new_attachment(
                                    CreateAttachment::bytes(
                                        data,
                                        p.split("/").last().unwrap_or("unknown.yml"),
                                    ),
                                ),
                            )
                            .await
                            .unwrap();

                        Logger::low(
                            "commands.save_plus",
                            &format!(
                                "unloaded save {} by {} ({}",
                                p,
                                inter.user.display_name(),
                                inter.user.id.get()
                            ),
                        )
                        .await;
                    }
                    Err(e) => match e {
                        ConnectionError::InvalidUrl(url) => {
                            inter
                                .edit_response(
                                    &ctx.http,
                                    EditInteractionResponse::new().content(get_string(
                                        "invalid-url",
                                        Some(HashMap::from([("path", url.as_str())])),
                                    )),
                                )
                                .await
                                .unwrap();
                        }
                        ConnectionError::NotAllowedUrl(_) => {
                            inter
                                .edit_response(
                                    &ctx.http,
                                    EditInteractionResponse::new()
                                        .content(get_string("not-allowed-url", None)),
                                )
                                .await
                                .unwrap();
                        }
                        _ => {
                            Logger::error(
                                "commands.save_plus",
                                &format!("error while connecting: {:?}", e),
                            )
                            .await;

                            inter
                                .edit_response(
                                    &ctx.http,
                                    EditInteractionResponse::new()
                                        .content(get_string("save-unload-error", None)),
                                )
                                .await
                                .unwrap();
                        }
                    },
                }
            }
        } else {
            inter
                .edit_response(
                    &ctx.http,
                    EditInteractionResponse::new()
                        .content(get_string("unload-save-menu-title", None))
                        .components(Vec::from([save_menu_component(
                            String::new(),
                            String::new(),
                        )
                        .await])),
                )
                .await
                .unwrap();
        }
    }

    async fn save_menu_component(path: String, root: String) -> CreateActionRow {
        let mut options = Vec::new();
        let mut index = 0;
        for (filename, date) in file_dates(path.clone()).await.unwrap() {
            if options.len() == 25 {
                break;
            }

            if filename.as_str() == "../" {
                let parent = path.replace(path.split("/").last().unwrap_or(""), "");

                if parent == root {
                    continue;
                }
            }

            options.push(
                CreateSelectMenuOption::new(
                    filename.clone(),
                    format!("{}:::{}:::{}{}", index, root, path, filename),
                )
                .description(date),
            );
            index += 1;
        }

        CreateActionRow::SelectMenu(CreateSelectMenu::new(
            "save-menu-option",
            serenity::all::CreateSelectMenuKind::String { options },
        ))
    }

    #[listen_component("save-menu-option")]
    async fn save_menu_response(ctx: &Context, inter: ComponentInteraction) {
        if let ComponentInteractionDataKind::StringSelect { values } = &inter.data.kind {
            for value in values {
                let root = value
                    .split(":::")
                    .collect::<Vec<&str>>()
                    .get(1)
                    .unwrap()
                    .to_string();

                if value.contains("../") {
                    let mut path = Vec::new();
                    for p in value.split(":::").last().unwrap().to_string().split("/") {
                        if p == ".." {
                            path.pop();
                            break;
                        }

                        path.push(p.to_string());
                    }

                    inter
                        .create_response(
                            &ctx.http,
                            CreateInteractionResponse::UpdateMessage(
                                CreateInteractionResponseMessage::new()
                                    .components(Vec::from([save_menu_component(
                                        format!("{}/", path.join("/")),
                                        root,
                                    )
                                    .await]))
                                    .ephemeral(true),
                            ),
                        )
                        .await
                        .unwrap();
                } else if value.ends_with("/") {
                    inter
                        .create_response(
                            &ctx.http,
                            CreateInteractionResponse::UpdateMessage(
                                CreateInteractionResponseMessage::new()
                                    .components(Vec::from([save_menu_component(
                                        value.split(":::").last().unwrap().to_string(),
                                        root,
                                    )
                                    .await]))
                                    .ephemeral(true),
                            ),
                        )
                        .await
                        .unwrap();
                } else {
                    inter.defer_ephemeral(&ctx.http).await.unwrap();
                    let path = value.split(":::").last().unwrap();

                    inter
                        .edit_response(
                            &ctx.http,
                            EditInteractionResponse::new().new_attachment(CreateAttachment::bytes(
                                unload_content(path.to_string()).await.unwrap(),
                                path.split("/").last().unwrap(),
                            )),
                        )
                        .await
                        .unwrap();

                    Logger::low(
                        "components.save-menu-option",
                        &format!(
                            "unloaded save {} by {} ({}",
                            path,
                            inter.user.display_name(),
                            inter.user.id.get()
                        ),
                    )
                    .await;
                }
            }
        }
    }
}
