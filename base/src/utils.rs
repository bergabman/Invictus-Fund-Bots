use serde_derive::Deserialize;
use anyhow::Result;
use serenity::{
    model::id::GuildId,
    model::id::ChannelId,
    model::gateway::Activity,
    prelude::*
    };
use std::{
    sync::{Arc},
};

use tracing::{info, /*debug*/};
use tokio::time::{sleep, Duration};
use invictus_api as invictus;
use uniswap_v2_api as uniswap;

#[allow(unused_assignments)]
pub async fn update_nick_and_activity(ctx: Arc<Context>, fund_ticker: &str, guilds: Vec<GuildId>, update_frequency: u64, playing: &str) {
    let mut perf_percent = String::new();
    let mut fund_nav = String::new();
    loop {
        if fund_ticker.to_lowercase() == "icap" {
            perf_percent = match uniswap::fund_perf(uniswap::ICAP, playing).await {
                Ok(percent) => percent,
                Err(e) => {
                    info!("fund_perf {} uniswap api call failed\n{}", &fund_ticker, e.to_string());
                    sleep(Duration::from_secs(update_frequency)).await;
                    continue;
                },
            };
            fund_nav = uniswap::fund_nav(&fund_ticker).await.unwrap_or("failed".into());

        } else {
            if let Err(why) = invictus::normalize_fund_name(fund_ticker) {
                info!("fund name {} normalize failed\n{}", &fund_ticker, why.to_string());
                    sleep(Duration::from_secs(update_frequency)).await;
                    continue;
            }
            perf_percent = match invictus::fund_perf(fund_ticker, playing).await {
                Ok(percent) => percent,
                Err(e) => {
                    info!("fund_perf {} invictus api call failed\n{}", &fund_ticker, e.to_string());
                    sleep(Duration::from_secs(update_frequency)).await;
                    continue;
                },
            };
            fund_nav = invictus::fund_nav(&fund_ticker).await.unwrap_or("failed".into());
        }
        // let perf_percent = match fund_perf(&fund_ticker, playing).await {
        //     Ok(percent) => percent,
        //     Err(e) => {
        //         info!("fund_perf {} api call failed\n{}", &fund_ticker, e.to_string());
        //         sleep(Duration::from_secs(update_frequency)).await;
        //         continue;
        //     },
        // };
        let trend = if perf_percent.contains("-") {"⬂"} else {"⬀"};
        //  = invictus::fund_nav(&fund_ticker).await.unwrap_or("failed".into());
        fund_nav.truncate(fund_nav.find(".").unwrap_or(1) + 4);

        for server in guilds.clone() {
            if let Err(e) = server.edit_nickname(&ctx.http, Some(&format!("{} ${} {}",fund_ticker, fund_nav, trend))).await {
                info!("{} failed to update nick with nav\n{}", &fund_ticker, e);
                sleep(Duration::from_secs(update_frequency)).await;
                continue;
            }
        }
        ctx.set_activity(Activity::playing(format!("{} {}%", playing, perf_percent))).await;
        sleep(Duration::from_secs(update_frequency)).await;
    }
}

impl TypeMapKey for Config {
    type Value = Config;
}
#[derive(Debug, Deserialize)]
pub struct Config {
    pub fund_ticker: String,
    pub bot_token: String,
    pub update_frequency: u64,
    pub playing: String,
    pub prefix: String,
    pub allowed_channels: Vec<ChannelId>
}

// Loading bot config file.
pub fn loadconfig() -> Result<Config> {
    let configtoml = std::fs::read_to_string("botconfig.toml")?;
    let config: Config = toml::from_str(&configtoml)?;
    Ok(config)
}
