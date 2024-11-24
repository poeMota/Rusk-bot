use serenity::all::{
    ActionRowComponent, CreateActionRow, CreateInputText, CreateModal, InputTextStyle,
};

use crate::prelude::*;

pub async fn member_changer_listener() {
    #[listen_component("member-changer:score")]
    async fn score_changer(ctx: &Context, inter: ComponentInteraction) {
        let mut mem_man = member::MEMBERSMANAGER.write().await;
        let member = mem_man.get(inter.user.id).await.unwrap();

        inter
            .create_response(
                &ctx.http,
                CreateInteractionResponse::Modal(
                    CreateModal::new(
                        "member-changer:score",
                        get_string("member-changer-modal-score-title", None),
                    )
                    .components(Vec::from([CreateActionRow::InputText(
                        CreateInputText::new(
                            InputTextStyle::Short,
                            get_string("member-changer-modal-score-components-score-title", None),
                            "member-changer:score:score",
                        )
                        .value(member.score.to_string()),
                    )])),
                ),
            )
            .await
            .unwrap();
    }

    #[listen_component("member-changer:own-folder")]
    async fn folder_changer(ctx: &Context, inter: ComponentInteraction) {
        let mut mem_man = member::MEMBERSMANAGER.write().await;
        let member = mem_man.get(inter.user.id).await.unwrap();

        inter
            .create_response(
                &ctx.http,
                CreateInteractionResponse::Modal(
                    CreateModal::new(
                        "member-changer:own-folder",
                        get_string("member-changer-modal-own-folder-title", None),
                    )
                    .components(Vec::from([CreateActionRow::InputText(
                        CreateInputText::new(
                            InputTextStyle::Short,
                            get_string(
                                "member-changer-modal-own-folder-components-folder-title",
                                None,
                            ),
                            "member-changer:own-folder:folder",
                        )
                        .value(member.own_folder.clone().unwrap_or(String::new())),
                    )])),
                ),
            )
            .await
            .unwrap();
    }

    #[listen_component("member-changer:tasks")]
    async fn tasks_changer(ctx: &Context, inter: ComponentInteraction) {
        inter
            .create_response(
                &ctx.http,
                CreateInteractionResponse::Modal(CreateModal::new(
                    "member-changer:tasks",
                    get_string("member-changer-modal-tasks-title", None),
                )),
            )
            .await
            .unwrap();
    }

    #[listen_component("member-changer:notes")]
    async fn notes_changer(ctx: &Context, inter: ComponentInteraction) {
        inter
            .create_response(
                &ctx.http,
                CreateInteractionResponse::Modal(CreateModal::new(
                    "member-changer:notes",
                    get_string("member-changer-modal-notes-title", None),
                )),
            )
            .await
            .unwrap();
    }

    #[listen_component("member-changer:warns")]
    async fn warns_changer(ctx: &Context, inter: ComponentInteraction) {
        inter
            .create_response(
                &ctx.http,
                CreateInteractionResponse::Modal(CreateModal::new(
                    "member-changer:warns",
                    get_string("member-changer-modal-warns-title", None),
                )),
            )
            .await
            .unwrap();
    }

    #[listen_modal("member-changer:score")]
    async fn score_modal_submit(ctx: &Context, inter: ModalInteraction) {
        let mut mem_man = member::MEMBERSMANAGER.write().await;
        let member = mem_man.get_mut(inter.user.id).await.unwrap();

        for row in inter.data.components.iter() {
            for comp in row.components.iter() {
                match comp {
                    ActionRowComponent::InputText(text) => {
                        if text.custom_id == "member-changer:score:score" {
                            let score: i64 =
                                match text.value.clone().unwrap_or(String::new()).parse() {
                                    Ok(num) => num,
                                    Err(_) => {
                                        inter
                                            .create_response(
                                                &ctx.http,
                                                CreateInteractionResponse::Message(
                                                    CreateInteractionResponseMessage::new()
                                                        .content(get_string(
                                                            "member-changer-score-parse-error",
                                                            None,
                                                        )),
                                                ),
                                            )
                                            .await
                                            .unwrap();
                                        return;
                                    }
                                };

                            member.change_score(score - member.score).await;
                        }
                    }
                    _ => (),
                }
            }
        }

        inter
            .create_response(
                &ctx.http,
                CreateInteractionResponse::Message(
                    CreateInteractionResponseMessage::new()
                        .embed(member.to_embed(&ctx, true).await)
                        .ephemeral(true),
                ),
            )
            .await
            .unwrap();
    }

    #[listen_modal("member-changer:own-folder")]
    async fn folder_modal_submit(ctx: &Context, inter: ModalInteraction) {
        let mut mem_man = member::MEMBERSMANAGER.write().await;
        let member = mem_man.get_mut(inter.user.id).await.unwrap();

        for row in inter.data.components.iter() {
            for comp in row.components.iter() {
                match comp {
                    ActionRowComponent::InputText(text) => {
                        if text.custom_id == "member-changer:own-folder:folder" {
                            let score: Option<String> = text.value.clone();

                            member.change_folder(score).await;
                        }
                    }
                    _ => (),
                }
            }
        }

        inter
            .create_response(
                &ctx.http,
                CreateInteractionResponse::Message(
                    CreateInteractionResponseMessage::new()
                        .embed(member.to_embed(&ctx, true).await)
                        .ephemeral(true),
                ),
            )
            .await
            .unwrap();
    }
}
