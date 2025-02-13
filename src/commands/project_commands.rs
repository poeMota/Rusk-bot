use crate::{model::project::PROJECTMANAGER, prelude::*};
use serenity::{
    self,
    all::{Colour, CreateEmbed},
};

pub async fn project_commands(ctx: &Context, guild: GuildId) {
    #[slash_command([])]
    async fn create_project(
        ctx: &Context,
        inter: CommandInteraction,
        name: String,
        max_tasks_per_user: i64,
        tasks_forum: PartialChannel,
        waiter_role: Option<Role>,
        stat_channel: Option<PartialChannel>,
    ) {
        inter.defer_ephemeral(&ctx.http).await.unwrap();
        let mut proj_man = PROJECTMANAGER.try_write().unwrap();

        match proj_man
            .new_project(
                name,
                max_tasks_per_user as u32,
                tasks_forum.id,
                match waiter_role {
                    Some(role) => Some(role.id),
                    None => None,
                },
                match stat_channel {
                    Some(channel) => Some(channel.id),
                    None => None,
                },
            )
            .await
        {
            Ok(_) => {
                inter
                    .edit_response(
                        &ctx.http,
                        EditInteractionResponse::new().content(loc!("command-done-response")),
                    )
                    .await
                    .unwrap();
            }
            Err(e) => {
                inter
                    .edit_response(
                        &ctx.http,
                        EditInteractionResponse::new()
                            .content(loc!("command-create-project-error")),
                    )
                    .await
                    .unwrap();
                Logger::error(
                    "commands.create_project",
                    &format!("cannot create project, {}", e.to_string()),
                )
                .await;
            }
        };
    }

    #[slash_command([])]
    async fn change_project(ctx: &Context, inter: CommandInteraction, project_name: String) {
        inter.defer_ephemeral(&ctx.http).await.unwrap();
        let mut mem_man = member::MEMBERSMANAGER.write().await;
        let proj_man = project::PROJECTMANAGER.read().await;

        if let Some(proj) = proj_man.get(&project_name) {
            mem_man
                .get_mut(inter.user.id.clone())
                .await
                .unwrap()
                .changed_project = Some(proj.name().clone());

            inter
                .edit_response(
                    &ctx.http,
                    EditInteractionResponse::new()
                        .embed(
                            CreateEmbed::new()
                                .title(loc!("project-changer-embed-title"))
                                .description(loc!(
                                    "project-changer-embed-description",
                                    "project" = proj.name()
                                ))
                                .color(Colour::MAGENTA),
                        )
                        .components(proj.main_changer().await),
                )
                .await
                .unwrap();
        } else {
            inter
                .edit_response(
                    &ctx.http,
                    EditInteractionResponse::new().content(loc!("project-not-found")),
                )
                .await
                .unwrap();
        }
    }

    #[slash_command([])]
    async fn project_config(ctx: &Context, inter: CommandInteraction, project_name: String) {
        inter.defer_ephemeral(&ctx.http).await.unwrap();

        let proj_man = project::PROJECTMANAGER.read().await;

        if let Some(project) = proj_man.get(&project_name) {
            inter
                .edit_response(
                    &ctx.http,
                    EditInteractionResponse::new().embed(project.to_embed().await),
                )
                .await
                .unwrap();
        } else {
            inter
                .edit_response(
                    &ctx.http,
                    EditInteractionResponse::new().content(loc!("project-not-found")),
                )
                .await
                .unwrap();
        }
    }

    #[slash_command([])]
    async fn delete_project(ctx: &Context, inter: CommandInteraction, project_name: String) {
        inter.defer_ephemeral(&ctx.http).await.unwrap();

        let mut proj_man = project::PROJECTMANAGER.write().await;
        if let Some(_) = proj_man.delete(&project_name).await {
            inter
                .edit_response(
                    &ctx.http,
                    EditInteractionResponse::new().content(loc!("command-done-response")),
                )
                .await
                .unwrap();
        } else {
            inter
                .edit_response(
                    &ctx.http,
                    EditInteractionResponse::new().content(loc!("project-not-found")),
                )
                .await
                .unwrap();
        }
    }
}
