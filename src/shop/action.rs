use crate::{
    prelude::*,
    shop::{Replacement, ShopManager},
};
use serde::Deserialize;
use serenity::{
    all::{GuildChannel, Timestamp, UserId},
    builder::CreateMessage,
    model::{
        application::ComponentInteraction,
        guild::{Member, Role},
        id::{ChannelId, RoleId},
    },
};
use std::{collections::HashMap, future::Future};

#[derive(Debug, Deserialize, Clone)]
#[serde(tag = "type", rename_all = "camelCase")]
pub enum ShopActions {
    GiveRoles(GiveRoles),
    RemoveRoles(RemoveRoles),
    SendMessage(SendMessage),
    Mute(Mute),
}

pub trait Action {
    fn call(&self, inter: ComponentInteraction) -> impl Future<Output = Result<(), String>>;

    fn convert(&mut self, shop_man: &ShopManager) -> impl Future<Output = Result<(), String>>;
}

#[derive(Debug, Deserialize, Clone)]
pub struct GiveRoles {
    #[serde(default)]
    member: Replacement,
    roles: Vec<Replacement>,
}

impl Action for GiveRoles {
    async fn call(&self, inter: ComponentInteraction) -> Result<(), String> {
        let member = match self.member.clone() {
            Replacement::Member(member) => member,
            Replacement::Nothing => inter.member.ok_or_else(|| "interaction have not member")?,
            _ => {
                return Err("kys".to_string());
            }
        };

        for role_repl in self.roles.iter() {
            if let Replacement::Role(role) = role_repl {
                if let Err(e) = member.add_role(get_http(), role.id).await {
                    return Err(format!(
                        "cannot give role {} because - {}",
                        role.name,
                        e.to_string()
                    ));
                } else {
                    return Err("kys".to_string());
                }
            }
        }
        Ok(())
    }

    async fn convert(&mut self, shop_man: &ShopManager) -> Result<(), String> {
        if let Replacement::Str(ref string) = self.member {
            self.member = shop_man.convert_string(string.clone());
        }

        if let Replacement::Nothing = self.member {
        } else {
            self.member = Replacement::Member(get_member(&self.member).await?);
        }

        let mut new_roles = Vec::new();
        for role in self.roles.iter_mut() {
            new_roles.push(Replacement::Role(get_role(&role).await?));
        }

        self.roles = new_roles;

        Ok(())
    }
}

#[derive(Debug, Deserialize, Clone)]
pub struct RemoveRoles {
    #[serde(default)]
    member: Replacement,
    roles: Vec<Replacement>,
}

impl Action for RemoveRoles {
    async fn call(&self, inter: ComponentInteraction) -> Result<(), String> {
        let member = match self.member.clone() {
            Replacement::Member(member) => member,
            Replacement::Nothing => inter.member.ok_or_else(|| "")?,
            _ => {
                return Err("kys".to_string());
            }
        };

        for role_repl in self.roles.iter() {
            if let Replacement::Role(role) = role_repl {
                if let Err(e) = member.remove_role(get_http(), role.id).await {
                    return Err(format!(
                        "cannot remove role {} because - {}",
                        role.name,
                        e.to_string()
                    ));
                } else {
                    return Err("kys".to_string());
                }
            }
        }
        Ok(())
    }

    async fn convert(&mut self, shop_man: &ShopManager) -> Result<(), String> {
        if let Replacement::Str(ref string) = self.member {
            self.member = shop_man.convert_string(string.clone());
        }

        if let Replacement::Nothing = self.member {
        } else {
            self.member = Replacement::Member(get_member(&self.member).await?);
        }

        let mut new_roles = Vec::new();
        for role in self.roles.iter_mut() {
            new_roles.push(Replacement::Role(get_role(&role).await?));
        }

        self.roles = new_roles;

        Ok(())
    }
}

#[derive(Debug, Deserialize, Clone)]
pub struct SendMessage {
    #[serde(default)]
    channel: Replacement,
    message: String,
}

impl Action for SendMessage {
    async fn call(&self, inter: ComponentInteraction) -> Result<(), String> {
        let replacements = HashMap::from([("AuthorPing", format!("<@{}>", inter.user.id))]);
        let mut message = self.message.clone();

        for (replacement, value) in replacements.iter() {
            if message.contains(&format!("<{}>", replacement)) {
                message = message.replace(&format!("<{}>", replacement), &value);
            }
        }

        let channel = match self.channel.clone() {
            Replacement::Channel(channel) => channel,
            Replacement::Nothing => match get_guild().channels(get_http()).await {
                Ok(channels) => match channels.get(&inter.channel_id) {
                    Some(channel) => channel.clone(),
                    None => {
                        return Err("cannot found channel from interaction".to_string());
                    }
                },
                Err(_) => {
                    return Err("cannot get guild channels, wtf".to_string());
                }
            },
            _ => {
                return Err("kys".to_string());
            }
        };

        match channel
            .send_message(get_http(), CreateMessage::new().content(message))
            .await
        {
            Ok(_) => Ok(()),
            Err(e) => Err(format!(
                "cannot send message in SendMessage action: {}",
                e.to_string()
            )),
        }
    }

