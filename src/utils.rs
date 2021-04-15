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

    let mut previous_asset_values: Vec<PieAsset> = vec![];
    let mut previous_cash_allocation: Option<f64> = None;
    // let rebalance_channel = ChannelId(831545825753694229); //  rebalance channel ID in test server
    let rebalance_channel = ChannelId(830739714054291486); //  bot-test channel ID in test server


    loop {
        let mut cash_rebalanced = false;
        let mut api_response = match api_c10_pie().await {
            Ok(response) => response,
            Err(e) => {
                info!("c10_rebalance_check failed to retrieve pie data from the api\n{}", e);
                sleep(Duration::from_secs(10)).await;
                continue;
            } 
        };
        api_response.remove_small_assets();
        debug!("api response {:?}", &api_response);
        let current_asset_values = api_response.assets.clone();
        debug!("current values {:?}", &current_asset_values);
        if previous_asset_values.is_empty() {
            previous_asset_values = current_asset_values.clone();
        }
        let c10_full = match api_c10_full().await {
            Ok(value) => value,
            Err(e) => {
                info!("c10_rebalance_check failed to retrieve full c10 data from the api\n{}", e);
                sleep(Duration::from_secs(10)).await;
                continue;
            },
        };
        // compare cash allocation
        let current_cash_allocation = api_response.get_cash_allocation_percent(c10_full.net_fund_value());
        if previous_cash_allocation.is_none() {
            previous_cash_allocation = current_cash_allocation;
        }

        let allowed_percentage_difference = 10.0;
        let mut cash_rebalance_string = String::from("Assets moved to ");
        if (previous_cash_allocation.unwrap() + allowed_percentage_difference) < current_cash_allocation.unwrap() {
            //moved to cash
            cash_rebalanced = true;
            cash_rebalance_string.push_str("Cash")

        } else if (previous_cash_allocation.expect("prev cash alloc1") - allowed_percentage_difference) > current_cash_allocation.expect("current cash 1") {
            //moved to crypto
            cash_rebalanced = true;
            cash_rebalance_string.push_str("Crypto")
        }
        // compare crypto assets
        let (crypto_rebalanced, comparison_summary) = compare_assets(current_asset_values.clone(), previous_asset_values.clone());
        previous_asset_values = current_asset_values.clone();

        if crypto_rebalanced || cash_rebalanced {
            let net_value = match api_c10_full().await {
                Ok(value) => (value.net_fund_value().parse::<f64>().unwrap()) as i64,
                Err(_) => 0,
            };
            let mut summary = String::new();
            summary.push_str(&format!(":tada:**  C10 Rebalanced!  **:tada:\n**Fund Net Value: **${}\n",net_value.separate_with_commas()));
            if cash_rebalanced {
                summary.push_str(&format!("*{}*\n",cash_rebalance_string));
            }
            for asset in api_response.assets {
                if asset.ticker != "USD" {
                    summary.push_str(&format!("**{}**: {}%\n", asset.ticker, asset.percentage));
                }
            }
            let usd_asset_value_string = current_asset_values.iter().filter(|f| f.ticker == "USD").next().unwrap();
            let usd_asset = usd_asset_value_string.value.parse::<f64>().unwrap() as i64;

            summary.push_str(&format!("**Cash allocation(USD):** {}% ${}", current_cash_allocation.unwrap(), usd_asset.separate_with_commas()));

            // if let Err(why) = rebalance_channel.say(http,  comparison_summary).await {
                if let Err(why) = rebalance_channel.say(http,  summary).await {
                info!("c10_rebalance_check failed to send alert\n{}", why);
                sleep(Duration::from_secs(10)).await;
                continue;    
            };

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
        if asset_curr.ticker == "USD" {
            continue;
        }
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
