use crate::prelude::*;
use chrono::prelude::*;
use once_cell::sync::Lazy;
use serde::Deserialize;
use serenity::{
    builder::CreateMessage,
    client::Context,
    model::{channel::GuildChannel, id::ChannelId},
};
use std::backtrace::Backtrace;
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
    let cfg = CONFIG.try_read().expect("Cannot lock CONFIG for LOGGER");

    Arc::new(RwLock::new(Logger::new(
        cfg.logging.clone(),
        cfg.logging_template.clone(),
    )))
});

pub struct Logger {
    settings: HashMap<LoggingLevels, bool>,
    log_channel: Arc<RwLock<Option<GuildChannel>>>,
    notify: Arc<RwLock<Option<GuildChannel>>>,
    loggig_template: String,
}

impl Logger {
    fn new(settings: LoggingConfig, loggig_template: String) -> Self {
        Self {
            settings: settings.levels,
            log_channel: Arc::new(RwLock::new(None)),
            notify: Arc::new(RwLock::new(None)),
            loggig_template,
        }
    }

    pub async fn set_log_channel(ctx: &Context, channel: &ChannelId) {
        match get_guild().channels(&ctx.http).await {
            Ok(channels) => {
                LOGGER.write().await.log_channel =
                    Arc::new(RwLock::new(channels.get(channel).cloned()));

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

    pub async fn set_notify(ctx: &Context, channel_type: String, channel: ChannelId) {
        match channel_type.as_str() {
            "thread" => match fetch_thread(&ctx, channel) {
                Ok(channel) => {
                    LOGGER.write().await.notify = Arc::new(RwLock::new(Some(channel.clone())));

                    Self::debug(
                        "logger.set_notify",
                        &format!("the notify thread is set to {}", channel.id.get()),
                    )
                    .await;
                }
                Err(e) => {
                    Self::error(
                        "logger.set_notify",
                        &format!("Error while setting notify thread: {}", e.to_string()),
                    )
                    .await;
                }
            },
            "channel" => match fetch_channel(&ctx, channel) {
                Ok(channel) => {
                    LOGGER.write().await.notify = Arc::new(RwLock::new(Some(channel.clone())));

                    Self::debug(
                        "logger.set_notify",
                        &format!("the notify channel is set to {}", channel.id.get()),
                    )
                    .await;
                }
                Err(e) => {
                    Self::error(
                        "logger.set_notify",
                        &format!("Error while setting notify channel: {}", e.to_string()),
                    )
                    .await;
                }
            },
            _ => panic!("wrong notify channel type: {}", channel_type),
        }
    }

    async fn log(&self, level: LoggingLevels, author: &str, content: &str) {
        let prefix = match level {
            LoggingLevels::Low => "Low",
            LoggingLevels::Medium => "**Medium**",
            LoggingLevels::High => "**High**",
            LoggingLevels::Debug => "Debug",
            LoggingLevels::Error => "**Error**",
        };

        let utc: DateTime<Utc> = Utc::now();
        let time_prefix = &format!(
            "{}-{}-{}, {}:{}:{}",
            utc.year(),
            utc.month(),
            utc.day(),
            utc.hour(),
            utc.minute(),
            utc.second()
        );

        let logging_text = self
            .loggig_template
            .replace("%prefix%", prefix)
            .replace("%time%", time_prefix)
            .replace("%author%", author)
            .replace("%content%", content);

        let enabled = match self.settings.get(&level) {
            Some(enable) => *enable,
            None => true,
        };

        let log_channel = self.log_channel.read().await;

        if enabled {
            if let Some(channel) = &*log_channel {
                match channel
                    .send_message(
                        get_http(),
                        CreateMessage::new().content(logging_text.clone()),
                    )
                    .await
                {
                    Ok(_) => (),
                    Err(e) => Self::file_logging(
                        self.loggig_template
                            .replace("%prefix%", "Error")
                            .replace("%time%", time_prefix)
                            .replace("%author%", "logger.log")
                            .replace("%content%", e.to_string().as_str())
                            .replace("**", "")
                            .as_str(),
                    ),
                }
            }
        }

        Self::file_logging(logging_text.replace("**", "").as_str());

        if level == LoggingLevels::Error {
            if cfg!(debug_assertions) {
                panic!("{}\nat {}", logging_text, Backtrace::capture())
            } else {
                println!("{}", Backtrace::capture());
            }
        }
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

    pub async fn if_ok<T, E: ToString>(
        author: &str,
        error_message: &str,
        result: Result<T, E>,
    ) -> bool {
        match result {
            Ok(_) => true,
            Err(e) => {
                Self::error(author, &format!("{} - {}", error_message, e.to_string())).await;
                false
            }
        }
    }

    pub async fn low(author: &str, content: &str) {
        let log = LOGGER.read().await;
        log.log(LoggingLevels::Low, author, content).await;
    }

    pub async fn medium(author: &str, content: &str) {
        let log = LOGGER.read().await;
        log.log(LoggingLevels::Medium, author, content).await;
    }

    pub async fn high(author: &str, content: &str) {
        let log = LOGGER.read().await;
        log.log(LoggingLevels::High, author, content).await;
    }

    pub async fn debug(author: &str, content: &str) {
        let log = LOGGER.read().await;
        log.log(LoggingLevels::Debug, author, content).await;
    }

    pub async fn error(author: &str, content: &str) {
        let log = LOGGER.read().await;
        log.log(LoggingLevels::Error, author, content).await;
    }

    pub async fn notify(author: &str, content: &str) {
        let log = LOGGER.read().await;

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

        let builder = CreateMessage::new().content(format!(
            "**[{}]** ({}) <{}>: {}",
            loc!("notify-prefix"),
            time_prefix,
            author,
            content
        ));

        let notify_channel = log.notify.read().await;

        if let Some(notify) = &*notify_channel {
            notify.send_message(get_http(), builder).await.unwrap();
        }
    }
}
