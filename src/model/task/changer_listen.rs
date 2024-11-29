use crate::prelude::*;
use serenity::{
    self,
    all::{
        ActionRowComponent, ComponentInteractionDataKind, CreateActionRow, CreateInputText,
        CreateModal,
    },
};

pub async fn task_changer_listener() {
    #[listen_component("task-changer")]
    async fn changer(ctx: &Context, inter: ComponentInteraction) {
        let mut mem_man = member::MEMBERSMANAGER.write().await;
        let task_man = task::TASKMANAGER.read().await;

        if let Some(task) = task_man.get(
            mem_man
                .get(inter.user.id)
                .await
                .unwrap()
                .changed_task
                .unwrap_or(0),
        ) {
            inter
                .edit_response(
                    &ctx.http,
                    EditInteractionResponse::new().components(task.main_changer().await),
                )
                .await
                .unwrap();
        }
    }

    #[listen_component("task-changer:score")]
    async fn score_response(ctx: &Context, inter: ComponentInteraction) {
        let mut mem_man = member::MEMBERSMANAGER.write().await;
        let member = mem_man.get(inter.user.id).await.unwrap();
        let task_man = task::TASKMANAGER.read().await;

        if let Some(task) = task_man.get(member.changed_task.unwrap_or(0)) {
            inter
                .create_response(
                    &ctx.http,
                    CreateInteractionResponse::Modal(
                        CreateModal::new(
                            "task-changer:score",
                            get_string("task-changer-score-modal-label", None),
                        )
                        .components(Vec::from([
                            CreateActionRow::InputText(
                                CreateInputText::new(
                                    serenity::all::InputTextStyle::Short,
                                    get_string("task-changer-score-input-label", None),
                                    "task-changer:score:input",
                                )
                                .value(task.score.get().to_string()),
                            ),
                        ])),
                    ),
                )
                .await
                .unwrap();
        }
    }

    #[listen_modal("task-changer:score")]
    async fn score_submit(ctx: &Context, inter: ModalInteraction) {
        inter.defer_ephemeral(&ctx.http).await.unwrap();

        let mut mem_man = member::MEMBERSMANAGER.write().await;
        let member = mem_man.get(inter.user.id).await.unwrap();
        let mut task_man = task::TASKMANAGER.write().await;

        if let Some(task) = task_man.get_mut(member.changed_task.unwrap_or(0)) {
            for row in inter.data.components.iter() {
                for comp in row.components.iter() {
                    match comp {
                        ActionRowComponent::InputText(text) => {
                            if text.custom_id == "task-changer:score:input" {
                                let score: i64 =
                                    match text.value.clone().unwrap_or(String::new()).parse() {
                                        Ok(num) => num,
                                        Err(_) => {
                                            inter
                                                .edit_response(
                                                    &ctx.http,
                                                    EditInteractionResponse::new().content(
                                                        get_string(
                                                            "task-changer-score-parse-error",
                                                            None,
                                                        ),
                                                    ),
                                                )
                                                .await
                                                .unwrap();
                                            return;
                                        }
                                    };

                                task.set_score(&ctx, score).await;
                            }
                        }
                        _ => (),
                    }
                }
            }

            inter
                .edit_response(
                    &ctx.http,
                    EditInteractionResponse::new().embed(task.to_embed()),
                )
                .await
                .unwrap();
        }
    }

    #[listen_component("task-changer:max-members")]
    async fn max_members_response(ctx: &Context, inter: ComponentInteraction) {
        let mut mem_man = member::MEMBERSMANAGER.write().await;
        let member = mem_man.get(inter.user.id).await.unwrap();
        let task_man = task::TASKMANAGER.read().await;

        if let Some(task) = task_man.get(member.changed_task.unwrap_or(0)) {
            inter
                .create_response(
                    &ctx.http,
                    CreateInteractionResponse::Modal(
                        CreateModal::new(
                            "task-changer:max-members",
                            get_string("task-changer-max-members-modal-label", None),
                        )
                        .components(Vec::from([
                            CreateActionRow::InputText(
                                CreateInputText::new(
                                    serenity::all::InputTextStyle::Short,
                                    get_string("task-changer-max-members-input-label", None),
                                    "task-changer:max-member:input",
                                )
                                .value(task.score.get().to_string()),
                            ),
                        ])),
                    ),
                )
                .await
                .unwrap();
        }
    }

    #[listen_modal("task-changer:max-members")]
    async fn max_members_submit(ctx: &Context, inter: ModalInteraction) {
        inter.defer_ephemeral(&ctx.http).await.unwrap();

        let mut mem_man = member::MEMBERSMANAGER.write().await;
        let member = mem_man.get(inter.user.id).await.unwrap();
        let mut task_man = task::TASKMANAGER.write().await;

        if let Some(task) = task_man.get_mut(member.changed_task.unwrap_or(0)) {
            for row in inter.data.components.iter() {
                for comp in row.components.iter() {
                    match comp {
                        ActionRowComponent::InputText(text) => {
                            if text.custom_id == "task-changer:max-members:input" {
                                let max_members: u32 =
                                    match text.value.clone().unwrap_or(String::new()).parse() {
                                        Ok(num) => num,
                                        Err(_) => {
                                            inter
                                                .edit_response(
                                                    &ctx.http,
                                                    EditInteractionResponse::new().content(
                                                        get_string(
                                                            "task-changer-max-members-parse-error",
                                                            None,
                                                        ),
                                                    ),
                                                )
                                                .await
                                                .unwrap();
                                            return;
                                        }
                                    };

                                task.set_max_members(&ctx, max_members).await;
                            }
                        }
                        _ => (),
                    }
                }
            }

            inter
                .edit_response(
                    &ctx.http,
                    EditInteractionResponse::new().embed(task.to_embed()),
                )
                .await
                .unwrap();
        }
    }

    // TODO
    #[listen_component("task-changer:close")]
    async fn members_response(ctx: &Context, inter: ComponentInteraction) {}

    #[listen_component("task-changer:open")]
    async fn members_response(ctx: &Context, inter: ComponentInteraction) {}

    #[listen_component("task-changer:members")]
    async fn members_response(ctx: &Context, inter: ComponentInteraction) {}

    #[listen_component("task-changer:mentor")]
    async fn mentor_response(ctx: &Context, inter: ComponentInteraction) {
        inter.defer_ephemeral(&ctx.http).await.unwrap();

        let mut mem_man = member::MEMBERSMANAGER.write().await;
        let member = mem_man.get(inter.user.id).await.unwrap();
        let mut task_man = task::TASKMANAGER.write().await;

        if let Some(task) = task_man.get_mut(member.changed_task.unwrap_or(0)) {
            if let ComponentInteractionDataKind::UserSelect { ref values } = inter.data.kind {
                drop(mem_man);

                if values.is_empty() {
                    task.set_mentor(&ctx, None, true).await;
                }

                for value in values {
                    task.set_mentor(&ctx, Some(value.clone()), true).await;
                }
            }

            inter
                .edit_response(
                    &ctx.http,
                    EditInteractionResponse::new().embed(task.to_embed()),
                )
                .await
                .unwrap();
        }
    }
}
