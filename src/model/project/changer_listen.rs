use crate::prelude::*;
use serenity::{
    self,
    all::{ActionRowComponent, CreateActionRow, CreateInputText, CreateModal},
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
                    CreateInteractionResponse::Message(
                        CreateInteractionResponseMessage::new()
                            .embed(project.to_embed().await)
                            .ephemeral(true),
                    ),
                )
                .await
                .unwrap();
        }
    }
}
