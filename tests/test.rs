use std::collections::HashMap;

use event_macro::*;
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
    let loc = LOCALIZATION.lock().unwrap();

    assert_eq!(String::from("test"), loc.get_string("test", None));
    assert_ne!(
        String::from("test-string-1"),
        loc.get_string("test-string-1", None)
    );
    assert_eq!(
        String::from("test output - test test"),
        loc.get_string(
            "test-string-2",
            Some(HashMap::from([("output", "test test")]))
        )
    );
}

#[test]
fn events_test() {
    #[event]
    struct Event1 {
        name: String,
    }

    #[event]
    struct Event2 {
        id: u8,
    }

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
