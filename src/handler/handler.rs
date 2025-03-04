use crate::{commands::*, config::CONFIG, prelude::*, shop};
use serenity::{
    all::{async_trait, ForumEmoji, Reaction, ReactionType},
    client::{Context, EventHandler},
    http::Http,
    model::{application::Interaction, gateway::Ready, id::GuildId},
};
use std::sync::Arc;

pub struct Handler;

#[async_trait]
impl EventHandler for Handler {
    #[allow(unused_variables)]
    async fn ready(&self, ctx: Context, data_about_bot: Ready) {
        let cfg = CONFIG.try_read().unwrap();
        if let Some(channel) = cfg.log {
            Logger::set_log_channel(&ctx, &channel).await;
        }

        if let Some((notify_type, id)) = cfg.notify_on.clone() {
            Logger::set_notify(&ctx, notify_type, id).await;
        }
        drop(cfg);

        let guild_id = GuildId::new(CONFIG.try_read().unwrap().guild);

        fun_commands(&ctx, guild_id).await;
        debug_commands(&ctx, guild_id).await;
        shop_commands(&ctx, guild_id).await;
        member_commands(&ctx, guild_id).await;
        project_commands(&ctx, guild_id).await;
        task_commands(&ctx, guild_id).await;
        save_commands(&ctx, guild_id).await;
        tag_commands(&ctx, guild_id).await;
        config_commands(&ctx, guild_id).await;
        role_commands(&ctx, guild_id).await;

        sync_guild_commands(&ctx.http, &guild_id).await;

        shop::shop_component_listeners().await;
        member::member_changer_listener().await;
        task::task_changer_listener().await;
        project::project_listen().await;
        tag::tag_changer_listener().await;

        project::ProjectManager::start_update_stat(ctx).await;

        Logger::low("handler.ready", "bot is ready").await;
    }

    async fn interaction_create(&self, ctx: Context, interaction: Interaction) {
        let command_man = match COMMANDMANAGER.try_read() {
            Ok(man) => man,
            Err(_) => {
                Logger::error(
                    "handler.interaction_create",
                    "error while try_read COMMANDMANAGER, maybe deadlock, trying await...",
                )
                .await;
                COMMANDMANAGER.read().await
            }
        };

        match interaction {
            Interaction::Command(ref command) => {
                command_man
                    .call_command(&command.data.name, command, Arc::new(ctx))
                    .await;
            }
            Interaction::Component(ref component) => {
                command_man
                    .call_component(&component.data.custom_id, component, Arc::new(ctx))
                    .await;
            }
            Interaction::Modal(ref modal) => {
                command_man
                    .call_modal(&modal.data.custom_id, modal, Arc::new(ctx))
                    .await;
            }
            _ => (),
        }
    }

    async fn thread_create(&self, ctx: Context, thread: GuildChannel) {
        let proj_man = project::PROJECTMANAGER.read().await;
        let mut task_man = task::TASKMANAGER.write().await;
        let mut thread = thread;

        if let Some(_) = task_man.get_thread(thread.id) {
            return;
        }

        if let Some(ref parent) = thread.parent_id {
            if let Some(project) = proj_man.get_from_forum(parent) {
                match task_man
                    .new_task(
                        &ctx,
                        &mut thread,
                        project.name().clone(),
                        project.waiter_role.clone(),
                    )
                    .await
                {
                    Ok(_) => (),
                    Err(e) => {
                        Logger::error(
                            "handler.thread_create",
                            &format!(
                            "error while creating task from thread \"{}\" for project \"{}\": {}",
                            thread.name,
                            project.name(),
                            e
                        ),
                        )
                        .await
                    }
                }
            }
        }
    }