    async fn convert(&mut self, shop_man: &ShopManager) -> Result<(), String> {
        self.message = match shop_man.convert_string(self.message.clone()) {
            Replacement::Str(string) => string,
            _ => {
                return Err(
                    "message field in sendMessage must be string or string replacement".to_string(),
                )
            }
        };
        if let Replacement::Str(ref string) = self.channel {
            self.channel = shop_man.convert_string(string.clone());
        }

        if let Replacement::Nothing = self.channel {
        } else {
            self.channel = Replacement::Channel(get_channel(&self.channel).await?);
        }

        Ok(())
    }
}

#[derive(Debug, Deserialize, Clone)]
pub struct Mute {
    #[serde(default)]
    member: Replacement,
    duration: i64,
}

impl Action for Mute {
    async fn call(&self, inter: ComponentInteraction) -> Result<(), String> {
        match Timestamp::from_unix_timestamp(Timestamp::now().unix_timestamp() + self.duration) {
            Ok(time) => {
                let mut member = match self.member.clone() {
                    Replacement::Member(mem) => mem,
                    Replacement::Nothing => {
                        inter.member.ok_or_else(|| "member field if not specified if mute action and cannot take member from interaction".to_string())?
                    },
                    _ => {
                        return Err("kys".to_string());
                    }
                };

                match member
                    .disable_communication_until_datetime(get_http(), time)
                    .await
                {
                    Ok(_) => Ok(()),
                    Err(e) => Err(format!(
                        "error while disabling member communication in mute action: {}",
                        e.to_string()
                    )
                    .to_string()),
                }
            }
            Err(_) => Err("invalid duration given in mute action".to_string()),
        }
    }

    async fn convert(&mut self, shop_man: &ShopManager) -> Result<(), String> {
        if let Replacement::Str(ref string) = self.member {
            self.member = shop_man.convert_string(string.clone());
        }

        if let Replacement::Nothing = self.member {
        } else {
            self.member = Replacement::Member(get_member(&self.member).await?);
        }

        Ok(())
    }
}

async fn get_member(content: &Replacement) -> Result<Member, String> {
    let http = get_http();
    let guild = match get_guild().to_partial_guild(&http).await {
        Ok(g) => g,
        Err(_) => return Err("Failed to fetch guild from API".to_string()),
    };

    match content {
        Replacement::Num(num) => match guild.member(&http, UserId::new(*num as u64)).await {
            Ok(member) => Ok(member.clone()),
            Err(e) => Err(e.to_string()),
        },
        Replacement::Member(member) => Ok(member.clone()),
        _ => Err("uncompatible type to convert into member".to_string()),
    }
}

async fn get_channel(content: &Replacement) -> Result<GuildChannel, String> {
    let http = get_http();
    let guild = match get_guild().to_partial_guild(&http).await {
        Ok(g) => g,
        Err(_) => return Err("Failed to fetch guild from API".to_string()),
    };

    match content {
        Replacement::Num(num) => {
            match guild
                .channels(&http)
                .await
                .map_err(|e| e.to_string())?
                .get(&ChannelId::new(*num as u64))
            {
                Some(channel) => Ok(channel.clone()),
                None => Err(format!("channel with {} id not found", num)),
            }
        }
        Replacement::Channel(channel) => Ok(channel.clone()),
        _ => Err("uncompatible type to convert into channel".to_string()),
    }
}

async fn get_role(content: &Replacement) -> Result<Role, String> {
    let http = get_http();
    let guild = match get_guild().to_partial_guild(&http).await {
        Ok(g) => g,
        Err(_) => return Err("Failed to fetch guild from API".to_string()),
    };

    match content {
        Replacement::Str(string) => {
            for (_, role) in guild.roles {
                if string.to_lowercase() == role.name.to_lowercase() {
                    return Ok(role);
                }
            }
            Err(format!("cannot find role with name {} in guild", string))
        }
        Replacement::Num(num) => match guild.roles.get(&RoleId::new(*num as u64)) {
            Some(role) => Ok(role.clone()),
            None => Err(format!("cannot find role with id {} in guild", num)),
        },
        Replacement::Role(role) => Ok(role.clone()),
        _ => Err("uncompatible type to convert into guild role".to_string()),
    }
}
