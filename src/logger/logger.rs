use crate::config::{read_file, write_file, CONFIG, DATA_PATH};
use chrono::prelude::*;
use once_cell::sync::Lazy;
use serde::Deserialize;
use serenity::{
    builder::CreateMessage,
    client::Context,
    model::{
        channel::GuildChannel,
        id::{ChannelId, GuildId},
    },
};
use std::cmp::Eq;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::{runtime::Runtime, sync::Mutex};

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
    let config = Runtime::new()
        .unwrap()
        .block_on(async { CONFIG.lock().await });
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

    pub async fn set_log_channel(ctx: &Context, channel: u64) {
        let mut log = LOGGER.lock().await;

        if let Ok(channels) = GuildId::new(channel).channels(&ctx.http).await {
            log.log_channel = channels.get(&ChannelId::new(channel)).cloned();
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

    pub async fn expect<T: Clone, E: Clone + ToString>(
        ctx: &Context,
        author: &str,
        expected: Result<T, E>,
    ) -> Result<T, E> {
        if let Err(ref error) = expected {
            Self::error(&ctx, author, error.to_string().as_str()).await
        }
        expected
    }

    pub async fn low(ctx: &Context, author: &str, content: &str) {
        let log = LOGGER.lock().await;
        log.log(&ctx, LoggingLevels::Low, author, content).await;
    }

    pub async fn medium(ctx: &Context, author: &str, content: &str) {
        let log = LOGGER.lock().await;
        log.log(&ctx, LoggingLevels::Medium, author, content).await;
    }

    pub async fn high(ctx: &Context, author: &str, content: &str) {
        let log = LOGGER.lock().await;
        log.log(&ctx, LoggingLevels::High, author, content).await;
    }

    pub async fn debug(ctx: &Context, author: &str, content: &str) {
        let log = LOGGER.lock().await;
        log.log(&ctx, LoggingLevels::Debug, author, content).await;
    }

    pub async fn error(ctx: &Context, author: &str, content: &str) {
        let log = LOGGER.lock().await;
        log.log(&ctx, LoggingLevels::Error, author, content).await;
    }
}
