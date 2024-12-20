use crate::prelude::*;
use serenity::{
    self,
    all::{
        ActionRowComponent, ComponentInteractionDataKind, CreateActionRow, CreateInputText,
        CreateModal,
    },
};

pub async fn project_listen() {
    #[listen_component("project-changer:max-tasks-per-user")]
    async fn max_users_response(ctx: &Context, inter: ComponentInteraction) {
        let mut men_man = member::MEMBERSMANAGER.write().await;
        let proj_man = project::PROJECTMANAGER.read().await;
        let member = men_man.get(inter.user.id).await.unwrap();

        if let Some(project) = proj_man.get(&member.changed_project.clone().unwrap()) {
            inter
                .create_response(
                    &ctx.http,
                    CreateInteractionResponse::Modal(
                        CreateModal::new(
                            "project-changer:max-tasks-per-user",
                            get_string("project-changer-max-tasks-per-user-modal-title", None),
                        )
                        .components(Vec::from([
                            CreateActionRow::InputText(
                                CreateInputText::new(
                                    serenity::all::InputTextStyle::Short,
                                    get_string(
                                        "project-changer-max-tasks-per-user-input-label",
                                        None,
                                    ),
                                    "project-changer:max-tasks-per-user:input",
                                )
                                .value(project.max_tasks_per_user.to_string()),
                            ),
                        ])),
                    ),
                )
                .await
                .unwrap();
        }
    }

    #[listen_modal("project-changer:max-tasks-per-user")]
    async fn max_task_submit(ctx: &Context, inter: ModalInteraction) {
        inter.defer_ephemeral(&ctx.http).await.unwrap();

        let mut proj_man = project::PROJECTMANAGER.write().await;
        let mut mem_man = member::MEMBERSMANAGER.write().await;

        if let Some(project) = proj_man.get_mut(
            &mem_man
                .get(inter.user.id)
                .await
                .unwrap()
                .changed_project
                .clone()
                .unwrap(),
        ) {
            for row in inter.data.components.iter() {
                for comp in row.components.iter() {
                    match comp {
                        ActionRowComponent::InputText(text) => {
                            let max_tasks: u32 =
                                match text.value.clone().unwrap_or(String::new()).parse() {
                                    Ok(num) => num,
                                    Err(_) => {
                                        inter
                                    .create_response(
                                        &ctx.http,
                                        CreateInteractionResponse::Message(
                                            CreateInteractionResponseMessage::new()
                                                .content(get_string(
                                                "project-changer-max-tasks-per-user-parse-error",
                                                None,
                                            )),
                                        ),
                                    )
                                    .await
                                    .unwrap();
                                        return;
                                    }
                                };

                            project.set_max_task_per_user(max_tasks).await;
                        }
                        _ => (),
                    }
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

    #[listen_component("project-changer:tasks-forum")]
    async fn task_forum_response(ctx: &Context, inter: ComponentInteraction) {
        let mut proj_man = project::PROJECTMANAGER.write().await;
        let mut mem_man = member::MEMBERSMANAGER.write().await;

        if let Some(project) = proj_man.get_mut(
            &mem_man
                .get(inter.user.id)
                .await
                .unwrap()
                .changed_project
                .clone()
                .unwrap(),
        ) {
            if let ComponentInteractionDataKind::ChannelSelect { values } = &inter.data.kind {
                for value in values.iter() {
                    project.set_tasks_forum(value.clone()).await;
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

    #[listen_component("project-changer:waiter-role")]
    async fn waiter_role_response(ctx: &Context, inter: ComponentInteraction) {
        inter.defer_ephemeral(&ctx.http).await.unwrap();

        let mut proj_man = project::PROJECTMANAGER.write().await;
        let mut mem_man = member::MEMBERSMANAGER.write().await;

        if let Some(project) = proj_man.get_mut(
            &mem_man
                .get(inter.user.id)
                .await
                .unwrap()
                .changed_project
                .clone()
                .unwrap(),
        ) {
            if let ComponentInteractionDataKind::RoleSelect { values } = &inter.data.kind {
                if values.is_empty() {
                    project.set_waiter_role(None).await;
                }

                for value in values.iter() {
                    project.set_waiter_role(Some(value.clone())).await;
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

    #[listen_component("project-changer:stat-channel")]
    async fn stat_channel_response(ctx: &Context, inter: ComponentInteraction) {
        inter.defer_ephemeral(&ctx.http).await.unwrap();

        let mut proj_man = project::PROJECTMANAGER.write().await;
        let mut mem_man = member::MEMBERSMANAGER.write().await;

        if let Some(project) = proj_man.get_mut(
            &mem_man
                .get(inter.user.id)
                .await
                .unwrap()
                .changed_project
                .clone()
                .unwrap(),
        ) {
            if let ComponentInteractionDataKind::ChannelSelect { values } = &inter.data.kind {
                if values.is_empty() {
                    project.set_stat_channel(None).await;
                }

                for value in values.iter() {
                    project.set_stat_channel(Some(value.clone())).await;
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

    #[listen_component("project-changer:associated-roles")]
    async fn associated_roles_response(ctx: &Context, inter: ComponentInteraction) {
        let mut proj_man = project::PROJECTMANAGER.write().await;
        let mut mem_man = member::MEMBERSMANAGER.write().await;

        if let Some(project) = proj_man.get_mut(
            &mem_man
                .get(inter.user.id)
                .await
                .unwrap()
                .changed_project
                .clone()
                .unwrap(),
        ) {
            if let ComponentInteractionDataKind::RoleSelect { values } = &inter.data.kind {
                for value in values.iter() {
                    if !project.associated_roles.contains(&value) {
                        project.add_role(value.clone()).await;
                    }
                }

                for role in project.associated_roles.clone() {
                    if !values.contains(&role) {
                        project.remove_role(role).await;
                    }
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
}
