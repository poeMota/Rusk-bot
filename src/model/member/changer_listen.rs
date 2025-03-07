use serenity::all::{
    ActionRowComponent, ComponentInteractionDataKind, CreateActionRow, CreateButton,
    CreateInputText, CreateModal, CreateSelectMenu, CreateSelectMenuOption, InputTextStyle,
};

use crate::{connect::ConnectionError, model::role::ROLEMANAGER, prelude::*};

pub async fn member_changer_listener() {
    #[listen_component("member-changer")]
    async fn changer(ctx: &Context, inter: ComponentInteraction) {
        let mut mem_man = member::MEMBERSMANAGER.try_write().unwrap();
        let member = mem_man.get(inter.user.id).await.unwrap();

        inter
            .create_response(
                &ctx.http,
                CreateInteractionResponse::UpdateMessage(member.main_changer().await),
            )
            .await
            .unwrap();
    }

    #[listen_component("member-changer:score")]
    async fn score_changer(ctx: &Context, inter: ComponentInteraction) {
        let mut mem_man = member::MEMBERSMANAGER.write().await;

        let author = mem_man.get(inter.user.id).await.unwrap().clone();
        let member = mem_man
            .get_mut(author.changed_member.unwrap())
            .await
            .unwrap();

        inter
            .create_response(
                &ctx.http,
                CreateInteractionResponse::Modal(
                    CreateModal::new(
                        "member-changer:score",
                        loc!("member-changer-modal-score-title"),
                    )
                    .components(Vec::from([CreateActionRow::InputText(
                        CreateInputText::new(
                            InputTextStyle::Short,
                            loc!("member-changer-modal-score-components-score-title"),
                            "member-changer:score:score",
                        )
                        .value(member.score.to_string()),
                    )])),
                ),
            )
            .await
            .unwrap();
    }

    #[listen_component("member-changer:own-folder")]
    async fn folder_changer(ctx: &Context, inter: ComponentInteraction) {
        let mut mem_man = member::MEMBERSMANAGER.write().await;
        let role_man = ROLEMANAGER.read().await;

        let author = mem_man.get(inter.user.id).await.unwrap().clone();
        let member = mem_man
            .get_mut(author.changed_member.unwrap())
            .await
            .unwrap();

        inter
            .create_response(
                &ctx.http,
                CreateInteractionResponse::Modal(
                    CreateModal::new(
                        "member-changer:own-folder",
                        loc!("member-changer-modal-own-folder-title"),
                    )
                    .components(
                        role_man
                            .get_dbs()
                            .iter()
                            .map(|x| {
                                CreateActionRow::InputText(
                                    CreateInputText::new(
                                        InputTextStyle::Short,
                                        loc!(
                                    "member-changer-modal-own-folder-components-folder-title",
                                    "db" = x
                                ),
                                        "member-changer:own-folder:folder",
                                    )
                                    .value(format!(
                                        "{}:::{}",
                                        x,
                                        member
                                            .own_folder
                                            .get(x.as_str())
                                            .cloned()
                                            .unwrap_or(None)
                                            .unwrap_or(String::new())
                                    ))
                                    .min_length(0),
                                )
                            })
                            .collect(),
                    ),
                ),
            )
            .await
            .unwrap();
    }

    #[listen_component("member-changer:notes")]
    async fn notes_changer(ctx: &Context, inter: ComponentInteraction) {
        let mut mem_man = member::MEMBERSMANAGER.write().await;

        let author = mem_man.get(inter.user.id).await.unwrap().clone();
        let member = mem_man
            .get_mut(author.changed_member.unwrap())
            .await
            .unwrap();

        let mut rows = Vec::new();
        let mut notes = Vec::new();
        let mut index = 0;

        for note in member.notes.iter() {
            let value = match note {
                member::NotesHistory::Current((_, _, string)) => string,
                member::NotesHistory::OldFormat(string) => string,
            };

            notes.push(CreateSelectMenuOption::new(
                value,
                format!("{}:::{}", index, value),
            ));
            index += 1;
        }

        if !notes.is_empty() {
            rows.push(CreateActionRow::SelectMenu(
                CreateSelectMenu::new(
                    "member-changer:notes:note-remove",
                    serenity::all::CreateSelectMenuKind::String { options: notes },
                )
                .placeholder(loc!("member-changer-notes-remove")),
            ));
        }

        rows.push(CreateActionRow::Buttons(Vec::from([CreateButton::new(
            "member-changer:notes:note-add",
        )
        .label(loc!("member-changer-notes-add-button"))
        .style(serenity::all::ButtonStyle::Success)])));

        rows.push(CreateActionRow::Buttons(Vec::from([CreateButton::new(
            "member-changer",
        )
        .label(loc!("back-button"))
        .style(serenity::all::ButtonStyle::Success)])));

        inter
            .create_response(
                &ctx.http,
                CreateInteractionResponse::UpdateMessage(
                    CreateInteractionResponseMessage::new().components(rows),
                ),
            )
            .await
            .unwrap();
    }

    #[listen_component("member-changer:warns")]
    async fn warns_changer(ctx: &Context, inter: ComponentInteraction) {
        let mut mem_man = member::MEMBERSMANAGER.write().await;

        let author = mem_man.get(inter.user.id).await.unwrap().clone();
        let member = mem_man
            .get_mut(author.changed_member.unwrap())
            .await
            .unwrap();

        let mut rows = Vec::new();
        let mut warns = Vec::new();
        let mut index = 0;

        for warn in member.warns.iter() {
            let value = match warn {
                member::NotesHistory::Current((_, _, string)) => string,
                member::NotesHistory::OldFormat(string) => string,
            };

            warns.push(CreateSelectMenuOption::new(
                value,
                format!("{}:::{}", index, value),
            ));
            index += 1;
        }

        if !warns.is_empty() {
            rows.push(CreateActionRow::SelectMenu(
                CreateSelectMenu::new(
                    "member-changer:warns:warn-remove",
                    serenity::all::CreateSelectMenuKind::String { options: warns },
                )
                .placeholder(loc!("member-changer-warns-remove")),
            ));
        }

        rows.push(CreateActionRow::Buttons(Vec::from([CreateButton::new(
            "member-changer:warns:warn-add",
        )
        .label(loc!("member-changer-warns-add-button"))
        .style(serenity::all::ButtonStyle::Success)])));

        rows.push(CreateActionRow::Buttons(Vec::from([CreateButton::new(
            "member-changer",
        )
        .label(loc!("back-button"))
        .style(serenity::all::ButtonStyle::Success)])));

        inter
            .create_response(
                &ctx.http,
                CreateInteractionResponse::UpdateMessage(
                    CreateInteractionResponseMessage::new().components(rows),
                ),
            )
            .await
            .unwrap();
    }

    #[listen_modal("member-changer:score")]
    async fn score_modal_submit(ctx: &Context, inter: ModalInteraction) {
        let mut mem_man = member::MEMBERSMANAGER.write().await;

        let author = mem_man.get(inter.user.id).await.unwrap().clone();
        let member = mem_man
            .get_mut(author.changed_member.unwrap())
            .await
            .unwrap();

        for row in inter.data.components.iter() {
            for comp in row.components.iter() {
                match comp {
                    ActionRowComponent::InputText(text) => {
                        if text.custom_id == "member-changer:score:score" {
                            let score: i64 =
                                match text.value.clone().unwrap_or(String::new()).parse() {
                                    Ok(num) => num,
                                    Err(_) => {
                                        inter
                                            .create_response(
                                                &ctx.http,
                                                CreateInteractionResponse::Message(
                                                    CreateInteractionResponseMessage::new()
                                                        .content(loc!(
                                                            "member-changer-score-parse-error"
                                                        )),
                                                ),
                                            )
                                            .await
                                            .unwrap();
                                        return;
                                    }
                                };

                            member.change_score(score - member.score).await;
                        }
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
                        .embed(member.to_embed(&ctx, true).await)
                        .ephemeral(true),
                ),
            )
            .await
            .unwrap();
    }

    #[listen_modal("member-changer:own-folder")]
    async fn folder_modal_submit(ctx: &Context, inter: ModalInteraction) {
        inter.defer_ephemeral(&ctx.http).await.unwrap();

        let mut mem_man = member::MEMBERSMANAGER.write().await;

        let author = mem_man.get(inter.user.id).await.unwrap().clone();
        let member = mem_man
            .get_mut(author.changed_member.unwrap())
            .await
            .unwrap();

        for row in inter.data.components.iter() {
            for comp in row.components.iter() {
                match comp {
                    ActionRowComponent::InputText(text) => {
                        if text.custom_id == "member-changer:own-folder:folder" {
                            let parts = text
                                .value
                                .clone()
                                .unwrap()
                                .split(":::")
                                .map(|x| x.to_string())
                                .collect::<Vec<String>>();
                            let db = parts.first().unwrap().clone();
                            let folder = parts.get(1).cloned();

                            match member.change_folder(db, folder).await {
                                Ok(_) => inter
                                    .edit_response(
                                        &ctx.http,
                                        EditInteractionResponse::new()
                                            .embed(member.to_embed(&ctx, true).await),
                                    )
                                    .await
                                    .unwrap(),
                                Err(e) => match e {
                                    ConnectionError::StatusCodeError(url, _) => inter
                                        .edit_response(
                                            &ctx.http,
                                            EditInteractionResponse::new()
                                                .content(loc!("invalid-url", "path" = url)),
                                        )
                                        .await
                                        .unwrap(),
                                    ConnectionError::ReqwestError(error) => {
                                        Logger::error(
                                            "commands.link_folder",
                                            &format!(
                                                "reqwest error while connection: {}",
                                                error.to_string()
                                            ),
                                        )
                                        .await;

                                        inter
                                            .edit_response(
                                                &ctx.http,
                                                EditInteractionResponse::new()
                                                    .content(loc!("link-folder-reqwest-error")),
                                            )
                                            .await
                                            .unwrap()
                                    }
                                    _ => inter
                                        .edit_response(
                                            &ctx.http,
                                            EditInteractionResponse::new().content(loc!(
                                                "link-folder-error",
                                                "error" = format!("{:#?}", e)
                                            )),
                                        )
                                        .await
                                        .unwrap(),
                                },
                            };
                        }
                    }
                    _ => (),
                }
            }
        }
    }

    #[listen_component("member-changer:tasks")]
    async fn tasks_changer_project(ctx: &Context, inter: ComponentInteraction) {
        let mut mem_man = member::MEMBERSMANAGER.write().await;
        let proj_man = project::PROJECTMANAGER.read().await;

        let author = mem_man.get(inter.user.id).await.unwrap().clone();
        let member = mem_man
            .get_mut(author.changed_member.unwrap())
            .await
            .unwrap();

        let mut rows = Vec::new();

        let mut projects = Vec::new();
        for proj in proj_man.projects() {
            projects.push(CreateSelectMenuOption::new(proj, proj));
        }

        if !member.done_tasks.is_empty() {
            let mut projs = Vec::new();
            for (proj, _) in member.done_tasks.iter() {
                projs.push(CreateSelectMenuOption::new(proj, proj));
            }

            rows.push(CreateActionRow::SelectMenu(
                CreateSelectMenu::new(
                    "member-changer:tasks:done-tasks-remove-project",
                    serenity::all::CreateSelectMenuKind::String { options: projs },
                )
                .placeholder(loc!("member-changer-tasks-done-tasks-remove")),
            ));
        }

        if !projects.is_empty() {
            rows.push(CreateActionRow::SelectMenu(
                CreateSelectMenu::new(
                    "member-changer:tasks:done-tasks-add",
                    serenity::all::CreateSelectMenuKind::String {
                        options: projects.clone(),
                    },
                )
                .placeholder(loc!("member-changer-tasks-done-tasks-add")),
            ));
        }

        if !member.mentor_tasks.is_empty() {
            let mut projs = Vec::new();
            for (proj, _) in member.mentor_tasks.iter() {
                projs.push(CreateSelectMenuOption::new(proj, proj));
            }

            rows.push(CreateActionRow::SelectMenu(
                CreateSelectMenu::new(
                    "member-changer:tasks:mentor-tasks-remove-project",
                    serenity::all::CreateSelectMenuKind::String { options: projs },
                )
                .placeholder(loc!("member-changer-tasks-mentor-tasks-remove")),
            ));
        }

        if !projects.is_empty() {
            rows.push(CreateActionRow::SelectMenu(
                CreateSelectMenu::new(
                    "member-changer:tasks:mentor-tasks-add",
                    serenity::all::CreateSelectMenuKind::String {
                        options: projects.clone(),
                    },
                )
                .placeholder(loc!("member-changer-tasks-mentor-tasks-add")),
            ));
        }

        rows.push(CreateActionRow::Buttons(Vec::from([CreateButton::new(
            "member-changer",
        )
        .label(loc!("back-button"))
        .style(serenity::all::ButtonStyle::Success)])));

        inter
            .create_response(
                &ctx.http,
                CreateInteractionResponse::UpdateMessage(
                    CreateInteractionResponseMessage::new().components(rows),
                ),
            )
            .await
            .unwrap();
    }

    #[listen_component("member-changer:tasks:done-tasks-remove-project")]
    async fn done_tasks_project(ctx: &Context, inter: ComponentInteraction) {
        let mut mem_man = member::MEMBERSMANAGER.write().await;
        let task_man = task::TASKMANAGER.read().await;

        let author = mem_man.get(inter.user.id).await.unwrap().clone();
        let member = mem_man
            .get_mut(author.changed_member.unwrap())
            .await
            .unwrap();

        let project = match &inter.data.kind {
            ComponentInteractionDataKind::StringSelect { values } => {
                values.first().unwrap().clone()
            }
            _ => return,
        };
        let mut rows = Vec::new();
        let mut done_tasks_remove = Vec::new();

        for task in task_man.get_by_project(&project) {
            let mut in_done = false;
            for hist in member
                .done_tasks
                .get(&project)
                .unwrap_or(&Vec::new())
                .iter()
            {
                if let member::TaskHistory::Current(current) = hist {
                    for (_, id) in current.iter() {
                        if task.id == *id {
                            in_done = true;
                        }
                    }
                }
            }

            if in_done {
                done_tasks_remove.push(CreateSelectMenuOption::new(
                    task.name.get(),
                    format!("{}:::{}", project, task.id.to_string()),
                ));
            }
        }

        for task in member
            .done_tasks
            .get(&project)
            .unwrap_or(&Vec::new())
            .iter()
        {
            if let member::TaskHistory::OldFormat(string) = task {
                done_tasks_remove.push(CreateSelectMenuOption::new(
                    string,
                    format!("{}:::{}", project, string),
                ));
            }
        }

        if !done_tasks_remove.is_empty() {
            rows.push(CreateActionRow::SelectMenu(
                CreateSelectMenu::new(
                    "member-changer:tasks:done-tasks-remove",
                    serenity::all::CreateSelectMenuKind::String {
                        options: done_tasks_remove,
                    },
                )
                .placeholder(loc!("member-changer-tasks-done-tasks-remove")),
            ));
        }

        rows.push(CreateActionRow::Buttons(Vec::from([CreateButton::new(
            "member-changer:tasks",
        )
        .label(loc!("back-button"))
        .style(serenity::all::ButtonStyle::Success)])));

        inter
            .create_response(
                &ctx.http,
                CreateInteractionResponse::UpdateMessage(
                    CreateInteractionResponseMessage::new().components(rows),
                ),
            )
            .await
            .unwrap();
    }

    #[listen_component("member-changer:tasks:mentor-tasks-remove-project")]
    async fn mentor_tasks_project(ctx: &Context, inter: ComponentInteraction) {
        let mut mem_man = member::MEMBERSMANAGER.write().await;
        let task_man = task::TASKMANAGER.read().await;

        let author = mem_man.get(inter.user.id).await.unwrap().clone();
        let member = mem_man
            .get_mut(author.changed_member.unwrap())
            .await
            .unwrap();

        let project = match &inter.data.kind {
            ComponentInteractionDataKind::StringSelect { values } => {
                values.first().unwrap().clone()
            }
            _ => return,
        };
        let mut rows = Vec::new();
        let mut mentor_tasks_remove = Vec::new();

        for task in task_man.get_by_project(&project) {
            let mut in_mentor = false;
            for hist in member
                .mentor_tasks
                .get(&project)
                .unwrap_or(&Vec::new())
                .iter()
            {
                if let member::TaskHistory::Current(current) = hist {
                    for (_, id) in current.iter() {
                        if task.id == *id {
                            in_mentor = true;
                        }
                    }
                }
            }

            if in_mentor {
                mentor_tasks_remove.push(CreateSelectMenuOption::new(
                    task.name.get(),
                    format!("{}:::{}", project, task.id.to_string()),
                ));
            }
        }

        for task in member
            .mentor_tasks
            .get(&project)
            .unwrap_or(&Vec::new())
            .iter()
        {
            if let member::TaskHistory::OldFormat(string) = task {
                mentor_tasks_remove.push(CreateSelectMenuOption::new(
                    string,
                    format!("{}:::{}", project, string),
                ));
            }
        }

        if !mentor_tasks_remove.is_empty() {
            rows.push(CreateActionRow::SelectMenu(
                CreateSelectMenu::new(
                    "member-changer:tasks:mentor-tasks-remove",
                    serenity::all::CreateSelectMenuKind::String {
                        options: mentor_tasks_remove,
                    },
                )
                .placeholder(loc!("member-changer-tasks-mentor-tasks-remove")),
            ));
        }

        rows.push(CreateActionRow::Buttons(Vec::from([CreateButton::new(
            "member-changer:tasks",
        )
        .label(loc!("back-button"))
        .style(serenity::all::ButtonStyle::Success)])));

        inter
            .create_response(
                &ctx.http,
                CreateInteractionResponse::UpdateMessage(
                    CreateInteractionResponseMessage::new().components(rows),
                ),
            )
            .await
            .unwrap();
    }

    #[listen_component("member-changer:tasks:done-tasks-add")]
    async fn add_done_task(ctx: &Context, inter: ComponentInteraction) {
        let project = match &inter.data.kind {
            ComponentInteractionDataKind::StringSelect { values } => {
                values.first().unwrap().clone()
            }
            _ => return,
        };

        inter
            .create_response(
                &ctx.http,
                CreateInteractionResponse::Modal(
                    CreateModal::new(
                        "member-changer:tasks:done-tasks-add-custom",
                        loc!("member-changer-tasks-done-tasks-add-custom-modal-label"),
                    )
                    .components(Vec::from([
                        CreateActionRow::InputText(
                            CreateInputText::new(
                                InputTextStyle::Short,
                                loc!("member-changer-tasks-done-tasks-add-project-input-label"),
                                "member-changer:tasks:done-tasks-add-project-input",
                            )
                            .value(project),
                        ),
                        CreateActionRow::InputText(CreateInputText::new(
                            InputTextStyle::Short,
                            loc!("member-changer-tasks-done-tasks-add-custom-input-label"),
                            "member-changer:tasks:done-tasks-add-custom-input",
                        )),
                    ])),
                ),
            )
            .await
            .unwrap();
    }

    #[listen_component("member-changer:tasks:mentor-tasks-add")]
    async fn add_mentor_task(ctx: &Context, inter: ComponentInteraction) {
        let project = match &inter.data.kind {
            ComponentInteractionDataKind::StringSelect { values } => {
                values.first().unwrap().clone()
            }
            _ => return,
        };

        inter
            .create_response(
                &ctx.http,
                CreateInteractionResponse::Modal(
                    CreateModal::new(
                        "member-changer:tasks:mentor-tasks-add-custom",
                        loc!("member-changer-tasks-mentor-tasks-add-custom-modal-label"),
                    )
                    .components(Vec::from([
                        CreateActionRow::InputText(
                            CreateInputText::new(
                                InputTextStyle::Short,
                                loc!("member-changer-tasks-mentor-tasks-add-project-input-label"),
                                "member-changer:tasks:mentor-tasks-add-project-input",
                            )
                            .value(project),
                        ),
                        CreateActionRow::InputText(CreateInputText::new(
                            InputTextStyle::Short,
                            loc!("member-changer-tasks-mentor-tasks-add-custom-input-label"),
                            "member-changer:tasks:mentor-tasks-add-custom-input",
                        )),
                    ])),
                ),
            )
            .await
            .unwrap();
    }

    #[listen_modal("member-changer:tasks:done-tasks-add-custom")]
    async fn add_done_task_submit(ctx: &Context, inter: ModalInteraction) {
        let mut mem_man = member::MEMBERSMANAGER.write().await;
        let author = mem_man.get(inter.user.id).await.unwrap().clone();
        let member = mem_man
            .get_mut(author.changed_member.unwrap())
            .await
            .unwrap();

        let mut project = &String::new();
        let mut task = &String::new();

        for row in inter.data.components.iter() {
            for comp in row.components.iter() {
                match comp {
                    ActionRowComponent::InputText(input) => {
                        if input.custom_id == "member-changer:tasks:done-tasks-add-custom-input" {
                            if let Some(ref text) = input.value {
                                task = text;
                            }
                        }
                        if input.custom_id == "member-changer:tasks:done-tasks-add-project-input" {
                            if let Some(ref text) = input.value {
                                project = text;
                            }
                        }
                    }
                    _ => (),
                }
            }
        }

        member
            .add_custom_done_task(project, member::TaskHistory::OldFormat(task.clone()))
            .await;

        inter
            .create_response(
                &ctx.http,
                CreateInteractionResponse::Message(
                    CreateInteractionResponseMessage::new()
                        .embed(member.to_embed(&ctx, true).await)
                        .ephemeral(true),
                ),
            )
            .await
            .unwrap();
    }

    #[listen_modal("member-changer:tasks:mentor-tasks-add-custom")]
    async fn add_mentor_task_submit(ctx: &Context, inter: ModalInteraction) {
        let mut mem_man = member::MEMBERSMANAGER.write().await;
        let author = mem_man.get(inter.user.id).await.unwrap().clone();
        let member = mem_man
            .get_mut(author.changed_member.unwrap())
            .await
            .unwrap();

        let mut project = &String::new();
        let mut task = &String::new();

        for row in inter.data.components.iter() {
            for comp in row.components.iter() {
                match comp {
                    ActionRowComponent::InputText(input) => {
                        if input.custom_id == "member-changer:tasks:mentor-tasks-add-custom-input" {
                            if let Some(ref text) = input.value {
                                task = text;
                            }
                        }
                        if input.custom_id == "member-changer:tasks:mentor-tasks-add-project-input"
                        {
                            if let Some(ref text) = input.value {
                                project = text;
                            }
                        }
                    }
                    _ => (),
                }
            }
        }

        member
            .add_custom_mentor_task(project, member::TaskHistory::OldFormat(task.clone()))
            .await;

        inter
            .create_response(
                &ctx.http,
                CreateInteractionResponse::Message(
                    CreateInteractionResponseMessage::new()
                        .embed(member.to_embed(&ctx, true).await)
                        .ephemeral(true),
                ),
            )
            .await
            .unwrap();
    }

    #[listen_component("member-changer:tasks:done-tasks-remove")]
    async fn remove_done_tasks(ctx: &Context, inter: ComponentInteraction) {
        let mut mem_man = member::MEMBERSMANAGER.write().await;
        let author = mem_man.get(inter.user.id).await.unwrap().clone();
        let member = mem_man
            .get_mut(author.changed_member.unwrap())
            .await
            .unwrap();

        if let ComponentInteractionDataKind::StringSelect { ref values } = inter.data.kind {
            for value in values {
                let project = value
                    .split(":::")
                    .collect::<Vec<&str>>()
                    .first()
                    .unwrap()
                    .to_string();

                let val = value.split(":::").last().unwrap();

                match val.parse::<u32>() {
                    Ok(id) => {
                        let mut index = 0;

                        'hist: for hist in member
                            .done_tasks
                            .get(&project)
                            .unwrap_or(&Vec::new())
                            .iter()
                        {
                            if let member::TaskHistory::Current(current) = hist {
                                for (_, hist_id) in current.iter() {
                                    if id == *hist_id {
                                        member.remove_done_task(&project, index).await;
                                        break 'hist;
                                    }
                                }

                                index += 1;
                            }
                        }
                    }
                    Err(_) => {
                        let mut index = 0;

                        for hist in member
                            .done_tasks
                            .get(&project)
                            .unwrap_or(&Vec::new())
                            .iter()
                        {
                            if hist.get().await.contains(&val) {
                                member.remove_done_task(&project, index).await;
                                break;
                            }

                            index += 1;
                        }
                    }
                }
            }
        }

        inter
            .create_response(
                &ctx.http,
                CreateInteractionResponse::Message(
                    CreateInteractionResponseMessage::new()
                        .embed(member.to_embed(&ctx, true).await)
                        .ephemeral(true),
                ),
            )
            .await
            .unwrap();
    }

    #[listen_component("member-changer:tasks:mentor-tasks-remove")]
    async fn remove_mentor_tasks(ctx: &Context, inter: ComponentInteraction) {
        let mut mem_man = member::MEMBERSMANAGER.write().await;
        let author = mem_man.get(inter.user.id).await.unwrap().clone();
        let member = mem_man
            .get_mut(author.changed_member.unwrap())
            .await
            .unwrap();

        if let ComponentInteractionDataKind::StringSelect { ref values } = inter.data.kind {
            for value in values {
                let project = value
                    .split(":::")
                    .collect::<Vec<&str>>()
                    .first()
                    .unwrap()
                    .to_string();

                let val = value.split(":::").last().unwrap();

                match val.parse::<u32>() {
                    Ok(id) => {
                        let mut index = 0;

                        'hist: for hist in member
                            .mentor_tasks
                            .get(&project)
                            .unwrap_or(&Vec::new())
                            .iter()
                        {
                            if let member::TaskHistory::Current(current) = hist {
                                for (_, hist_id) in current.iter() {
                                    if id == *hist_id {
                                        member.remove_mentor_task(&project, index).await;
                                        break 'hist;
                                    }
                                }

                                index += 1;
                            }
                        }
                    }
                    Err(_) => {
                        let mut index = 0;

                        for hist in member
                            .mentor_tasks
                            .get(&project)
                            .unwrap_or(&Vec::new())
                            .iter()
                        {
                            if hist.get().await.contains(&val) {
                                member.remove_mentor_task(&project, index).await;
                                break;
                            }

                            index += 1;
                        }
                    }
                }
            }
        }

        inter
            .create_response(
                &ctx.http,
                CreateInteractionResponse::Message(
                    CreateInteractionResponseMessage::new()
                        .embed(member.to_embed(&ctx, true).await)
                        .ephemeral(true),
                ),
            )
            .await
            .unwrap();
    }

    #[listen_component("member-changer:notes:note-add")]
    async fn note_add_changer(ctx: &Context, inter: ComponentInteraction) {
        inter
            .create_response(
                &ctx.http,
                CreateInteractionResponse::Modal(
                    CreateModal::new(
                        "member-changer:notes:note-add",
                        loc!("member-changer-notes-note-add-modal"),
                    )
                    .components(Vec::from([CreateActionRow::InputText(
                        CreateInputText::new(
                            InputTextStyle::Short,
                            loc!("member-changer-notes-note-add-label"),
                            "member-changer:notes:note-add-input",
                        ),
                    )])),
                ),
            )
            .await
            .unwrap();
    }

    #[listen_component("member-changer:warns:warn-add")]
    async fn warn_add_changer(ctx: &Context, inter: ComponentInteraction) {
        inter
            .create_response(
                &ctx.http,
                CreateInteractionResponse::Modal(
                    CreateModal::new(
                        "member-changer:warns:warn-add",
                        loc!("member-changer-warns-warn-add-modal"),
                    )
                    .components(Vec::from([CreateActionRow::InputText(
                        CreateInputText::new(
                            InputTextStyle::Short,
                            loc!("member-changer-warns-warn-add-label"),
                            "member-changer:warns:warn-add-input",
                        ),
                    )])),
                ),
            )
            .await
            .unwrap();
    }

    #[listen_modal("member-changer:notes:note-add")]
    async fn note_add_submit(ctx: &Context, inter: ModalInteraction) {
        let mut mem_man = member::MEMBERSMANAGER.write().await;
        let author = mem_man.get(inter.user.id).await.unwrap().clone();
        let member = mem_man
            .get_mut(author.changed_member.unwrap())
            .await
            .unwrap();

        for row in inter.data.components.iter() {
            for comp in row.components.iter() {
                match comp {
                    ActionRowComponent::InputText(input) => {
                        if input.custom_id == "member-changer:notes:note-add-input" {
                            if let Some(ref text) = input.value {
                                member.add_note(inter.user.id, text.clone()).await;
                            }
                        }
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
                        .embed(member.to_embed(&ctx, true).await)
                        .ephemeral(true),
                ),
            )
            .await
            .unwrap();
    }

    #[listen_modal("member-changer:warns:warn-add")]
    async fn warn_add_submit(ctx: &Context, inter: ModalInteraction) {
        let mut mem_man = member::MEMBERSMANAGER.write().await;
        let author = mem_man.get(inter.user.id).await.unwrap().clone();
        let member = mem_man
            .get_mut(author.changed_member.unwrap())
            .await
            .unwrap();

        for row in inter.data.components.iter() {
            for comp in row.components.iter() {
                match comp {
                    ActionRowComponent::InputText(input) => {
                        if input.custom_id == "member-changer:warns:warn-add-input" {
                            if let Some(ref text) = input.value {
                                member.add_warn(inter.user.id, text.clone()).await;
                            }
                        }
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
                        .embed(member.to_embed(&ctx, true).await)
                        .ephemeral(true),
                ),
            )
            .await
            .unwrap();
    }

    #[listen_component("member-changer:notes:note-remove")]
    async fn note_remove(ctx: &Context, inter: ComponentInteraction) {
        let mut mem_man = member::MEMBERSMANAGER.write().await;
        let author = mem_man.get(inter.user.id).await.unwrap().clone();
        let member = mem_man
            .get_mut(author.changed_member.unwrap())
            .await
            .unwrap();

        if let ComponentInteractionDataKind::StringSelect { ref values } = inter.data.kind {
            for value in values {
                let mut index: usize = 0;

                for note in member.notes.clone().iter() {
                    match note {
                        member::NotesHistory::Current((_, _, string)) => {
                            if string == value.split(":::").last().unwrap() {
                                member.remove_note(author.id, index).await;
                            }
                        }
                        member::NotesHistory::OldFormat(string) => {
                            if string == value.split(":::").last().unwrap() {
                                member.remove_note(author.id, index).await;
                            }
                        }
                    }

                    index += 1;
                }
            }
        }

        inter
            .create_response(
                &ctx.http,
                CreateInteractionResponse::Message(
                    CreateInteractionResponseMessage::new()
                        .embed(member.to_embed(&ctx, true).await)
                        .ephemeral(true),
                ),
            )
            .await
            .unwrap();
    }

    #[listen_component("member-changer:warns:warn-remove")]
    async fn warn_remove(ctx: &Context, inter: ComponentInteraction) {
        let mut mem_man = member::MEMBERSMANAGER.write().await;
        let author = mem_man.get(inter.user.id).await.unwrap().clone();
        let member = mem_man
            .get_mut(author.changed_member.unwrap())
            .await
            .unwrap();

        if let ComponentInteractionDataKind::StringSelect { ref values } = inter.data.kind {
            for value in values {
                let mut index: usize = 0;

                for warn in member.warns.clone().iter() {
                    match warn {
                        member::NotesHistory::Current((_, _, string)) => {
                            if string == value.split(":::").last().unwrap() {
                                member.remove_warn(author.id, index).await;
                            }
                        }
                        member::NotesHistory::OldFormat(string) => {
                            if string == value.split(":::").last().unwrap() {
                                member.remove_warn(author.id, index).await;
                            }
                        }
                    }

                    index += 1;
                }
            }
        }

        inter
            .create_response(
                &ctx.http,
                CreateInteractionResponse::Message(
                    CreateInteractionResponseMessage::new()
                        .embed(member.to_embed(&ctx, true).await)
                        .ephemeral(true),
                ),
            )
            .await
            .unwrap();
    }
}
