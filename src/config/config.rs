use crate::localization::LocalizationData;
use once_cell::sync::Lazy;
use serde::Deserialize;
use std::env::current_dir;
use std::fs;
use std::fs::File;
use std::io::ErrorKind;
use std::sync::{Arc, Mutex};
use std::{collections::HashMap, path::PathBuf};

pub static DATA_PATH: Lazy<PathBuf> = Lazy::new(|| {
    let current_dir = current_dir().expect("Cannot find data folder");
    current_dir.join("data/")
});

pub static CONFIG: Lazy<Arc<Mutex<Config>>> =
    Lazy::new(|| Arc::new(Mutex::new(Config::new("config.toml"))));

#[derive(Debug, Deserialize)]
pub struct Config {
    pub guild: u64,
    pub max_dropdowns_per_message: u64,
    pub brigadire_score_modifier: f64,
    pub userid_api_url: String,
    pub log: Option<u64>,
    pub guest_role: Option<u64>,
    #[serde(rename = "Localization")]
    pub localization: LocalizationData,
    #[serde(rename = "TaskEndRating")]
    pub task_retings: HashMap<String, f64>,
    #[serde(rename = "Commands")]
    pub commands: HashMap<String, bool>,
    #[serde(rename = "Logging")]
    pub logging: HashMap<String, bool>,
}

impl Config {
    fn new(config_file: &str) -> Self {
        let content = read_file(&DATA_PATH.join(config_file));
        let config: Config = toml::from_str(&content)
            .expect("Could not read the config, it may be missing required fields");
        config
    }
}

pub fn read_file(path: &PathBuf) -> String {
    let content = match fs::read_to_string(path) {
        Ok(s) => s,
        Err(error) => match error.kind() {
            ErrorKind::NotFound => {
                File::create(path).expect(
                    format!("Cannot create file in path: {}", path.to_str().unwrap()).as_str(),
                );
                String::new()
            }
            other_error => {
                panic!("{other_error:?}")
            }
        },
    };
    content
}
