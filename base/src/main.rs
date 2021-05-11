use std::{
    collections::HashSet,
    sync::{Arc, atomic::{Ordering, AtomicBool}},
};

use serenity::{
    async_trait,
    client::bridge::gateway::ShardManager,
    framework::{
        StandardFramework,
        standard::macros::group,
    },
    http::Http,
    model::{event::ResumedEvent, gateway::Ready, id::GuildId},
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
    // fund_commands::*,
};
mod utils;

#[group]
#[commands(quit)]
struct General;
pub struct ShardManagerContainer;

impl TypeMapKey for ShardManagerContainer {
    type Value = Arc<Mutex<ShardManager>>;
}

struct Handler {
    is_loop_running: AtomicBool,
}

#[async_trait]
impl EventHandler for Handler {
    async fn ready(&self, _ctx: Context, ready: Ready) {
        info!("Connected as {}", ready.user.name);
    }

    async fn resume(&self, _: Context, _: ResumedEvent) {
        debug!("Resumed");
    }

    async fn cache_ready(&self, ctx: Context, guilds: Vec<GuildId>) {
        let ctx = Arc::new(ctx.clone());
       
        // We need to check that the loop is not already running when this event triggers,
        // as this event triggers every time the bot enters or leaves a guild, along every time the
        // ready shard event triggers.
        //
        // An AtomicBool is used because it doesn't require a mutable reference to be changed, as
        // we don't have one due to self being an immutable reference.
        if !self.is_loop_running.load(Ordering::Relaxed) {
            let ( fund_ticker, update_frequency, playing) = {
                let data_read_lock = ctx.data.read().await;
                let config = data_read_lock.get::<utils::Config>().expect("Expected Config in TypeMap.");
                (config.fund_ticker.clone(), config.update_frequency.clone(), config.playing.clone())
    
            };
            // let (fund_ticker_clone, update_frequency_clone) = (fund_ticker.clone(), update_frequency.clone());

            // We have to clone the Arc, as it gets moved into the new thread.
            // let ctx1 = Arc::clone(&ctx);
            // tokio::spawn(async move {
            //     utils::update_activity(Arc::clone(&ctx1), &fund_ticker, update_frequency).await;
            // });

            let ctx2 = Arc::clone(&ctx);
            tokio::spawn(async move {
                utils::update_nick_and_activity(Arc::clone(&ctx2), &fund_ticker, guilds, update_frequency, &playing).await;
            });

            // Now that the loop is running, we set the bool to true
            self.is_loop_running.swap(true, Ordering::Relaxed);
        }
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

    // Create the framework
    let framework = StandardFramework::new()
        .configure(|c| c
                    .owners(owners)
                    .prefix(&config.prefix)
                    .allowed_channels(config.allowed_channels.clone().into_iter().collect()))
        .group(&GENERAL_GROUP);

    let mut client = Client::builder(&config.bot_token)
        .framework(framework)
        .event_handler(Handler {
            is_loop_running: AtomicBool::new(false),
        })
        .await
        .expect("Err creating client");

    {
        let mut data = client.data.write().await;
        data.insert::<ShardManagerContainer>(client.shard_manager.clone());
        data.insert::<utils::Config>(config);
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
