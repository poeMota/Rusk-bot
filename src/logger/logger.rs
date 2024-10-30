use crate::config::{read_file, write_file, CONFIG, DATA_PATH};
use chrono::prelude::*;
use once_cell::sync::Lazy;
use serde::Deserialize;
use serenity::{builder::CreateMessage, client::Context, model::channel::GuildChannel};
use std::cmp::Eq;
use std::collections::HashMap;
use std::sync::{Arc, Mutex};

#[derive(Debug, Deserialize, Hash, PartialEq, Eq, Clone)]
#[serde(rename_all = "lowercase")]
enum LoggingLevels {
    Low,
    Medium,
    High,
    Debug,
    Error,
}

#[derive(Debug, Deserialize, Clone)]
pub struct LoggingConfig {
    #[serde(flatten)]
    levels: HashMap<LoggingLevels, bool>,
}

static LOGGER: Lazy<Arc<Mutex<Logger>>> = Lazy::new(|| {
    let config = CONFIG.lock().unwrap();
    Arc::new(Mutex::new(Logger::new(None, config.logging.clone())))
});

pub struct Logger {
    settings: HashMap<LoggingLevels, bool>,
    log_channel: Option<GuildChannel>,
}

impl Logger {
    fn new(log: Option<GuildChannel>, settings: LoggingConfig) -> Self {
        Self {
            settings: settings.levels,
            log_channel: log,
        }
    }

    async fn log(&self, ctx: &Context, level: LoggingLevels, author: &str, content: &str) {
        let prefix = match level {
            LoggingLevels::Low => "[Low]",
            LoggingLevels::Medium => "**[Medium]**",
            LoggingLevels::High => "**[High]**",
            LoggingLevels::Debug => "[Debug]",
            LoggingLevels::Error => "**[Error]**",
        };

        let utc: DateTime<Utc> = Utc::now();
        let time_prefix = format!(
            "{}-{}-{}, {}:{}:{}",
            utc.year(),
            utc.month(),
            utc.day(),
            utc.hour(),
            utc.minute(),
            utc.second()
        );

        let message = CreateMessage::new().content(format!("{} <{}>: {}", prefix, author, content));

        let enabled = match self.settings.get(&level) {
            Some(enable) => *enable,
            None => true,
        };

        if enabled {
            if let Some(channel) = self.log_channel.clone() {
                match channel.send_message(&ctx.http, message).await {
                    Ok(_) => (),
                    Err(e) => Self::file_logging(
                        format!("{} | [Error] <logger.log>: {}", time_prefix, e.to_string())
                            .as_str(),
                    ),
                }
            }
        }

        Self::file_logging(
            format!(
                "{} | {} <{}>: {}",
                time_prefix,
                prefix.replace("**", ""),
                author,
                content
            )
            .as_str(),
        );
    }

    pub fn file_logging(content: &str) {
        let utc: DateTime<Utc> = Utc::now();
        let file_name = format!("logs/{}-{}-{}.txt", utc.day(), utc.month(), utc.year());

        let log_content = format!("{}{}\n", read_file(&DATA_PATH.join(&file_name)), content);

        write_file(&DATA_PATH.join(file_name), log_content);
        println!("{}", content);
    }

    pub async fn low(ctx: &Context, author: &str, content: &str) {
        let log = LOGGER.lock().unwrap();
        log.log(&ctx, LoggingLevels::Low, author, content).await;
    }

    pub async fn medium(ctx: &Context, author: &str, content: &str) {
        let log = LOGGER.lock().unwrap();
        log.log(&ctx, LoggingLevels::Medium, author, content).await;
    }

    pub async fn high(ctx: &Context, author: &str, content: &str) {
        let log = LOGGER.lock().unwrap();
        log.log(&ctx, LoggingLevels::High, author, content).await;
    }

    pub async fn debug(ctx: &Context, author: &str, content: &str) {
        let log = LOGGER.lock().unwrap();
        log.log(&ctx, LoggingLevels::Debug, author, content).await;
    }

    pub async fn error(ctx: &Context, author: &str, content: &str) {
        let log = LOGGER.lock().unwrap();
        log.log(&ctx, LoggingLevels::Error, author, content).await;
    }
}
