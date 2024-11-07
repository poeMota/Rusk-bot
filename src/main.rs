use serenity::prelude::*;
use std::env;
use task_bot::{config::load_env, handler::Handler};
use tokio;

#[tokio::main]
async fn main() {
    load_env();
    let token = env::var("TOKEN").unwrap();

    let mut client = Client::builder(token, GatewayIntents::all())
        .event_handler(Handler)
        .await
        .unwrap();

    if let Err(why) = client.start().await {
        println!("Bot start error: {:?}", why);
    }
}
