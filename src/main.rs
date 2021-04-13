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
    model::{event::ResumedEvent, gateway::Ready, gateway::Activity, id::GuildId},
    prelude::*,
};

use tracing::{error, info, debug};
use tracing_subscriber::{
    FmtSubscriber,
    filter::{EnvFilter, LevelFilter}
};

mod commands;
use commands::{
    owner::*,
    fund_commands::*,
    invictus_api::*
};

mod utils;


#[group]
#[commands(quit, nav, /*full,*/ mov, stats)]
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
        loop {
            ctx.set_activity(Activity::playing(format!("24h {}%", api_c10_mov_time("24".into()).await.unwrap()))).await;
            tokio::time::sleep(tokio::time::Duration::from_secs(300)).await;
        }
    }

    async fn resume(&self, _: Context, _: ResumedEvent) {
        debug!("Resumed");
    }
}


#[tokio::main]
async fn main() {
    let config: utils::Config = utils::loadconfig().expect("Can't load config file: botconfig.toml. Please make sure you have one next to the executable and it's correct.");
    info!("Botconfig loaded {:?}", &config);

    let filter = EnvFilter::from_default_env()
        .add_directive(LevelFilter::INFO.into());// Set the base level when not matched by other directives to INFO.

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

    // Start nicname update with nav
    tokio::spawn(async move {
        utils::update_nick(&http).await;
    });
    let http2 = Http::new_with_token(&config.bot_token);

    tokio::spawn(async move {
        utils::c10_rebalance_check(&http2).await;
    });

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
