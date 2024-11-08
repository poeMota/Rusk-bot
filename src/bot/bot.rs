use serenity::{
    http::Http,
    model::{
        guild::Member,
        id::{GuildId, UserId},
    },
};

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

pub async fn fetch_member(id: u64) -> Result<Member, serenity::Error> {
    let user_id = UserId::new(id);
    let http = get_http();
    let guild = get_guild();

    guild.member(http, user_id).await
}