    #[allow(unused_variables)]
    async fn thread_update(&self, ctx: Context, old: Option<GuildChannel>, new: GuildChannel) {
        if let Some(old_channel) = old {
            let old_id: Vec<u64> = old_channel.applied_tags.iter().map(|x| x.get()).collect();
            let new_id: Vec<u64> = new.applied_tags.iter().map(|x| x.get()).collect();

            if old_id != new_id {
                let mut task_man = task::TASKMANAGER.write().await;
                if let Some(task) = task_man.get_thread_mut(new.id) {
                    task.fetch_tags(&new).await;
                }
            }
        }
    }

    async fn guild_member_addition(&self, ctx: Context, new_member: Member) {
        if let Some(guest_role) = CONFIG.read().await.guest_role {
            if let Err(e) = new_member.add_role(&ctx.http, guest_role).await {
                Logger::error(
                    "handler.guild_member_addition",
                    &format!(
                        "cannot give role {} for new guild member {} ({}): {}",
                        guest_role.get(),
                        new_member.display_name(),
                        new_member.user.id.get(),
                        e.to_string()
                    ),
                )
                .await;
            }
        }
    }

    #[allow(unused_variables)]
    async fn guild_member_removal(
        &self,
        ctx: Context,
        guild_id: GuildId,
        user: User,
        member_data_if_avaliable: Option<Member>,
    ) {
        let mut mem_man = member::MEMBERSMANAGER.write().await;
        let mut task_man = task::TASKMANAGER.write().await;

        if let Ok(member) = mem_man.get_mut(user.id).await {
            for (_, tasks) in member.in_tasks.iter() {
                for id in tasks.iter() {
                    if let Some(task) = task_man.get_mut(*id) {
                        task.remove_member(&ctx, user.id).await;
                    }
                }
            }
        }
    }

    async fn reaction_add(&self, ctx: Context, add_reaction: Reaction) {
        let Some(user) = add_reaction.user_id else {
            return;
        };
        let Ok(thread) = fetch_thread(&ctx, add_reaction.channel_id) else {
            return;
        };
        let Some(parent_id) = thread.parent_id else {
            return;
        };
        let Ok(parent) = fetch_channel(&ctx, parent_id) else {
            return;
        };
        let Some(default_react) = parent.default_reaction_emoji else {
            return;
        };

        let mut task_man = task::TASKMANAGER.write().await;
        let Some(task) = task_man.get_thread_mut(add_reaction.channel_id) else {
            return;
        };

        let is_matching_reaction = match (default_react, &add_reaction.emoji) {
            (ForumEmoji::Id(emoji_id), ReactionType::Custom { id, .. }) => emoji_id == *id,
            (ForumEmoji::Name(emoji_name), ReactionType::Unicode(unicode)) => {
                emoji_name == *unicode
            }
            _ => false,
        };

        if is_matching_reaction {
            if !task.add_member(&ctx, user, false).await {
                if let Err(e) = add_reaction.delete(&ctx.http).await {
                    Logger::error(
                        "handler.reaction_add",
                        &format!(
                            "cannot delete reaction on task \"{}\": {}",
                            task.name.get(),
                            e
                        ),
                    )
                    .await;
                }
            }
        }
    }
}

async fn sync_guild_commands(http: &Http, guild_id: &GuildId) {
    match http.get_guild_commands(guild_id.clone()).await {
        Ok(commands) => {
            let commands_man = COMMANDMANAGER.read().await;

            for command in commands {
                if !commands_man.contains_command(&command.name) {
                    if Logger::if_ok(
                        "handler.sync_guild_commands",
                        &format!("error while clear guild command {}", command.name),
                        http.delete_guild_command(guild_id.clone(), command.id)
                            .await,
                    )
                    .await
                    {
                        Logger::debug(
                            "handler.sync_guild_commands",
                            &format!("deleted interaction command: {}", command.name),
                        )
                        .await;
                    }
                }
            }

            Logger::debug(
                "handler.sync_guild_commands",
                "all commands synchronized successfully",
            )
            .await;
        }
        Err(e) => {
            Logger::error(
                "handler.sync_guild_commands",
                &format!("error while getting guild commands: {}", e.to_string()),
            )
            .await;
        }
    }
}
