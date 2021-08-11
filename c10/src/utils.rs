use serde_derive::Deserialize;
use anyhow::Result;
use serenity::{
    http::Http,
    model::id::GuildId,
    model::id::ChannelId,
    model::gateway::Activity,
    prelude::*
    };
use std::{
    sync::{Arc},
};

use tracing::{info, debug};
use tokio::time::{sleep, Duration};
use thousands::Separable;
use invictus_api::*;

pub async fn update_nick_and_activity(ctx: Arc<Context>, fund_ticker: &str, guilds: Vec<GuildId>, update_frequency: u64, playing: &str) {
    let fund_name = normalize_fund_name(fund_ticker).expect("update_nick, Fund name unknown");
    loop {
        let perf_percent = match fund_perf(&fund_ticker, playing).await {
            Ok(percent) => percent,
            Err(e) => {
                info!("fund_perf {} api call failed\n{}", &fund_ticker, e.to_string());
                sleep(Duration::from_secs(update_frequency)).await;
                continue;
            },
        };
        let trend = if perf_percent.contains("-") {"⬂"} else {"⬀"};
            
        let mut fund_nav = fund_nav(&fund_name).await.unwrap_or("failed".into());
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

pub async fn c10_rebalance_check(http: &Http) {

    let mut previous_asset_values: Vec<FundPieAsset> = vec![];
    let rebalance_channels = vec![
        ChannelId(831545825753694229),
        ChannelId(832461214083973140),
        ChannelId(799268744890679377)]; //  rebalance channel IDs

    loop {
        let mut api_response = match FundPie::fund_pie("c10").await {
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
            let net_value = match FundNav::fund_nav("c10").await {
                Ok(value) => (value.net_asset_value().parse::<f64>().unwrap()) as i64,
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

            'rbchannels: for channel in &rebalance_channels {
                if let Err(why) = channel.say(http,  &rebalance_message).await {
                    info!("c10_rebalance_check failed to send alert\n{}", why);
                    sleep(Duration::from_secs(10)).await;
                    continue 'rbchannels;    
                };
                sleep(Duration::from_secs(2)).await;
            }

            previous_asset_values = control.current_values;
        }

        sleep(Duration::from_secs(360)).await;
    }
}

struct RebalanceControl {
    previous_values: Vec<FundPieAsset>,
    current_values: Vec<FundPieAsset>,
    movement_tolerance: f64,
    crypto_rebalanced: bool,
    cash_rebalanced: bool,
    value_moved_to: String,
    asset_summary: String
}

impl RebalanceControl {
    fn new(previous_values: Vec<FundPieAsset>, current_values: Vec<FundPieAsset>, movement_tolerance: f64) -> Self {
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
                self.asset_summary.push_str(&format!("**{} {}%** *new token in the fund*\n", asset_curr.ticker, asset_curr.percentage ));
            }
        }
    }
}


pub fn get_last_rebalance_info(ticker: &str) -> Result<RebalanceInfo> {
    let filename = format!("{}lrb.toml", ticker);
    let fund_rebalance_info_string = std::fs::read_to_string(filename).unwrap();
    let fund_rebalance_info: RebalanceInfo = toml::from_str(&fund_rebalance_info_string).unwrap();
    Ok(fund_rebalance_info)
}

#[derive(Debug, Deserialize)]
pub struct RebalanceInfo {
    pub date: String,
    pub lrb_type: String,
    pub stats: Vec<String>,
}