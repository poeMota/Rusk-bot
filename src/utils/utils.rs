use serenity::{
    all::RoleId,
    builder::{CreateActionRow, CreateButton},
    client::Context,
    http::Http,
    model::{
        application::ButtonStyle,
        channel::GuildChannel,
        guild::Member,
        id::{ChannelId, GuildId, UserId},
    },
};

use crate::{
    config::{load_env, CONFIG},
    prelude::*,
};
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
    *GUILD
}

pub async fn fetch_member(id: &UserId) -> Result<Member, serenity::Error> {
    let http = get_http();
    let guild = get_guild();

    guild.member(http, id).await
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

pub fn fetch_thread(ctx: &Context, id: ChannelId) -> Result<GuildChannel, String> {
    let guild = match get_guild().to_guild_cached(&ctx.cache) {
        Some(g) => g,
        None => {
            return Err("cannot get guild from id".to_string());
        }
    };

    for thread in guild.threads.iter() {
        if thread.id == id {
            return Ok(thread.clone());
        }
    }

    Err("cannot get channel by id".to_string())
}

pub fn get_params_buttons(name: &str, params: Vec<&str>) -> Vec<CreateActionRow> {
    let mut buttons = Vec::new();
    for param in params.iter() {
        buttons.push(CreateActionRow::Buttons(Vec::from([
            CreateButton::new(format!("{}:{}-label", name, param))
                .label(get_string(&format!("{}-{}-label", name, param), None))
                .style(ButtonStyle::Secondary)
                .disabled(true),
            CreateButton::new(format!("{}:{}", name, param))
                .emoji('🛠')
                .style(ButtonStyle::Success),
        ])));
    }
    buttons
}

pub async fn get_highest_role_in(
    ctx: &Context,
    user: UserId,
    roles: &Vec<RoleId>,
) -> Result<Option<RoleId>, String> {
    let mut highest_role: Option<&Role> = None;
    let member = fetch_member(&user).await.map_err(|e| e.to_string())?;

    let guild = match get_guild().to_guild_cached(&ctx.cache) {
        Some(g) => g,
        None => {
            return Err("cannot get guild from id".to_string());
        }
    };

    for role_id in roles.iter() {
        if member.roles.contains(&role_id) {
            if let Some(role) = guild.roles.get(&role_id) {
                if let Some(highest) = &highest_role {
                    if role.position < highest.position {
                        highest_role = Some(role)
                    }
                } else {
                    highest_role = Some(role);
                }
            }
        }
    }

    Ok(match highest_role {
        Some(role) => Some(role.id),
        None => None,
    })
}
