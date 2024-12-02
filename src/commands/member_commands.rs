use std::collections::HashMap;

use crate::{
    connect::{get_user_id, ConnectionError},
    model::member::MEMBERSMANAGER,
    prelude::*,
};
use serenity;

pub async fn member_commands(ctx: &Context, guild: GuildId) {
    #[slash_command([])]
    async fn my_statistics(ctx: &Context, inter: CommandInteraction) {
        let mut mem_man = MEMBERSMANAGER.try_write().unwrap();
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
        let mut mem_man = MEMBERSMANAGER.try_write().unwrap();
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
        let mut mem_man = MEMBERSMANAGER.try_write().unwrap();
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

        let mut mem_man = MEMBERSMANAGER.try_write().unwrap();
        let member = mem_man.get_mut(inter.user.id).await.unwrap();

        if let None = member.own_folder {
            match member.change_folder(Some(folder)).await {
                Ok(_) => inter
                    .edit_response(
                        &ctx.http,
                        EditInteractionResponse::new()
                            .content(get_string("command-done-response", None)),
                    )
                    .await
                    .unwrap(),
                Err(e) => match e {
                    ConnectionError::InvalidUrl(url) => inter
                        .edit_response(
                            &ctx.http,
                            EditInteractionResponse::new().content(get_string(
                                "invalid-url",
                                Some(HashMap::from([("path", url.as_str())])),
                            )),
                        )
                        .await
                        .unwrap(),
                    ConnectionError::ReqwestError(error) => {
                        Logger::error(
                            "commands.link_folder",
                            &format!("reqwest error while connection: {}", error.to_string()),
                        )
                        .await;

                        inter
                            .edit_response(
                                &ctx.http,
                                EditInteractionResponse::new()
                                    .content(get_string("link-folder-reqwest-error", None)),
                            )
                            .await
                            .unwrap()
                    }
                    _ => inter
                        .edit_response(
                            &ctx.http,
                            EditInteractionResponse::new().content(get_string(
                                "link-folder-error",
                                Some(HashMap::from([("error", format!("{:#?}", e).as_str())])),
                            )),
                        )
                        .await
                        .unwrap(),
                },
            };
        } else {
            inter
                .edit_response(
                    &ctx.http,
                    EditInteractionResponse::new().content(get_string(
                        "link-folder-command-already-linked-response",
                        None,
                    )),
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
                        .content(get_string("command-done-response", None))
                        .ephemeral(true),
                ),
            )
            .await
            .unwrap();
    }

    #[slash_command([])]
    async fn warn(ctx: &Context, inter: CommandInteraction, member: User, text: String) {
        let mut mem_man = member::MEMBERSMANAGER.write().await;
        let mem = mem_man.get_mut(member.id).await.unwrap();

        mem.add_warn(inter.user.id, text).await;

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
    }
}
