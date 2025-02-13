mod localization;

pub use localization::*;

#[macro_export]
macro_rules! loc {
    ($key:expr) => {
        LOCALIZATION.try_read().unwrap().get_string($key, None)
    };
    ($key:expr, $($k:literal = $v:expr),+ $(,)?) => {
        {
            use std::collections::HashMap;
            let mut map = HashMap::new();
            $( map.insert($k.to_string(), $v.to_string()); )+
            LOCALIZATION.try_read().unwrap().get_string($key, Some(map))}
    };
}
