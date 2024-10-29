use command_macro::command;
use serenity::model::{
    channel::{Attachment, PartialChannel},
    guild::Role,
    user::User,
};
use std::fs;
use tokio;

use event_macro::*;
use std::collections::HashMap;
use task_bot::command_manager::*;
use task_bot::config::*;
use task_bot::events::*;
use task_bot::localization::*;
use task_bot::shop::*;

#[test]
fn read_config_test() {
    let config = CONFIG.lock().unwrap();

    println!("{:#?}", config);
}

#[test]
fn locale_test() {
    assert_eq!(String::from("test"), get_string("test", None));
    assert_ne!(
        String::from("test-string-1"),
        get_string("test-string-1", None)
    );
    assert_eq!(
        String::from("test output - test test"),
        get_string(
            "test-string-2",
            Some(HashMap::from([("output", "test test")]))
        )
    );
}

#[test]
fn events_test() {
    #[derive(Event)]
    struct Event1 {
        name: String,
    }
    register_event::<Event1>();

    #[derive(Event)]
    struct Event2 {
        id: u8,
    }
    register_event::<Event2>();

    fn test_event_fn1(ev: &Event1) {
        println!("Raised test func 1 - {}", ev.name);
    }

    struct TestStruct {}

    impl TestStruct {
        fn test_event_fn2(&self, ev: &Event2) {
            println!("Raised test func 2 - {}", ev.id);
        }
    }

    subscribe_event::<Event1>(Box::new(test_event_fn1));

    let test = TestStruct {};
    subscribe_event::<Event2>(Box::new(move |ev: &Event2| test.test_event_fn2(ev)));

    Event1 {
        name: String::from("test"),
    }
    .raise();

    Event2 { id: 9 }.raise();
}

#[test]
fn macro_test() {
    #[command([])]
    fn save(
        _num: i64,
        _float: f64,
        _string: String,
        _param3: User,
        _param4: Role,
        _param5: PartialChannel,
        _param6: Attachment,
    ) {
    }

    #[command([])]
    fn save_plus(
        _num: Option<i64>,
        _float: Option<f64>,
        _string: Option<String>,
        _param3: Option<User>,
        _param4: Option<Role>,
        _param5: Option<PartialChannel>,
        _param6: Option<Attachment>,
    ) {
    }

    #[command([
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
            choice = int,
            min_length = 5,
            max_length = 50
        ],
        _param3 = [choice = int],
        _param4 = [choice = int],
        _param5 = [choice = int],
        _param6 = [choice = int],
    ])]
    fn save_save(
        _num: i64,
        _float: f64,
        _string: String,
        _param3: User,
        _param4: Role,
        _param5: PartialChannel,
        _param6: Attachment,
    ) {
    }

    #[command([
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
    fn _command(
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

#[tokio::test]
async fn shop_test() {
    write_file(
        &DATA_PATH.join("shop/test_shop.yml"),
        r#"
        - type: page
          name: test page name.
          description: test page desc.
          price: 2
          onBuy:
            - type: sendMessage
              message: test test

        - type: replacement
          name: Test Str
          value: test 1 2 3

        - type: replacement
          name: Test Num
          value: 123

        - type: replacement
          name: Test Float
          value: 1.23
        "#
        .to_string(),
    );

    let shop_man = SHOPMANAGER.read().await;
    println!("{:#?}", shop_man);

    fs::remove_file(DATA_PATH.join("shop/test_shop.yml")).expect("Cannot delete test shop file");
}
