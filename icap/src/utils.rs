use serde:: Serialize;
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

#[derive(Debug, Serialize, Deserialize)]
struct Post {
    id: Option<i32>,
    title: String,
    body: String,
    user_id: i32,
}

pub async fn update_nick_and_activity(ctx: Arc<Context>, fund_ticker: &str, guilds: Vec<GuildId>, update_frequency: u64, playing: &str) {
    // let fund_name = normalize_fund_name(fund_ticker).unwrap_or("NaN".into());
    loop {
        let perf_percent = "~";
        // let perf_percent = match fund_perf(&fund_ticker, "24h").await {
        //     Ok(percent) => percent,
        //     Err(e) => {
        //         info!("fund_perf {} api call failed\n{}", &fund_ticker, e.to_string());
        //         // sleep(Duration::from_secs(update_frequency)).await;
        //         // continue;
        //         "NaN".into()
        //     },
        // };
        let trend = if perf_percent.contains("-") {"⬂"} else {"⬀"};
            
        let mut fund_nav = match uniswap_icap().await {
            Ok(nav) => nav,
            Err(e) => {
                info!("uniswap {} api call failed\n{}", &fund_ticker, e.to_string());
                sleep(Duration::from_secs(update_frequency)).await;
                continue;
            },
        };
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

pub async fn uniswap_icap() -> Result<String> {
    
    let uniswap_url = "https://api.thegraph.com/subgraphs/name/uniswap/uniswap-v2";

    let uniswap_json: serde_json::Value = reqwest::Client::new()
        .post(uniswap_url)
        .json(&serde_json::json!({
            "operationName":"NavPrice",
            "variables":{"token_address":"0xd83c569268930fadad4cde6d0cb64450fef32b65","dai_token_address":"0x6b175474e89094c44da98b954eedeac495271d0f"},
                "query":"query NavPrice($token_address: String!, $dai_token_address: String!) {\n  pairs(where: {token0: $dai_token_address, token1: $token_address}) {\n    token0Price\n    __typename\n  }\n}\n"
                }))
        .send()
        .await?
        .json()
        .await?;
    // info!("{}", uniswap_json["data"]["pairs"][0]["token0Price"]);
    let mut icap_price = format!("{}",uniswap_json["data"]["pairs"][0]["token0Price"]);
    icap_price = icap_price.replace("\"", "");

    Ok(icap_price)
}
