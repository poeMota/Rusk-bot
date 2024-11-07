use crate::{
    bot::{get_guild, get_http},
    config::{read_file, write_file, CONFIG, DATA_PATH},
};
use chrono::prelude::*;
use once_cell::sync::Lazy;
use serde::Deserialize;
use serenity::{
    builder::CreateMessage,
    client::Context,
    model::{channel::GuildChannel, id::ChannelId},
};
use std::cmp::Eq;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;

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

static LOGGER: Lazy<Arc<RwLock<Logger>>> = Lazy::new(|| {
    Arc::new(RwLock::new(Logger::new(
        None,
        CONFIG
            .try_read()
            .expect("Cannot lock CONFIG for LOGGER")
            .logging
            .clone(),
    )))
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

    pub async fn set_log_channel(ctx: &Context, channel: u64) {
        match get_guild().channels(&ctx.http).await {
            Ok(channels) => {
                LOGGER.write().await.log_channel = channels.get(&ChannelId::new(channel)).cloned();

                Self::debug(
                    "logger.set_log_channel",
                    &format!("the log channel is set to {}", channel),
                )
                .await;
            }
            Err(e) => {
                Self::error(
                    "logger.set_log_channel",
                    &format!("Error while setting log channel: {}", e.to_string()),
                )
                .await;
            }
        }
    }

    async fn log(&self, level: LoggingLevels, author: &str, content: &str) {
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

        let message = CreateMessage::new().content(format!(
            "{} ({}) <{}>: {}",
            prefix, time_prefix, author, content
        ));

        let enabled = match self.settings.get(&level) {
            Some(enable) => *enable,
            None => true,
        };

        if enabled {
            if let Some(channel) = self.log_channel.clone() {
                match channel.send_message(get_http(), message).await {
                    Ok(_) => (),
                    Err(e) => Self::file_logging(
                        format!("[Error] ({}) <logger.log>: {}", time_prefix, e.to_string())
                            .as_str(),
                    ),
                }
            }
        }

        Self::file_logging(
            format!(
                "{} ({}) <{}>: {}",
                prefix.replace("**", ""),
                time_prefix,
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

    pub async fn expect<T: Clone, E: Clone + ToString>(
        author: &str,
        expected: Result<T, E>,
    ) -> Result<T, E> {
        if let Err(ref error) = expected {
            Self::error(author, error.to_string().as_str()).await
        }
        expected
    }

    pub async fn low(author: &str, content: &str) {
        let log = LOGGER.try_read().expect("Cannot lock LOGGER for low log");
        log.log(LoggingLevels::Low, author, content).await;
    }

    pub async fn medium(author: &str, content: &str) {
        let log = LOGGER
            .try_read()
            .expect("Cannot lock LOGGER for medium log");
        log.log(LoggingLevels::Medium, author, content).await;
    }

    pub async fn high(author: &str, content: &str) {
        let log = LOGGER.try_read().expect("Cannot lock LOGGER for high log");
        log.log(LoggingLevels::High, author, content).await;
    }

    pub async fn debug(author: &str, content: &str) {
        let log = LOGGER.try_read().expect("Cannot lock LOGGER for debug log");
        log.log(LoggingLevels::Debug, author, content).await;
    }

    pub async fn error(author: &str, content: &str) {
        let log = LOGGER.try_read().expect("Cannot lock LOGGER for error log");
        log.log(LoggingLevels::Error, author, content).await;
    }
}
