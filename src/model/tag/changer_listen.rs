use crate::prelude::*;
use serenity::{
    self,
    all::{
        ActionRowComponent, ComponentInteractionDataKind, CreateActionRow, CreateInputText,
        CreateModal,
    },
};
use tag::TageTypes;

pub async fn tag_changer_listener() {
    #[listen_component("tag-changer:tag-type")]
    async fn tag_type_response(ctx: &Context, inter: ComponentInteraction) {
        let mut tag_man = tag::TAGSMANAGER.write().await;
        let mut mem_man = member::MEMBERSMANAGER.write().await;
        let member = mem_man.get(inter.user.id).await.unwrap();

        if let Some(tag) = tag_man.get_mut(&member.changed_tag.unwrap()) {
            if let ComponentInteractionDataKind::StringSelect { values } = &inter.data.kind {
                if values.is_empty() {
                    tag.set_tag_type(None).await;
                }

                for value in values.iter() {
                    tag.set_tag_type(match value.as_str() {
                        "base" => Some(TageTypes::Base),
                        "closedtask" => Some(TageTypes::ClosedTask),
                        "inwork" => Some(TageTypes::InWork),
                        _ => None,
                    })
                    .await;
                }
            }

            inter
                .create_response(
                    &ctx.http,
                    CreateInteractionResponse::UpdateMessage(
                        CreateInteractionResponseMessage::new(),
                    ),
                )
                .await
                .unwrap();
        }
    }

    #[listen_component("tag-changer:ping-role")]
    async fn ping_role_response(ctx: &Context, inter: ComponentInteraction) {
        let mut tag_man = tag::TAGSMANAGER.write().await;
        let mut mem_man = member::MEMBERSMANAGER.write().await;
        let member = mem_man.get(inter.user.id).await.unwrap();

        if let Some(tag) = tag_man.get_mut(&member.changed_tag.unwrap()) {
            if let ComponentInteractionDataKind::RoleSelect { values } = &inter.data.kind {
                if values.is_empty() {
                    tag.set_ping_role(None).await;
                }

                for value in values.iter() {
                    tag.set_ping_role(Some(value.clone())).await;
                }
            }

            inter
                .create_response(
                    &ctx.http,
                    CreateInteractionResponse::UpdateMessage(
                        CreateInteractionResponseMessage::new(),
                    ),
                )
                .await
                .unwrap();
        }
    }

    #[listen_component("tag-changer:task-project")]
    async fn task_project_response(ctx: &Context, inter: ComponentInteraction) {
        let mut tag_man = tag::TAGSMANAGER.write().await;
        let mut mem_man = member::MEMBERSMANAGER.write().await;
        let member = mem_man.get(inter.user.id).await.unwrap();

        if let Some(tag) = tag_man.get_mut(&member.changed_tag.unwrap()) {
            if let ComponentInteractionDataKind::StringSelect { values } = &inter.data.kind {
                if values.is_empty() {
                    tag.set_task_project(None).await;
                }

                for value in values.iter() {
                    tag.set_task_project(Some(value.clone())).await;
                }
            }

            inter
                .create_response(
                    &ctx.http,
                    CreateInteractionResponse::UpdateMessage(
                        CreateInteractionResponseMessage::new(),
                    ),
                )
                .await
                .unwrap();
        }
    }

    #[listen_component("tag-changer:max-members")]
    async fn max_members_response(ctx: &Context, inter: ComponentInteraction) {
        let mut tag_man = tag::TAGSMANAGER.write().await;
        let mut mem_man = member::MEMBERSMANAGER.write().await;
        let member = mem_man.get(inter.user.id).await.unwrap();

        if let Some(tag) = tag_man.get_mut(&member.changed_tag.unwrap()) {
            inter
                .create_response(
                    &ctx.http,
                    CreateInteractionResponse::Modal(
                        CreateModal::new(
                            "tag-changer:max-members",
                            loc!("tag-changer-max-members-modal-title"),
                        )
                        .components(Vec::from([
                            CreateActionRow::InputText(
                                CreateInputText::new(
                                    serenity::all::InputTextStyle::Short,
                                    loc!("tag-changer-max-members-input-label"),
                                    "tag-changer:max-members:input",
                                )
                                .value(match tag.max_members {
                                    Some(num) => num.to_string(),
                                    None => String::new(),
                                }),
                            ),
                        ])),
                    ),
                )
                .await
                .unwrap();
        }
    }

    #[listen_component("tag-changer:score-modifier")]
    async fn score_modifier_response(ctx: &Context, inter: ComponentInteraction) {
        let mut tag_man = tag::TAGSMANAGER.write().await;
        let mut mem_man = member::MEMBERSMANAGER.write().await;
        let member = mem_man.get(inter.user.id).await.unwrap();

        if let Some(tag) = tag_man.get_mut(&member.changed_tag.unwrap()) {
            inter
                .create_response(
                    &ctx.http,
                    CreateInteractionResponse::Modal(
                        CreateModal::new(
                            "tag-changer:score-modifier",
                            loc!("tag-changer-score-modifier-modal-title"),
                        )
                        .components(Vec::from([
                            CreateActionRow::InputText(
                                CreateInputText::new(
                                    serenity::all::InputTextStyle::Short,
                                    loc!("tag-changer-score-modifier-input-label"),
                                    "tag-changer:score-modifier:input",
                                )
                                .value(match tag.max_members {
                                    Some(num) => num.to_string(),
                                    None => String::new(),
                                }),
                            ),
                        ])),
                    ),
                )
                .await
                .unwrap();
        }
    }

    #[listen_modal("tag-changer:max-members")]
    async fn max_members_submit(ctx: &Context, inter: ModalInteraction) {
        inter.defer_ephemeral(&ctx.http).await.unwrap();

        let mut tag_man = tag::TAGSMANAGER.write().await;
        let mut mem_man = member::MEMBERSMANAGER.write().await;
        let member = mem_man.get(inter.user.id).await.unwrap();

        if let Some(tag) = tag_man.get_mut(&member.changed_tag.unwrap()) {
            for row in inter.data.components.iter() {
                for comp in row.components.iter() {
                    match comp {
                        ActionRowComponent::InputText(text) => {
                            if text.custom_id == "tag-changer:max-members:input" {
                                let max_members = match text
                                    .value
                                    .clone()
                                    .unwrap_or(String::new())
                                    .parse::<u32>()
                                {
                                    Ok(num) => Some(num),
                                    Err(_) => None,
                                };

                                tag.set_max_members(max_members).await;
                            }
                        }
                        _ => (),
                    }
                }
            }

            inter
                .edit_response(
                    &ctx.http,
                    EditInteractionResponse::new().embed(tag.to_embed()),
                )
                .await
                .unwrap();
        }
    }

    #[listen_modal("tag-changer:score-modifier")]
    async fn score_modifier_submit(ctx: &Context, inter: ModalInteraction) {
        inter.defer_ephemeral(&ctx.http).await.unwrap();

        let mut tag_man = tag::TAGSMANAGER.write().await;
        let mut mem_man = member::MEMBERSMANAGER.write().await;
        let member = mem_man.get(inter.user.id).await.unwrap();

        if let Some(tag) = tag_man.get_mut(&member.changed_tag.unwrap()) {
            for row in inter.data.components.iter() {
                for comp in row.components.iter() {
                    match comp {
                        ActionRowComponent::InputText(text) => {
                            if text.custom_id == "tag-changer:score-modifier:input" {
                                let score_modifier = match text
                                    .value
                                    .clone()
                                    .unwrap_or(String::new())
                                    .parse::<i64>()
                                {
                                    Ok(num) => Some(num),
                                    Err(_) => None,
                                };

                                tag.set_score_modifier(score_modifier).await;
                            }
                        }
                        _ => (),
                    }
                }
            }

            inter
                .edit_response(
                    &ctx.http,
                    EditInteractionResponse::new().embed(tag.to_embed()),
                )
                .await
                .unwrap();
        }
    }
}
