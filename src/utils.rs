use serde_derive::Deserialize;
use anyhow::Result;
use serenity::{
    http::Http,
    model::id::GuildId,
    model::id::ChannelId,
    // model::gateway::Activity,
    };

use tracing::{error, info, debug};
use tokio::time::{sleep, Duration};
use thousands::Separable;
use crate::commands::invictus_api::*;

pub async fn update_nick(http: &Http) {
    loop {
        let server = GuildId(830580361632153630); // test server guild id
        let mut status_message = nav_status().await;
        status_message.truncate(5);
        server.edit_nickname(&http, Some(&format!("C10 NAV {}$", status_message))).await.unwrap(); // requires edit own nick permission
        sleep(Duration::from_millis(60000)).await;
    }
}

pub async fn nav_status() -> String {
    let mut return_value = String::new();

    let api_response = match api_general().await {
        Ok(response) => response,
        Err(_) => return "api_problem".into(),
    };
    let funds_general_raw = api_response.data;
    let mut fund_found = false;
    for fund in funds_general_raw {
        if fund.name == "crypto10" {
            return_value = fund.nav_per_token;
            fund_found = true;
        }
    }
    if !fund_found {
        return_value = "not_found".into();
    }
    return_value
}

#[derive(Debug, Deserialize)]
pub struct Config {
    pub bot_token: String,
    pub prefix: String,
    allowed_channels: Vec<String>
}

// Loading bot config file.
pub fn loadconfig() -> Result<Config> {
    let configtoml = std::fs::read_to_string("botconfig.toml")?;
    let config: Config = toml::from_str(&configtoml)?;
    Ok(config)
}

pub async fn c10_rebalance_check(http: &Http) {

    let mut previous_values: Vec<PieAsset> = vec![];

    loop {
        let mut api_response = match api_c10_pie().await {
            Ok(response) => response,
            Err(e) => {
                info!("c10_rebalance_check failed to retrieve data from the api\n{}", e);
                sleep(Duration::from_secs(10)).await;
                continue;
            } 
        };
        api_response.remove_zero_asset();
        debug!("api response {:?}", &api_response);
        let current_values = api_response.assets.clone();
        debug!("current values {:?}", &current_values);
        if previous_values.is_empty() {
            previous_values = current_values.clone();
        }

        let (rebalanced, assetcomparison) = compare_assets(current_values.clone(), previous_values.clone());

        previous_values = current_values;
        if rebalanced {
            let net_value = match api_c10_full().await {
                Ok(value) => (value.net_fund_value().parse::<f64>().unwrap()) as i64,
                Err(_) => 0,
            };
            let channel = ChannelId(831545825753694229); //  rebalance channel ID in test server
            let mut summary = String::new();
            for asset in api_response.assets {
                summary.push_str(&format!("**{}**: {}%\n", asset.ticker, asset.percentage));
            }
            channel.say(http, format!(":tada:**  C10 Rebalanced!  **:tada:\nFund Net Value: **{}**\n{}",net_value.separate_with_commas(), assetcomparison)).await.unwrap();

        }
        sleep(Duration::from_secs(300)).await;
    }
}

fn compare_assets(previous_values: Vec<PieAsset>, current_values: Vec<PieAsset>) -> (bool, String) {
    // comparing current assets with previous dataset
    // filters: asset still in fund; amount of token difference bigger than 5%
    let mut rebalanced = false;
    let mut return_value = String::new();
    for asset_curr in current_values {
        let mut asset_found = false;
        for asset_prev in &previous_values {
            if asset_prev.name == asset_curr.name {
                asset_found = true;
                let current_amount:f64 = asset_curr.amount.parse().unwrap();
                let previous_amount:f64 = asset_prev.amount.parse().unwrap();
                // previous_amount = previous_amount + 10.0; // testing
                let compared = (current_amount / previous_amount * 100.0) as i64;
                if compared < 85 || compared > 115 { // if asset token amount differs with 5%, we assume that we rebalanced
                    rebalanced = true;
                }

                debug!("**{}** amount *({})*, **{}%** of previous *({})*",asset_curr.ticker, current_amount, compared, previous_amount);
                return_value.push_str(&format!("**{}** amount *({})*, **{}%** of previous *({})*\n",asset_curr.ticker, current_amount, compared, previous_amount));
            }
        }
        if !asset_found { // if we can't find one of the assets in the previous dataset that is part of the fund now, we can assume that we rebalanced
            rebalanced = true;
            return_value.push_str(&format!("**{}** new token in the fund *({})*\n", asset_curr.ticker, asset_curr.amount));
        }
    }

    (rebalanced, return_value)
}
