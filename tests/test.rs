use command_macro::command;
use serenity::model::{
    channel::{Attachment, PartialChannel},
    guild::Role,
    user::User,
};

use event_macro::*;
use std::collections::HashMap;
use task_bot::command_manager::*;
use task_bot::config::*;
use task_bot::events::*;
use task_bot::localization::*;

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
    #[command([param1 = "sdfafs", sss = 123u64, testtest = test_str])]
    fn save(
        _num: i64,
        _string: String,
        _param3: User,
        _param4: Role,
        _param5: PartialChannel,
        _param6: Attachment,
    ) {
    }

    #[command([param2 = "jnsxnvksj", sss = 123])]
    fn save_plus(
        _num: Option<i64>,
        _string: Option<String>,
        _param3: Option<User>,
        _param4: Option<Role>,
        _param5: Option<PartialChannel>,
        _param6: Option<Attachment>,
    ) {
    }
}
