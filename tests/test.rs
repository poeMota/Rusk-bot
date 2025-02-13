use command_macro::slash_command;
use serenity::{
    client::Context,
    model::{
        application::CommandInteraction,
        channel::{Attachment, PartialChannel},
        guild::Role,
        id::GuildId,
        user::User,
    },
};
use std::fs;
use task_bot::{
    command_manager::*,
    config::*,
    connect::*,
    localization::*,
    logger::*,
    model::{member::MEMBERSMANAGER, project::PROJECTMANAGER, tag::TAGSMANAGER, task::TASKMANAGER},
    prelude::*,
};
use tokio;

#[test]
fn read_config_test() {
    let _ = CONFIG.try_read().unwrap();
}

#[test]
fn locale_test() {
    write_file(
        &DATA_PATH.join("locale/RU_ru/test.yml"),
        r#"
        test-string-1: test string output
        test-string-2: test output - {output}
        page-test-locale: locale test
        "#
        .to_string(),
    );

    assert_eq!(String::from("test"), loc!("test"));
    assert_ne!(String::from("test-string-1"), loc!("test-string-1"));
    assert_eq!(
        String::from("test output - test test"),
        loc!("test-string-2", "output" = "test test")
    );

    fs::remove_file(DATA_PATH.join("locale/RU_ru/test.yml"))
        .expect("Cannot delete test locale file");
}

#[allow(unused_must_use)]
#[tokio::test]
async fn macro_test() {
    async fn _test_apply_command(ctx: Context, guild: GuildId) {
        #[slash_command([])]
        async fn save(
            _ctx: &Context,
            _inter: CommandInteraction,
            _num: i64,
            _float: f64,
            _string: String,
            _param3: User,
            _param4: Role,
            _param5: PartialChannel,
            _param6: Attachment,
        ) {
        }

        #[slash_command([])]
        async fn save_plus(
            _ctx: &Context,
            _inter: CommandInteraction,
            _num: Option<i64>,
            _float: Option<f64>,
            _string: Option<String>,
            _param3: Option<User>,
            _param4: Option<Role>,
            _param5: Option<PartialChannel>,
            _param6: Option<Attachment>,
        ) {
        }

        #[slash_command([
            _num = [
                choice = int,
                min_int_value = 10,
                max_int_value = 100
            ],
            _float = [
                choice = int,
                min_number_value = 1.0,
                max_number_value = 10.0
            ],
            _string = [
                choice = locale,
                min_length = 5,
                max_length = 50
            ],
            _param3 = [choice = locale],
            _param4 = [choice = locale],
            _param5 = [choice = locale],
            _param6 = [choice = locale],
        ])]
        async fn save_save(
            _ctx: &Context,
            _inter: CommandInteraction,
            _num: i64,
            _float: f64,
            _string: String,
            _param3: User,
            _param4: Role,
            _param5: PartialChannel,
            _param6: Attachment,
        ) {
        }

        #[slash_command([
            _num = [
                base_value = 15,
                min_int_value = 10,
                max_int_value = 100
            ],
            _float = [
                base_value = 1.5,
                min_number_value = 1.0,
                max_number_value = 10.0
            ],
            _string = [
                base_value = "test",
                min_length = 5,
                max_length = 50
            ],
            _param3 = [choice = int],
            _param4 = [choice = int],
            _param5 = [choice = int],
            _param6 = [choice = int],
        ])]
        async fn _command(
            _ctx: &Context,
            _inter: CommandInteraction,
            _num: Option<i64>,
            _float: Option<f64>,
            _string: Option<String>,
            _param3: Option<User>,
            _param4: Option<Role>,
            _param5: Option<PartialChannel>,
            _param6: Option<Attachment>,
        ) {
        }
    }
}

