use std::collections::HashMap;

use crate::prelude::*;
use serenity::{
    self,
    all::{Colour, CreateEmbed},
};

pub async fn tag_commands(ctx: &Context, guild: GuildId) {
    #[slash_command([])]
    async fn create_tag(
        ctx: &Context,
        inter: CommandInteraction,
        channel: PartialChannel,
        tag_name: String,
    ) {
        inter.defer_ephemeral(&ctx.http).await.unwrap();

        let mut tag_man = tag::TAGSMANAGER.write().await;
        let channel = fetch_channel(&ctx, channel.id).unwrap();

        for tag in channel.available_tags.iter() {
            if tag.name == tag_name {
                tag_man.new_tag(tag.id, channel.id).await;
                inter
                    .edit_response(
                        &ctx.http,
                        EditInteractionResponse::new()
                            .content(get_string("command-done-response", None)),
                    )
                    .await
                    .unwrap();
                return;
            }
        }

        inter
            .edit_response(
                &ctx.http,
                EditInteractionResponse::new().content(get_string(
                    "tag-not-found",
                    Some(HashMap::from([
                        ("tag_name", tag_name.as_str()),
                        ("channel", channel.id.get().to_string().as_str()),
                    ])),
                )),
            )
            .await
            .unwrap();
    }

    #[slash_command([])]
    async fn change_tag(
        ctx: &Context,
        inter: CommandInteraction,
        channel: PartialChannel,
        tag_name: String,
    ) {
        inter.defer_ephemeral(&ctx.http).await.unwrap();

        let mut mem_man = member::MEMBERSMANAGER.write().await;
        let tag_man = tag::TAGSMANAGER.read().await;
        let channel = fetch_channel(&ctx, channel.id).unwrap();

        for tag in channel.available_tags.iter() {
            if tag.name == tag_name {
                if let Some(task_tag) = tag_man.get(&tag.id) {
                    mem_man.get_mut(inter.user.id).await.unwrap().changed_tag = Some(tag.id);

                    inter
                        .edit_response(
                            &ctx.http,
                            EditInteractionResponse::new()
                                .embed(
                                    CreateEmbed::new()
                                        .colour(Colour::DARK_GREY)
                                        .title(get_string("tag-changer-embed-title", None))
                                        .description(get_string(
                                            "tag-changer-embed-description",
                                            Some(HashMap::from([
                                                ("tag", tag.name.as_str()),
                                                ("channel", channel.id.get().to_string().as_str()),
                                            ])),
                                        )),
                                )
                                .components(task_tag.main_changer().await),
                        )
                        .await
                        .unwrap();
                    return;
                }
            }
        }

        inter
            .edit_response(
                &ctx.http,
                EditInteractionResponse::new().content(get_string(
                    "tag-not-found",
                    Some(HashMap::from([
                        ("tag_name", tag_name.as_str()),
                        ("channel", channel.id.get().to_string().as_str()),
                    ])),
                )),
            )
            .await
            .unwrap();
    }

    #[slash_command([])]
    async fn tag_config(
        ctx: &Context,
        inter: CommandInteraction,
        channel: PartialChannel,
        tag_name: String,
    ) {
        inter.defer_ephemeral(&ctx.http).await.unwrap();

        let tag_man = tag::TAGSMANAGER.read().await;
        let channel = fetch_channel(&ctx, channel.id).unwrap();

        for tag in channel.available_tags.iter() {
            if tag.name == tag_name {
                if let Some(task_tag) = tag_man.get(&tag.id) {
                    inter
                        .edit_response(
                            &ctx.http,
                            EditInteractionResponse::new().embed(task_tag.to_embed()),
                        )
                        .await
                        .unwrap();
                    return;
                }
            }
        }

        inter
            .edit_response(
                &ctx.http,
                EditInteractionResponse::new().content(get_string(
                    "tag-not-found",
                    Some(HashMap::from([
                        ("tag_name", tag_name.as_str()),
                        ("channel", channel.id.get().to_string().as_str()),
                    ])),
                )),
            )
            .await
            .unwrap();
    }
}
