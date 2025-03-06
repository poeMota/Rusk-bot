use crate::{
    connect::{get_user_id, ConnectionError},
    model::{member::MEMBERSMANAGER, role::ROLEMANAGER},
    prelude::*,
};
use serenity::{
    self,
    all::{
        ComponentInteractionDataKind, CreateActionRow, CreateSelectMenu, CreateSelectMenuOption,
    },
};

pub async fn member_commands(ctx: &Context, guild: GuildId) {
    #[slash_command([])]
    async fn my_statistics(ctx: &Context, inter: CommandInteraction) {
        let mut mem_man = MEMBERSMANAGER.write().await;
        let member = mem_man.get(inter.user.id).await.unwrap();

        inter.defer_ephemeral(&ctx.http).await.unwrap();

        inter
            .edit_response(
                &ctx.http,
                EditInteractionResponse::new().embed(member.to_embed(&ctx, false).await),
            )
            .await
            .unwrap();
    }

    #[slash_command([])]
    async fn member_statistics(ctx: &Context, inter: CommandInteraction, dismember: User) {
        let mut mem_man = MEMBERSMANAGER.write().await;
        let member = mem_man.get(dismember.id).await.unwrap();

        inter.defer_ephemeral(&ctx.http).await.unwrap();

        inter
            .edit_response(
                &ctx.http,
                EditInteractionResponse::new().embed(member.to_embed(&ctx, true).await),
            )
            .await
            .unwrap();
    }

    #[slash_command([])]
    async fn change_member(ctx: &Context, inter: CommandInteraction, member: User) {
        let mut mem_man = MEMBERSMANAGER.write().await;
        mem_man.get_mut(inter.user.id).await.unwrap().changed_member = Some(member.id.clone());
        let member = mem_man.get(member.id).await.unwrap();

        inter
            .create_response(
                &ctx.http,
                CreateInteractionResponse::Message(member.main_changer().await),
            )
            .await
            .unwrap();
    }

    #[slash_command([])]
    async fn link_folder(ctx: &Context, inter: CommandInteraction, folder: String) {
        inter.defer_ephemeral(&ctx.http).await.unwrap();

        let dis_member = fetch_member(&inter.user.id).await.unwrap();

        let role_man = ROLEMANAGER.read().await;
        let mut mem_man = MEMBERSMANAGER.write().await;

        let dbs = role_man.member_db_permissons(&dis_member);

        if dbs.is_empty() {
            inter
                .edit_response(
                    &ctx.http,
                    EditInteractionResponse::new().content(loc!("link-folder-command-no-dbs")),
                )
                .await
                .unwrap();

            return;
        }

        if dbs.len() > 1 {
            inter
                .edit_response(
                    &ctx.http,
                    EditInteractionResponse::new()
                        .content(loc!("link-folder-command-db-title"))
                        .components(Vec::from([CreateActionRow::SelectMenu(
                            CreateSelectMenu::new(
                                "link-folder:db",
                                serenity::all::CreateSelectMenuKind::String {
                                    options: dbs
                                        .iter()
                                        .map(|x| {
                                            CreateSelectMenuOption::new(
                                                *x,
                                                format!("{}:::{}", x, folder),
                                            )
                                        })
                                        .collect(),
                                },
                            ),
                        )])),
                )
                .await
                .unwrap();

            return;
        }

        let db = *dbs.get(0).unwrap();

        let other_folder = mem_man.get_by_folder(db.clone(), &folder).cloned();
        let member = mem_man.get_mut(inter.user.id).await.unwrap();

        if let None = member.own_folder.get(db.as_str()) {
            if let Some(id) = other_folder {
                inter
                    .edit_response(
                        &ctx.http,
                        EditInteractionResponse::new()
                            .content(loc!("link-folder-command-stranger-folder-response")),
                    )
                    .await
                    .unwrap();

                Logger::notify(
                    inter.user.display_name(),
                    &loc!(
                        "link-folder-command-stranger-folder-notify",
                        "member1" = inter.user.id.get(),
                        "folder" = folder,
                        "member2" = id.get(),
                    ),
                )
                .await;
            } else {
                match member.change_folder(db.clone(), Some(folder)).await {
                    Ok(_) => inter
                        .edit_response(
                            &ctx.http,
                            EditInteractionResponse::new().content(loc!("command-done-response")),
                        )
                        .await
                        .unwrap(),
                    Err(e) => match e {
                        ConnectionError::StatusCodeError(url, _) => inter
                            .edit_response(
                                &ctx.http,
                                EditInteractionResponse::new()
                                    .content(loc!("invalid-url", "path" = url)),
                            )
                            .await
                            .unwrap(),
                        ConnectionError::NotAllowedUrl(_) => inter
                            .edit_response(
                                &ctx.http,
                                EditInteractionResponse::new().content(loc!("not-allowed-url")),
                            )
                            .await
                            .unwrap(),
                        _ => {
                            Logger::error(
                                "commands.link_folder",
                                &format!("error while connecting: {:?}", e),
                            )
                            .await;

                            inter
                                .edit_response(
                                    &ctx.http,
                                    EditInteractionResponse::new()
                                        .content(loc!("link-folder-error")),
                                )
                                .await
                                .unwrap()
                        }
                    },
                };
            }
        } else {
            inter
                .edit_response(
                    &ctx.http,
                    EditInteractionResponse::new()
                        .content(loc!("link-folder-command-already-linked-response")),
                )
                .await
                .unwrap();
        }
    }

    #[slash_command([])]
    async fn user_id(ctx: &Context, inter: CommandInteraction, ckey: String) {
        inter.defer_ephemeral(&ctx.http).await.unwrap();

        inter
            .edit_response(
                &ctx.http,
                EditInteractionResponse::new()
                    .content(format!("```{}```", get_user_id(ckey).await)),
            )
            .await
            .unwrap();
    }

    #[slash_command([])]
    async fn note(ctx: &Context, inter: CommandInteraction, member: User, text: String) {
        let mut mem_man = member::MEMBERSMANAGER.write().await;
        let mem = mem_man.get_mut(member.id).await.unwrap();

        mem.add_note(inter.user.id, text).await;

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
    }

    #[slash_command([])]
    async fn warning(ctx: &Context, inter: CommandInteraction, member: User, text: String) {
        let mut mem_man = member::MEMBERSMANAGER.write().await;
        let mem = mem_man.get_mut(member.id).await.unwrap();

        mem.add_warn(inter.user.id, text).await;

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
    }

    #[listen_component("link-folder:db")]
    async fn link_folder_db(ctx: &Context, inter: ComponentInteraction) {
        if let ComponentInteractionDataKind::StringSelect { values } = &inter.data.kind {
            let parts = values.first().unwrap().split(":::").collect::<Vec<&str>>();
            let db = parts.get(0).unwrap().to_string();
            let folder = parts.get(1).unwrap().to_string();

            let mut mem_man = MEMBERSMANAGER.write().await;
            let member = mem_man.get_mut(inter.user.id).await.unwrap();

            match member.change_folder(db, Some(folder)).await {
                Ok(_) => inter
                    .edit_response(
                        &ctx.http,
                        EditInteractionResponse::new()
                            .content(loc!("command-done-response"))
                            .components(vec![]),
                    )
                    .await
                    .unwrap(),
                Err(e) => match e {
                    ConnectionError::StatusCodeError(url, _) => inter
                        .edit_response(
                            &ctx.http,
                            EditInteractionResponse::new()
                                .content(loc!("invalid-url", "path" = url))
                                .components(vec![]),
                        )
                        .await
                        .unwrap(),
                    ConnectionError::NotAllowedUrl(_) => inter
                        .edit_response(
                            &ctx.http,
                            EditInteractionResponse::new()
                                .content(loc!("not-allowed-url"))
                                .components(vec![]),
                        )
                        .await
                        .unwrap(),
                    _ => {
                        Logger::error(
                            "commands.link_folder",
                            &format!("error while connecting: {:?}", e),
                        )
                        .await;

                        inter
                            .edit_response(
                                &ctx.http,
                                EditInteractionResponse::new().content(loc!("link-folder-error")),
                            )
                            .await
                            .unwrap()
                    }
                },
            };
        }
    }
}
