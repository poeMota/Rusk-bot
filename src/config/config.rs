use crate::localization::LocalizationData;
use crate::logger::{Logger, LoggingConfig};
use dotenv;
use once_cell::sync::Lazy;
use serde::Deserialize;
use std::env::current_dir;
use std::fs;
use std::fs::File;
use std::io::ErrorKind;
use std::io::Write;
use std::sync::Arc;
use std::{collections::HashMap, path::PathBuf};
use tokio::{runtime::Runtime, sync::RwLock};

pub static DATA_PATH: Lazy<PathBuf> = Lazy::new(|| {
    let current_dir = current_dir().expect("Cannot find data folder");
    current_dir.join("data/")
});

pub static CONFIG: Lazy<Arc<RwLock<Config>>> = Lazy::new(|| {
    load_env();
    Arc::new(RwLock::new(Config::new("config.toml")))
});

pub fn load_env() {
    dotenv::from_path(DATA_PATH.join(".env")).expect("Cannot load .env");
}

#[derive(Debug, Deserialize)]
pub struct Config {
    pub guild: u64,
    pub max_dropdowns_per_message: u64,
    pub max_tasks_per_user: u64,
    pub brigadire_score_modifier: f64,
    pub userid_api_url: String,
    pub log: Option<u64>,
    pub guest_role: Option<u64>,
    pub shop_embed_color: Option<u32>,
    #[serde(rename = "Localization")]
    pub localization: LocalizationData,
    #[serde(rename = "TaskEndRating")]
    pub task_retings: HashMap<String, f64>,
    #[serde(rename = "Commands")]
    pub commands: HashMap<String, bool>,
    #[serde(rename = "Logging")]
    pub logging: LoggingConfig,
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
    if let Some(parent_dir) = path.parent() {
        fs::create_dir_all(parent_dir)
            .expect("Error while creating parent dirs while writing file");
    }

    let content = match fs::read_to_string(path) {
        Ok(s) => s,
        Err(error) => match error.kind() {
            ErrorKind::NotFound => {
                File::create(path).expect(
                    format!("Cannot create file in path: {}", path.to_str().unwrap()).as_str(),
                );
                String::new()
            }
            other_error => Runtime::new().unwrap().block_on(async {
                Logger::error(
                    "config.read_file",
                    &format!(
                        "uexpected error while reading file from path {}: {:?}",
                        path.to_str().unwrap(),
                        other_error
                    ),
                )
                .await;
                String::new()
            }),
        },
    };
    content
}

pub fn write_file(path: &PathBuf, content: String) {
    if let Some(parent_dir) = path.parent() {
        fs::create_dir_all(parent_dir)
            .expect("Error while creating parent dirs while writing file");
    }

    let mut file = File::create(path).expect("Cannot create or read file");
    write!(file, "{}", content).expect("Cannot write in file");
}
