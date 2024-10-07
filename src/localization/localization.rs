use crate::config::{read_file, CONFIG, DATA_PATH};
use once_cell::sync::Lazy;
use serde::Deserialize;
use serde_yaml;
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use walkdir::WalkDir;

pub static LOCALIZATION: Lazy<Arc<Mutex<Localization>>> = Lazy::new(|| {
    Arc::new(Mutex::new(Localization::new(
        CONFIG.lock().unwrap().localization.clone(),
    )))
});

#[derive(Deserialize, Clone, Debug)]
pub struct LocalizationData {
    locale_path: String,
    culture: String,
}

pub struct Localization {
    locale_path: String,
    culture: String,
    locale_data: HashMap<String, String>,
}

impl Localization {
    fn new(data: LocalizationData) -> Localization {
        let mut loc = Localization {
            locale_path: data.locale_path,
            culture: data.culture,
            locale_data: HashMap::new(),
        };
        loc.collect_locale();
        loc
    }

    fn collect_locale(&mut self) {
        self.locale_data = HashMap::new();

        for entry in WalkDir::new(DATA_PATH.join(&self.locale_path).join(&self.culture)) {
            let entry = match entry {
                Ok(s) => s,
                Err(error) => {
                    println!("Error with locale file: {}", error);
                    continue;
                }
            };

            if !entry.path().is_file() {
                continue;
            }

            let content: HashMap<String, String> =
                serde_yaml::from_str(read_file(&entry.path().to_path_buf()).as_str()).expect(
                    format!(
                        "Error while parsing locale file: {}",
                        entry.file_name().to_str().unwrap()
                    )
                    .as_str(),
                );

            self.locale_data.extend(content);
        }
    }

    pub fn get_str<'a>(&'a self, string: &'a str, replacements: HashMap<&str, &str>) -> String {
        let mut text = match self.locale_data.get(string) {
            Some(s) => s.clone(),
            None => {
                return String::from(string);
            }
        };

        for (key, replacement) in replacements.iter() {
            let repl = "{".to_string() + key + "}";
            if text.contains(repl.as_str()) {
                text = text.replace(repl.as_str(), replacement);
            }
        }
        text
    }
}
