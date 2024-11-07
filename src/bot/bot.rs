use serenity::{http::Http, model::id::GuildId};

use crate::config::{load_env, CONFIG};
use once_cell::sync::Lazy;
use std::env;

static HTTP: Lazy<Http> = Lazy::new(|| {
    load_env();
    Http::new(&env::var("TOKEN").unwrap())
});

pub fn get_http() -> &'static Http {
    &HTTP
}

static GUILD: Lazy<GuildId> = Lazy::new(|| GuildId::new(CONFIG.try_read().unwrap().guild));

pub fn get_guild() -> GuildId {
    GUILD.clone()
}
