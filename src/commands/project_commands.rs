use crate::{model::project::PROJECTMANAGER, prelude::*};
use serenity;

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
                        EditInteractionResponse::new()
                            .content(get_string("command-done-respose", None)),
                    )
                    .await
                    .unwrap();
            }
            Err(e) => {
                inter
                    .edit_response(
                        &ctx.http,
                        EditInteractionResponse::new()
                            .content(get_string("command-create-project-error", None)),
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

    #[slash_command([
        mode = [
            choice = locale
        ]
    ])]
    async fn change_project_roles(
        ctx: &Context,
        inter: CommandInteraction,
        project_name: String,
        mode: String,
        role: Role,
    ) {
        inter.defer_ephemeral(&ctx.http).await.unwrap();
        let mut proj_man = PROJECTMANAGER.try_write().unwrap();
        let project = proj_man
            .get_mut(&project_name)
            .expect(&format!("project with name {} not found", &project_name));

        match mode.as_str() {
            "change-project-role-command-param-mode-choice-add" => {
                project.add_role(role.id).await;
            }
            "change-project-role-command-param-mode-choice-remove" => {
                project.remove_role(role.id).await;
            }
            _ => panic!("unknown mode choice: {}", mode),
        }

        inter
            .edit_response(
                &ctx.http,
                EditInteractionResponse::new().content(get_string("command-done-respose", None)),
            )
            .await
            .unwrap();
    }
}
