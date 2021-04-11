use std::{
    collections::HashSet,
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
    model::{event::ResumedEvent, gateway::Ready, id::ChannelId},
    prelude::*,
};
use anyhow::Result;
use serde_derive::Deserialize;

use tracing::{error, info, debug};
use tracing_subscriber::{
    FmtSubscriber,
    filter::{EnvFilter, LevelFilter}
};

use serenity::model::gateway::Activity;
use serenity::model::id::GuildId;


mod commands;
use commands::{
    // math::*,
    // meta::*,
    owner::*,
    fund_commands::*,
};


#[group]
#[commands(quit, nav, full, mov)]
struct General;
pub struct ShardManagerContainer;

impl TypeMapKey for ShardManagerContainer {
    type Value = Arc<Mutex<ShardManager>>;
}

struct Handler;

#[async_trait]
impl EventHandler for Handler {
    async fn ready(&self, ctx: Context, ready: Ready) {
        info!("Connected as {}", ready.user.name);
        // let mut counter = 1;
        loop {
            let server = GuildId(830580361632153630); // test server guild id
            let mut status_message = nav_status().await;
            status_message.truncate(5);

            ctx.set_activity(Activity::playing(format!("24h {}%", api_c10_mov_time("24".into()).await.unwrap()))).await;
            server.edit_nickname(&ctx.http, Some(&format!("C10 NAV {}$", status_message))).await.unwrap();
            tokio::time::sleep(tokio::time::Duration::from_millis(60000)).await;
            // counter += 1;
        }
    }

    async fn resume(&self, _: Context, _: ResumedEvent) {
        debug!("Resumed");
    }
}

#[derive(Debug, Deserialize)]
struct Config {
    bot_token: String,
    prefix: String,
    allowed_channels: Vec<String>
}

#[tokio::main]
async fn main() {
    let config: Config = loadconfig().expect("Can't load config file: botconfig.toml. Please make sure you have one next to the executable and it's correct.");
    info!("Botconfig loaded {:?}", &config);

    let filter = EnvFilter::from_default_env()
        // Set the base level when not matched by other directives to INFO.
        .add_directive(LevelFilter::INFO.into());

    let subscriber = FmtSubscriber::builder()
        .with_env_filter(filter)
        .finish();

    tracing::subscriber::set_global_default(subscriber).expect("Failed to start the logger");

    let http = Http::new_with_token(&config.bot_token);

    // fetch bot owner ID
    let (owners, _bot_id) = match http.get_current_application_info().await {
        Ok(info) => {
            let mut owners = HashSet::new();
            owners.insert(info.owner.id);
            (owners, info.id)
        },
        Err(why) => panic!("Could not access application info: {:?}", why),
    };

    // Create the framework
    let framework = StandardFramework::new()
        .configure(|c| c
                   .owners(owners)
                   .prefix(&config.prefix))
        .group(&GENERAL_GROUP);

    let mut client = Client::builder(&config.bot_token)
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

// Loading bot config file.
fn loadconfig() -> Result<Config> {
    let configtoml = std::fs::read_to_string("botconfig.toml")?;
    let config: Config = toml::from_str(&configtoml)?;
    Ok(config)
}
