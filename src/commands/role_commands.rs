use serenity::all::{ComponentInteractionDataKind, CreateActionRow, CreateSelectMenu, RoleId};

use crate::prelude::*;

pub async fn role_commands(ctx: &Context, guild: GuildId) {
    #[slash_command([])]
    async fn change_member_roles(ctx: &Context, inter: CommandInteraction, user: User) {
        let dis_member = match fetch_member(&user.id).await {
            Ok(m) => m,
            Err(_) => {
                inter
                    .create_response(
                        &ctx.http,
                        CreateInteractionResponse::Message(
                            CreateInteractionResponseMessage::new()
                                .content(loc!("change-member-roles-command-member-not-found"))
                                .ephemeral(true),
                        ),
                    )
                    .await
                    .unwrap();
                return;
            }
        };

        let mut mem_man = member::MEMBERSMANAGER.write().await;
        mem_man.get_mut(inter.user.id).await.unwrap().changed_member = Some(user.id);

        inter
            .create_response(
                &ctx.http,
                CreateInteractionResponse::Message(
                    CreateInteractionResponseMessage::new()
                        .content(loc!("roles-changer-title", "member" = user.id.get()))
                        .components(Vec::from([CreateActionRow::SelectMenu(
                            CreateSelectMenu::new(
                                "roles-changer:member-roles",
                                serenity::all::CreateSelectMenuKind::Role {
                                    default_roles: Some(dis_member.roles),
                                },
                            )
                            .placeholder(loc!("roles-changer-roles-placeholder")),
                        )]))
                        .ephemeral(true),
                ),
            )
            .await
            .unwrap();
    }

    #[listen_component("roles-changer:member-roles")]
    async fn roles_changer_listen(ctx: &Context, inter: ComponentInteraction) {
        let mut mem_man = member::MEMBERSMANAGER.write().await;
        let member = mem_man.get_mut(inter.user.id).await.unwrap();
        let dis_member = fetch_member(&inter.user.id).await.unwrap();

        let changer_member = fetch_member(&member.changed_member.unwrap()).await.unwrap();

        if let ComponentInteractionDataKind::RoleSelect { values } = &inter.data.kind {
            let added = values
                .iter()
                .filter(|x| !changer_member.roles.contains(&x))
                .collect::<Vec<&RoleId>>();

            let removed = changer_member
                .roles
                .iter()
                .filter(|x| !values.contains(&x))
                .collect::<Vec<&RoleId>>();

            let role_man = role::ROLEMANAGER.read().await;

            for role in added
                .iter()
                .filter(|x| role_man.have_permission(&dis_member, ***x))
            {
                changer_member.add_role(&ctx.http, **role).await.unwrap();
            }

            for role in removed
                .iter()
                .filter(|x| role_man.have_permission(&dis_member, ***x))
            {
                changer_member.remove_role(&ctx.http, **role).await.unwrap();
            }
        }

        inter
            .create_response(
                &ctx.http,
                CreateInteractionResponse::UpdateMessage(
                    CreateInteractionResponseMessage::new().components(Vec::from([
                        CreateActionRow::SelectMenu(
                            CreateSelectMenu::new(
                                "roles-changer:member-roles",
                                serenity::all::CreateSelectMenuKind::Role {
                                    default_roles: Some(dis_member.roles),
                                },
                            )
                            .placeholder(loc!("roles-changer-roles-placeholder")),
                        ),
                    ])),
                ),
            )
            .await
            .unwrap();
    }

    #[slash_command([])]
    async fn change_roles_permissions(ctx: &Context, inter: CommandInteraction, role: Role) {
        let mut mem_man = member::MEMBERSMANAGER.write().await;
        mem_man.get_mut(inter.user.id).await.unwrap().changed_role = Some(role.id);
        let role_man = role::ROLEMANAGER.read().await;

        inter
            .create_response(
                &ctx.http,
                CreateInteractionResponse::Message(
                    CreateInteractionResponseMessage::new()
                        .content(loc!(
                            "roles-changer-permissions-title",
                            "role" = role.id.get()
                        ))
                        .components(Vec::from([CreateActionRow::SelectMenu(
                            CreateSelectMenu::new(
                                "roles-changer:role-permissions",
                                serenity::all::CreateSelectMenuKind::Role {
                                    default_roles: role_man.get_permissons(role.id).cloned(),
                                },
                            )
                            .placeholder(loc!("roles-changer-permissions-placeholder")),
                        )]))
                        .ephemeral(true),
                ),
            )
            .await
            .unwrap();
    }

    #[listen_component("roles-changer:role-permissions")]
    async fn change_role_permissions_listen(ctx: &Context, inter: ComponentInteraction) {
        let mut mem_man = member::MEMBERSMANAGER.write().await;
        let member = mem_man.get_mut(inter.user.id).await.unwrap();

        if let ComponentInteractionDataKind::RoleSelect { values } = &inter.data.kind {
            role::ROLEMANAGER
                .write()
                .await
                .set_permissions(member.changed_role.unwrap(), values.clone())
                .await;
        }

        inter
            .create_response(
                &ctx.http,
                CreateInteractionResponse::UpdateMessage(CreateInteractionResponseMessage::new()),
            )
            .await
            .unwrap();
    }
}
