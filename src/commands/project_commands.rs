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
}
