use serenity::{
    client::Context,
    http::Http,
    model::{
        channel::GuildChannel,
        guild::Member,
        id::{ChannelId, GuildId, UserId},
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

pub fn fetch_channel(ctx: &Context, id: ChannelId) -> Result<GuildChannel, String> {
    let guild = match get_guild().to_guild_cached(&ctx.cache) {
        Some(g) => g,
        None => {
            return Err("cannot get guild from id".to_string());
        }
    };

    match guild.channels.get(&id) {
        Some(channel) => Ok(channel.clone()),
        None => {
            return Err("cannot get channel by id".to_string());
        }
    }
}
