use serenity::prelude::*;
use std::env;
use task_bot::{config::load_env, handler::Handler, localization, prelude::*, shop};
use tokio;

#[allow(unused_must_use)]
#[tokio::main]
async fn main() {
    CONFIG.read().await;
    localization::LOCALIZATION.read().unwrap();
    shop::SHOPMANAGER.write().await.init().await;
    member::MEMBERSMANAGER.write().await.init().await;
    task::TASKMANAGER.write().await.init().await;
    tag::TAGSMANAGER.write().await.init().await;
    project::PROJECTMANAGER.write().await.init().await;
    load_env();

    let token = env::var("TOKEN").unwrap();

    let mut client = Client::builder(token, GatewayIntents::all())
        .event_handler(Handler)
        .await
        .unwrap();

    if let Err(why) = client.start().await {
        Logger::error("main", &format!("Bot start error: {:?}", why)).await;
    }
}