#[tokio::test]
async fn listen_macros_test() {
    #[listen_component("test_comp")]
    async fn test_comp_listener(_ctx: &Context, _inter: ComponentInteraction) {}

    #[listen_component("test_comp2")]
    async fn test_comp_listener2(_ctx: &Context, _inter: ComponentInteraction) {}

    #[listen_modal("test_modal")]
    async fn test_modal_listener(_ctx: &Context, _inter: ModalInteraction) {}

    #[listen_modal("test_modal2")]
    async fn test_modal_listener2(_ctx: &Context, _inter: ModalInteraction) {}
}

#[tokio::test]
async fn shop_test() {
    write_file(
        &DATA_PATH.join("shop/test_shop.yml"),
        r#"
        - type: page
          name: page-test-locale
          description: test page desc.
          price: 2
          onBuy:
            - type: sendMessage
              message: <test str> test test
            - type: giveRoles
              roles: ["1", "2", "3"]
            - type: removeRoles
              roles: ["1", "2", "3"]
            - type: mute
              duration: 60

        - type: replacement
          name: test str
          value: test 1 2 3

        - type: replacement
          name: test num
          value: 123

        - type: replacement
          name: Test Float
          value: 1.23
        "#
        .to_string(),
    );

    let _shop_man = SHOPMANAGER.try_write().unwrap();
    //TODO: shop_man.init().await;

    fs::remove_file(DATA_PATH.join("shop/test_shop.yml")).expect("Cannot delete test shop file");
}

#[test]
fn logger_test() {
    Logger::file_logging("test log");
    Logger::file_logging("test log 2");
}

#[tokio::test]
async fn connect_test() {
    assert_eq!(
        get_user_id("dfhshfehwifhewhj2h1h2jbfnewbjehfjdhskjkhejhfdkjsh".to_string()).await,
        "Not Found".to_string()
    );
    assert_ne!(
        get_user_id("M0ta".to_string()).await,
        "Not Found".to_string()
    )
}

#[tokio::test]
async fn members_manager_test() {
    write_file(
        &DATA_PATH.join("databases/members/test.json"),
        r#"
        {
            "id": 1234324,
            "done_tasks": { },
            "curation_tasks": { },
            "own_folder": "SomeFolder",
            "score": 12,
            "all_time_score": 123,
            "warns": [ ],
            "notes": [ ]
        }"#
        .to_string(),
    );

    let mut mem_man = MEMBERSMANAGER.try_write().unwrap();
    mem_man.init().await;

    fs::remove_file(DATA_PATH.join("databases/members/test.json"))
        .expect("Cannot delete test members database");
}

#[tokio::test]
async fn tasks_manager_test() {
    let mut task_man = TASKMANAGER.try_write().unwrap();

    task_man.init().await;
}

#[tokio::test]
async fn tags_manager_test() {
    write_file(
        &DATA_PATH.join("databases/tags/test.json"),
        r#"
        {
            "id": 12331424,
            "forum_id": 212467316,
            "max_members": 3,
            "score_modifier": 4
        }"#
        .to_string(),
    );

    let mut tags_man = TAGSMANAGER.try_write().unwrap();
    tags_man.init().await;

    fs::remove_file(DATA_PATH.join("databases/tags/test.json"))
        .expect("Cannot delete test tag file");
}

#[tokio::test]
async fn projects_manager_test() {
    write_file(
        &DATA_PATH.join("databases/projects/test_project.yml"),
        r#"
        {
            "name": "test project",
            "max_tasks_per_user": 2,
            "tasks_forum": 4321324324,
            "waiter_role": 3424323231,
            "stat_posts": {},
            "stat_channel": 4323421897,
            "associated_roles": [
                847395357483,
                354375984739,
                352759375498
            ]
        }
        "#
        .to_string(),
    );

    let mut proj_man = PROJECTMANAGER.try_write().unwrap();
    proj_man.init().await;

    fs::remove_file(DATA_PATH.join("databases/projects/test_project.yml"))
        .expect("Cannot delete test project file");
}
