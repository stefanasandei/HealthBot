#![allow(warnings)]

mod commands;
mod events;

use std::{
    collections::HashSet,
    env,
    sync::Arc,
};
use serenity::{
    async_trait,
    client::bridge::gateway::ShardManager,
    framework::{
        StandardFramework,
        standard::macros::group,
    },
    http::Http,
    model::{event::ResumedEvent, gateway::Ready, user::User, id::{RoleId, ChannelId, MessageId, GuildId}, channel::{Reaction, ReactionType::*},},
    prelude::*,
    framework::standard::{
        CommandResult,
        macros::command,
    }
};
use events::Handler;
use tracing::{error};
use tracing_subscriber::{
    FmtSubscriber,
    EnvFilter,
};
use commands::{
    math::*,
    meta::*,
    owner::*,
};

pub struct ShardManagerContainer;

impl TypeMapKey for ShardManagerContainer {
    type Value = Arc<Mutex<ShardManager>>;
}

#[group]
#[commands(multiply, ping, hello, quit)]
struct General;

#[tokio::main]
async fn main() {

    dotenv::dotenv().expect("Failed to load .env file");

    let subscriber = FmtSubscriber::builder()
        .with_env_filter(EnvFilter::from_default_env())
        .finish();

    tracing::subscriber::set_global_default(subscriber).expect("Failed to start the logger");


    let token = env::var("DISCORD_TOKEN")
        .expect("Expected a token in the environment");

    let http = Http::new_with_token(&token);

    let (owners, _bot_id) = match http.get_current_application_info().await {
        Ok(info) => {
            let mut owners = HashSet::new();
            owners.insert(info.owner.id);

            (owners, info.id)
        },
        Err(why) => panic!("Could not access application info: {:?}", why),
    };

    let framework = StandardFramework::new()
        .configure(|c| c
                   .owners(owners)
                   .prefix("$"))
        .group(&GENERAL_GROUP);

    let mut client = Client::builder(&token)
        .framework(framework)
        .event_handler(Handler)
        .await
        .expect("Err creating client");

    {
        let mut data = client.data.write().await;
        data.insert::<ShardManagerContainer>(client.shard_manager.clone());
    }

    let shard_manager = client.shard_manager.clone();

    tokio::spawn(async move {
        tokio::signal::ctrl_c().await.expect("Could not register ctrl+c handler");
        shard_manager.lock().await.shutdown_all().await;
    });

    if let Err(why) = client.start().await {
        error!("Client error: {:?}", why);
    }
}