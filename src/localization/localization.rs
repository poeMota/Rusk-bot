use crate::config::{read_file, CONFIG, DATA_PATH, ROOT_PATH};
use dashmap::DashMap;
use once_cell::sync::Lazy;
use serde::Deserialize;
use serde_yaml;
use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;
use std::sync::{Arc, RwLock};
use walkdir::WalkDir;

pub static LOCALIZATION: Lazy<Arc<RwLock<Localization>>> = Lazy::new(|| {
    Arc::new(RwLock::new(Localization::new(
        CONFIG.try_read().unwrap().localization.clone(),
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
    locale_data: DashMap<String, String>,
}

impl Localization {
    fn new(data: LocalizationData) -> Localization {
        let mut loc = Localization {
            locale_path: data.locale_path,
            culture: data.culture,
            locale_data: DashMap::new(),
        };
        loc.collect_all();
        loc
    }

    fn collect_all(&mut self) {
        self.locale_data = DashMap::new();

        self.collect_locale(ROOT_PATH.join("locale/"));
        self.collect_locale(DATA_PATH.join(&self.locale_path).join(&self.culture));
    }

    fn collect_locale(&mut self, path: PathBuf) {
        if !fs::exists(&path).unwrap() {
            fs::create_dir(&path).unwrap();
        }

        for entry in WalkDir::new(path) {
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

    pub fn get_string<'a>(
        &'a self,
        string: &'a str,
        replacements: Option<HashMap<&str, &str>>,
    ) -> String {
        let mut text = match self.locale_data.get(string) {
            Some(s) => s.clone(),
            None => {
                return String::from(string);
            }
        };

        match replacements {
            Some(_) => {
                for (key, replacement) in replacements.unwrap().iter() {
                    let repl = "{".to_string() + key + "}";
                    if text.contains(repl.as_str()) {
                        text = text.replace(repl.as_str(), replacement);
                    }
                }
            }
            None => {}
        }
        text
    }
}

pub fn get_string(key: &str, replacements: Option<HashMap<&str, &str>>) -> String {
    let loc = LOCALIZATION.read().unwrap();
    let result = loc.get_string(key, replacements);
    result
}
