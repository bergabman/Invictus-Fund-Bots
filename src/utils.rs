use serde_derive::Deserialize;
use anyhow::Result;
use serenity::{
    http::Http,
    model::id::GuildId,
    model::id::ChannelId,
    };

use tracing::{info, debug};
use tokio::time::{sleep, Duration};
use thousands::Separable;
use crate::commands::invictus_api::*;

pub async fn update_nick(http: &Http) {
    let servers = vec![830580361632153630, 394318139036270596];
    loop {
        let mut status_message = nav_status().await;
        status_message.truncate(5);
        for server in servers.clone() {
            let current_server = GuildId(server); // test server guild id
            current_server.edit_nickname(&http, Some(&format!("C10 NAV {}$", status_message))).await.unwrap(); // requires edit own nick permission
        }
        sleep(Duration::from_secs(60)).await;
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
    let rebalance_channels = ChannelId(831545825753694229); //  rebalance channel ID in test server
    // let rebalance_channel = ChannelId(830739714054291486); //  bot-test channel ID in test server
    let rebalance_channels = vec![ChannelId(831545825753694229), ChannelId(832461214083973140)]; //  rebalance channel ID in test server

    loop {
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

        // Rebalance control struct to hold all the data needed for the check
        let mut control = RebalanceControl::new(
            previous_asset_values.clone(),
            current_asset_values, 
            10.0,
        );

        control.run();

        if control.cash_rebalanced || control.crypto_rebalanced {
            let net_value = match api_c10_full().await {
                Ok(value) => (value.net_fund_value().parse::<f64>().unwrap()) as i64,
                Err(_) => 0,
            };
            let mut rebalance_message = String::new();
            rebalance_message.push_str(&format!(":tada:**  C10 Rebalanced!  **:tada:\n**Fund Net Value: **${}\n", net_value.separate_with_commas()));
            if control.cash_rebalanced {
                rebalance_message.push_str(&format!("*{}*\n", control.value_moved_to ));
            } else {
                rebalance_message.push_str(&format!("*Crypto assets rebalanced*\n",));
            }

            rebalance_message.push_str(&format!("{}", control.asset_summary));

            for channel in &rebalance_channels {
                if let Err(why) = channel.say(http,  &rebalance_message).await {
                    info!("c10_rebalance_check failed to send alert\n{}", why);
                    sleep(Duration::from_secs(10)).await;
                    continue;    
                };
            }

            previous_asset_values = control.current_values;
        }

        sleep(Duration::from_secs(5)).await;
    }
}

struct RebalanceControl {
    previous_values: Vec<PieAsset>,
    current_values: Vec<PieAsset>,
    movement_tolerance: f64,
    crypto_rebalanced: bool,
    cash_rebalanced: bool,
    value_moved_to: String,
    asset_summary: String
}

impl RebalanceControl {
    fn new (previous_values: Vec<PieAsset>, current_values: Vec<PieAsset>, movement_tolerance: f64) -> Self {
        Self {
            previous_values,
            current_values,
            movement_tolerance,
            crypto_rebalanced: false,
            cash_rebalanced: false,
            value_moved_to: "".to_string(),
            asset_summary: "".to_string()
        }
    }

    fn run(&mut self) {
        for asset_curr in &self.current_values {
            let mut asset_found = false;
            for asset_prev in &self.previous_values {
                if asset_curr.name == asset_prev.name {
                    asset_found = true;
                    let current_percentage:f64 = asset_curr.percentage.parse().unwrap();
                    let previous_percentage:f64 = asset_prev.percentage.parse().unwrap();
                    
                    if (current_percentage - self.movement_tolerance) > previous_percentage {           // asset allocation increased compared to previous dataset
                        if asset_curr.ticker == "USD" {
                            self.cash_rebalanced = true;
                            self.value_moved_to = "Value moved to Cash".into()
                        } else {
                            self.crypto_rebalanced = true;
                        }
    
                    } else if (current_percentage + self.movement_tolerance) < previous_percentage {    // asset allocation decreased compared to previous dataset
                        if asset_curr.ticker == "USD" {
                            self.cash_rebalanced = true;
                            self.value_moved_to = "Value moved to Cryptocurrencies".into()
                        } else {
                            self.crypto_rebalanced = true;
                        }
                    }
                    debug!("**{} {}%** *was {}%*\n",asset_curr.ticker, current_percentage, previous_percentage);
                    self.asset_summary.push_str(&format!("**{} {}%** *(before {}%*)\n",asset_curr.ticker, current_percentage, previous_percentage));
                }
            }
            if !asset_found { // if we can't find one of the assets in the previous dataset that is part of the fund now, we can assume that a rebalance happened 
                self.crypto_rebalanced = true;
                self.asset_summary.push_str(&format!("**{}** new token in the fund *{}%*\n", asset_curr.ticker, asset_curr.percentage ));
            }
        }
    }
}