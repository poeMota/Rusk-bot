use crate::{
    connect::*,
    model::{member::MEMBERSMANAGER, role::ROLEMANAGER},
    prelude::*,
};
use serenity::{
    self,
    all::{
        ComponentInteractionDataKind, CreateActionRow, CreateAttachment, CreateSelectMenu,
        CreateSelectMenuOption,
    },
};

pub async fn save_commands(ctx: &Context, guild: GuildId) {
    #[slash_command([])]
    async fn save(ctx: &Context, inter: CommandInteraction, path: Option<String>) {
        inter.defer_ephemeral(&ctx.http).await.unwrap();

        let dis_member = fetch_member(&inter.user.id).await.unwrap();

        let role_man = ROLEMANAGER.read().await;
        let mut mem_man = member::MEMBERSMANAGER.write().await;

        let member = mem_man.get(inter.user.id).await.unwrap();
        let dbs = role_man
            .member_db_permissons(&dis_member)
            .iter()
            .filter(|x| !member.own_folder.get(x.as_str()).unwrap_or(&None).is_none())
            .map(|x| *x)
            .collect::<Vec<&String>>();

        if dbs.is_empty() {
            inter
                .edit_response(
                    &ctx.http,
                    EditInteractionResponse::new().content(loc!("link-folder-command-no-dbs")),
                )
                .await
                .unwrap();

            return;
        }

        if dbs.len() > 1 {
            inter
                .edit_response(
                    &ctx.http,
                    EditInteractionResponse::new()
                        .content(loc!("save-command-db-title"))
                        .components(Vec::from([CreateActionRow::SelectMenu(
                            CreateSelectMenu::new(
                                "save:db",
                                serenity::all::CreateSelectMenuKind::String {
                                    options: dbs
                                        .iter()
                                        .map(|x| {
                                            CreateSelectMenuOption::new(
                                                *x,
                                                format!(
                                                    "{}:::{}/{}",
                                                    x,
                                                    member
                                                        .own_folder
                                                        .get(x.as_str())
                                                        .cloned()
                                                        .unwrap()
                                                        .unwrap(),
                                                    path.clone().unwrap_or(String::from("/"))
                                                ),
                                            )
                                        })
                                        .collect(),
                                },
                            ),
                        )])),
                )
                .await
                .unwrap();

            return;
        }

        let db = *dbs.get(0).unwrap();

        if let Some(Some(folder)) = member.own_folder.get(db.as_str()) {
            if let Some(p) = path {
                if p.ends_with("/") {
                    inter
                        .edit_response(
                            &ctx.http,
                            EditInteractionResponse::new()
                                .content(loc!("unload-save-menu-title"))
                                .components(Vec::from([save_menu_component(
                                    db.clone(),
                                    format!("{}/{}", folder, p),
                                    format!("{}/{}", folder, p),
                                )
                                .await])),
                        )
                        .await
                        .unwrap();
                } else {
                    match unload_content(db.clone(), format!("{}/{}", folder, p)).await {
                        Ok(data) => {
                            inter
                                .edit_response(
                                    &ctx.http,
                                    EditInteractionResponse::new().new_attachment(
                                        CreateAttachment::bytes(
                                            data,
                                            p.split("/").last().unwrap_or("unknown.yml"),
                                        ),
                                    ),
                                )
                                .await
                                .unwrap();

                            Logger::low(
                                "commands.save",
                                &format!(
                                    "unloaded save {}/{} by {} ({}",
                                    folder,
                                    p,
                                    inter.user.display_name(),
                                    inter.user.id.get()
                                ),
                            )
                            .await;
                        }
                        Err(e) => match e {
                            ConnectionError::StatusCodeError(url, error) => {
                                inter
                                    .edit_response(
                                        &ctx.http,
                                        EditInteractionResponse::new()
                                            .content(loc!("invalid-url", "path" = url)),
                                    )
                                    .await
                                    .unwrap();

                                Logger::error(
                                    "commands.save",
                                    &format!(
                                        "status code while unloading save bu url \"{}\": {}",
                                        url,
                                        error.to_string()
                                    ),
                                )
                                .await;
                            }
                            ConnectionError::NotAllowedUrl(_) => {
                                inter
                                    .edit_response(
                                        &ctx.http,
                                        EditInteractionResponse::new()
                                            .content(loc!("not-allowed-url")),
                                    )
                                    .await
                                    .unwrap();
                            }
                            _ => {
                                Logger::error(
                                    "commands.save",
                                    &format!("error while connecting: {:?}", e),
                                )
                                .await;

                                inter
                                    .edit_response(
                                        &ctx.http,
                                        EditInteractionResponse::new()
                                            .content(loc!("save-unload-error")),
                                    )
                                    .await
                                    .unwrap();
                            }
                        },
                    }
                }
            } else {
                inter
                    .edit_response(
                        &ctx.http,
                        EditInteractionResponse::new()
                            .content(loc!("unload-save-menu-title"))
                            .components(Vec::from([save_menu_component(
                                db.clone(),
                                format!("{}/", folder),
                                format!("{}/", folder),
                            )
                            .await])),
                    )
                    .await
                    .unwrap();
            }
        } else {
            inter
                .edit_response(
                    &ctx.http,
                    EditInteractionResponse::new().content(loc!("save-command-folder-not-linked")),
                )
                .await
                .unwrap();
        }
    }

    #[slash_command([])]
    async fn save_plus(ctx: &Context, inter: CommandInteraction, path: Option<String>) {
        inter.defer_ephemeral(&ctx.http).await.unwrap();

        let dis_member = fetch_member(&inter.user.id).await.unwrap();

        let role_man = ROLEMANAGER.read().await;

        let dbs = role_man.member_db_permissons(&dis_member);

        if dbs.is_empty() {
            inter
                .edit_response(
                    &ctx.http,
                    EditInteractionResponse::new().content(loc!("link-folder-command-no-dbs")),
                )
                .await
                .unwrap();

            return;
        }

        if dbs.len() > 1 {
            inter
                .edit_response(
                    &ctx.http,
                    EditInteractionResponse::new()
                        .content(loc!("save-command-db-title"))
                        .components(Vec::from([CreateActionRow::SelectMenu(
                            CreateSelectMenu::new(
                                "save:db",
                                serenity::all::CreateSelectMenuKind::String {
                                    options: dbs
                                        .iter()
                                        .map(|x| {
                                            CreateSelectMenuOption::new(
                                                *x,
                                                format!(
                                                    "{}:::{}",
                                                    x,
                                                    path.clone().unwrap_or(String::from("/"))
                                                ),
                                            )
                                        })
                                        .collect(),
                                },
                            ),
                        )])),
                )
                .await
                .unwrap();

            return;
        }

        let db = *dbs.get(0).unwrap();

        if let Some(p) = path {
            if p.ends_with("/") {
                inter
                    .edit_response(
                        &ctx.http,
                        EditInteractionResponse::new()
                            .content(loc!("unload-save-menu-title"))
                            .components(Vec::from([
                                save_menu_component(db.clone(), p.clone(), p).await
                            ])),
                    )
                    .await
                    .unwrap();
            } else {
                match unload_content(db.clone(), p.clone()).await {
                    Ok(data) => {
                        inter
                            .edit_response(
                                &ctx.http,
                                EditInteractionResponse::new().new_attachment(
                                    CreateAttachment::bytes(
                                        data,
                                        p.split("/").last().unwrap_or("unknown.yml"),
                                    ),
                                ),
                            )
                            .await
                            .unwrap();

                        Logger::low(
                            "commands.save_plus",
                            &format!(
                                "unloaded save {} by {} ({}",
                                p,
                                inter.user.display_name(),
                                inter.user.id.get()
                            ),
                        )
                        .await;
                    }
                    Err(e) => match e {
                        ConnectionError::StatusCodeError(url, error) => {
                            inter
                                .edit_response(
                                    &ctx.http,
                                    EditInteractionResponse::new()
                                        .content(loc!("invalid-url", "path" = url)),
                                )
                                .await
                                .unwrap();

                            Logger::error(
                                "commands.save_plus",
                                &format!(
                                    "status code while unloading save bu url \"{}\": {}",
                                    url,
                                    error.to_string()
                                ),
                            )
                            .await;
                        }
                        ConnectionError::NotAllowedUrl(_) => {
                            inter
                                .edit_response(
                                    &ctx.http,
                                    EditInteractionResponse::new().content(loc!("not-allowed-url")),
                                )
                                .await
                                .unwrap();
                        }
                        _ => {
                            Logger::error(
                                "commands.save_plus",
                                &format!("error while connecting: {:?}", e),
                            )
                            .await;

                            inter
                                .edit_response(
                                    &ctx.http,
                                    EditInteractionResponse::new()
                                        .content(loc!("save-unload-error")),
                                )
                                .await
                                .unwrap();
                        }
                    },
                }
            }
        } else {
            inter
                .edit_response(
                    &ctx.http,
                    EditInteractionResponse::new()
                        .content(loc!("unload-save-menu-title"))
                        .components(Vec::from([save_menu_component(
                            db.clone(),
                            String::new(),
                            String::new(),
                        )
                        .await])),
                )
                .await
                .unwrap();
        }
    }

    #[listen_component("save:db")]
    async fn save_db_component(ctx: &Context, inter: ComponentInteraction) {
        if let ComponentInteractionDataKind::StringSelect { values } = &inter.data.kind {
            let parts = values.first().unwrap().split(":::").collect::<Vec<&str>>();
            let db = parts.get(0).unwrap().to_string();
            let path = parts.get(1).unwrap().to_string();

            if path.ends_with("/") {
                inter
                    .edit_response(
                        &ctx.http,
                        EditInteractionResponse::new()
                            .content(loc!("unload-save-menu-title"))
                            .components(Vec::from([save_menu_component(
                                db.clone(),
                                path.clone(),
                                path,
                            )
                            .await])),
                    )
                    .await
                    .unwrap();
            } else {
                match unload_content(db.clone(), path.clone()).await {
                    Ok(data) => {
                        inter
                            .edit_response(
                                &ctx.http,
                                EditInteractionResponse::new().new_attachment(
                                    CreateAttachment::bytes(
                                        data,
                                        path.split("/").last().unwrap_or("unknown.yml"),
                                    ),
                                ),
                            )
                            .await
                            .unwrap();

                        Logger::low(
                            "commands.save",
                            &format!(
                                "unloaded save {} from {} by {} ({})",
                                path,
                                db,
                                inter.user.display_name(),
                                inter.user.id.get()
                            ),
                        )
                        .await;
                    }
                    Err(e) => match e {
                        ConnectionError::StatusCodeError(url, error) => {
                            inter
                                .edit_response(
                                    &ctx.http,
                                    EditInteractionResponse::new()
                                        .content(loc!("invalid-url", "path" = url)),
                                )
                                .await
                                .unwrap();

                            Logger::error(
                                "commands.save",
                                &format!(
                                    "status code while unloading save bu url \"{}\": {}",
                                    url,
                                    error.to_string()
                                ),
                            )
                            .await;
                        }
                        ConnectionError::NotAllowedUrl(_) => {
                            inter
                                .edit_response(
                                    &ctx.http,
                                    EditInteractionResponse::new().content(loc!("not-allowed-url")),
                                )
                                .await
                                .unwrap();
                        }
                        _ => {
                            Logger::error(
                                "commands.save",
                                &format!("error while connecting: {:?}", e),
                            )
                            .await;

                            inter
                                .edit_response(
                                    &ctx.http,
                                    EditInteractionResponse::new()
                                        .content(loc!("save-unload-error")),
                                )
                                .await
                                .unwrap();
                        }
                    },
                }
            }
        }
    }

    async fn save_menu_component(db: String, path: String, root: String) -> CreateActionRow {
        let mut options = Vec::new();
        let mut index = 0;
        for (filename, date) in file_dates(db.clone(), path.clone()).await.unwrap() {
            if options.len() == 25 {
                break;
            }

            if filename.as_str() == "../" {
                let parent = path.replace(path.split("/").last().unwrap_or(""), "");

                if parent == root {
                    continue;
                }
            }

            options.push(
                CreateSelectMenuOption::new(
                    filename.clone(),
                    format!("{}:::{}:::{}:::{}{}", index, db, root, path, filename),
                )
                .description(date),
            );
            index += 1;
        }

        CreateActionRow::SelectMenu(CreateSelectMenu::new(
            "save-menu-option",
            serenity::all::CreateSelectMenuKind::String { options },
        ))
    }

    #[listen_component("save-menu-option")]
    async fn save_menu_response(ctx: &Context, inter: ComponentInteraction) {
        if let ComponentInteractionDataKind::StringSelect { values } = &inter.data.kind {
            for value in values {
                let db = value
                    .split(":::")
                    .collect::<Vec<&str>>()
                    .get(1)
                    .unwrap()
                    .to_string();

                let root = value
                    .split(":::")
                    .collect::<Vec<&str>>()
                    .get(2)
                    .unwrap()
                    .to_string();

                if value.contains("../") {
                    let mut path = Vec::new();
                    for p in value.split(":::").last().unwrap().to_string().split("/") {
                        if p == ".." {
                            path.pop();
                            break;
                        }

                        path.push(p.to_string());
                    }

                    inter
                        .create_response(
                            &ctx.http,
                            CreateInteractionResponse::UpdateMessage(
                                CreateInteractionResponseMessage::new()
                                    .components(Vec::from([save_menu_component(
                                        db,
                                        format!("{}/", path.join("/")),
                                        root,
                                    )
                                    .await]))
                                    .ephemeral(true),
                            ),
                        )
                        .await
                        .unwrap();
                } else if value.ends_with("/") {
                    inter
                        .create_response(
                            &ctx.http,
                            CreateInteractionResponse::UpdateMessage(
                                CreateInteractionResponseMessage::new()
                                    .components(Vec::from([save_menu_component(
                                        db,
                                        value.split(":::").last().unwrap().to_string(),
                                        root,
                                    )
                                    .await]))
                                    .ephemeral(true),
                            ),
                        )
                        .await
                        .unwrap();
                } else {
                    inter.defer_ephemeral(&ctx.http).await.unwrap();
                    let path = value.split(":::").last().unwrap();

                    inter
                        .edit_response(
                            &ctx.http,
                            EditInteractionResponse::new().new_attachment(CreateAttachment::bytes(
                                unload_content(db.clone(), path.to_string()).await.unwrap(),
                                path.split("/").last().unwrap(),
                            )),
                        )
                        .await
                        .unwrap();

                    Logger::low(
                        "components.save-menu-option",
                        &format!(
                            "unloaded save {} from {} by {} ({}",
                            path,
                            db,
                            inter.user.display_name(),
                            inter.user.id.get()
                        ),
                    )
                    .await;
                }
            }
        }
    }

    #[slash_command([])]
    async fn create_db(
        ctx: &Context,
        inter: CommandInteraction,
        db: String,
        url: String,
        login: Option<String>,
        password: Option<String>,
    ) {
        let db = db.trim().replace(" ", "-");
        let env_content = read_file(&DATA_PATH.join(".env"));

        if env_content.contains(&db) {
            inter
                .create_response(
                    &ctx.http,
                    CreateInteractionResponse::Message(
                        CreateInteractionResponseMessage::new()
                            .content(loc!("created-db-command-exist")),
                    ),
                )
                .await
                .unwrap();
            return;
        }

        write_file(
            &DATA_PATH.join(".env"),
            format!(
                "{}\n{} = {}\n{}",
                env_content,
                db,
                url,
                match login == password && login == None {
                    true => format!("NEEDAUTH = false"),
                    false => format!(
                        "NEEDAUTH = true\n{}_LOGIN = {}\n{}_PASSWORD = {}",
                        db,
                        login.unwrap(),
                        db,
                        password.unwrap()
                    ),
                }
            ),
        );

        inter
            .create_response(
                &ctx.http,
                CreateInteractionResponse::Message(
                    CreateInteractionResponseMessage::new().content(loc!("done")),
                ),
            )
            .await
            .unwrap();

        Logger::high(
            "command.create_db",
            &format!("created a new save db {}", db),
        )
        .await;
    }

    #[slash_command([])]
    async fn change_db_permissions(ctx: &Context, inter: CommandInteraction) {
        inter.defer_ephemeral(&ctx.http).await.unwrap();

        let role_man = ROLEMANAGER.read().await;

        if role_man.get_dbs().is_empty() {
            inter
                .edit_response(
                    &ctx.http,
                    EditInteractionResponse::new()
                        .content(loc!("change-db-permissions-command-no-dbs")),
                )
                .await
                .unwrap();
            return;
        }

        inter
            .edit_response(
                &ctx.http,
                EditInteractionResponse::new()
                    .content(loc!("change-db-permissions-command-response-title"))
                    .components(Vec::from([CreateActionRow::SelectMenu(
                        CreateSelectMenu::new(
                            "db-changer:db",
                            serenity::all::CreateSelectMenuKind::String {
                                options: role_man
                                    .get_dbs()
                                    .iter()
                                    .map(|x| CreateSelectMenuOption::new(*x, *x))
                                    .collect(),
                            },
                        )
                        .placeholder(loc!("change-db-permissions-command-db-select-placeholder")),
                    )])),
            )
            .await
            .unwrap();
    }

    #[listen_component("db-changer:db")]
    async fn db_changer_db(ctx: &Context, inter: ComponentInteraction) {
        let role_man = ROLEMANAGER.read().await;
        let mut mem_man = MEMBERSMANAGER.write().await;

        if let ComponentInteractionDataKind::StringSelect { values } = &inter.data.kind {
            mem_man.get_mut(inter.user.id).await.unwrap().changed_db =
                Some(values.first().unwrap().clone());

            inter
                .edit_response(
                    &ctx.http,
                    EditInteractionResponse::new()
                        .content(loc!(
                            "change-db-permissions-command-roles-title",
                            "db_name" = values.first().unwrap()
                        ))
                        .components(Vec::from([CreateActionRow::SelectMenu(
                            CreateSelectMenu::new(
                                "db-changer:roles",
                                serenity::all::CreateSelectMenuKind::Role {
                                    default_roles: role_man
                                        .get_db_permissions(values.first().unwrap())
                                        .cloned(),
                                },
                            )
                            .placeholder(loc!("change-db-permissions-command-roles-placeholder")),
                        )])),
                )
                .await
                .unwrap();
        }
    }

    #[listen_component("db-changer:roles")]
    async fn db_changer_roles(ctx: &Context, inter: ComponentInteraction) {
        let mut role_man = ROLEMANAGER.write().await;
        let mut mem_man = MEMBERSMANAGER.write().await;

        if let ComponentInteractionDataKind::RoleSelect { values } = &inter.data.kind {
            let member = mem_man.get(inter.user.id).await.unwrap();
            role_man
                .set_db_permissions(member.changed_db.clone().unwrap(), values.clone())
                .await;
        }

        inter
            .edit_response(&ctx.http, EditInteractionResponse::new())
            .await
            .unwrap();
    }
}
