use std::usize;

use crate::Config;
use crate::ShardManagerContainer;

use serenity::framework::standard::{macros::command, Args, CommandResult};
use serenity::model::prelude::*;
use serenity::{prelude::*, utils::Colour};
use tokio::time::{sleep, Duration};
use anyhow::{Result, anyhow};
use serde_derive::{Serialize, Deserialize};

#[derive(Debug, Deserialize, Serialize)]
pub struct ApiFundsGeneral {
    status: String,
    data: Vec<FundGeneral>
}

#[derive(Debug, Deserialize, Serialize)]
pub struct FundGeneral {
    circulating_supply: String,
    net_asset_value: String,
    nav_per_token: String,
    name: String,
    ticker: String
}

#[derive(Debug, Deserialize, Serialize)]
pub struct Crypto10 {
    status: String,
    circulating_supply: String,
    net_asset_value: String,
    nav_per_token: String,
    name: String,
    ticker: String,
    price: String,
    assets: Vec<Asset>
}

#[derive(Debug, Deserialize, Serialize)]
pub struct Asset {
    name: String,
    ticker: String,
    usd_value: String
}

#[command]
pub async fn nav(ctx: &Context, msg: &Message, _args: Args) -> CommandResult {
    let api_response = api_general().await?;
    let funds_general_raw = api_response.data;
    let mut fund_found = false;
    for fund in funds_general_raw {
        if fund.name == "crypto10" {
            msg.channel_id.say(&ctx.http, format!("C10 nav ${}", fund.nav_per_token)).await?;
            fund_found = true;
        }
    }
    if !fund_found {
        msg.channel_id.say(&ctx.http, format!("Cannot find fund in received data")).await?;
    }
    Ok(())
}

#[command]
pub async fn full(ctx: &Context, msg: &Message, _args: Args) -> CommandResult {
    let api_response = api_c10_full().await?;
    // let fund_raw = api_response.data;
    // let mut fund_found = false;
    msg.channel_id.say(&ctx.http, format!("C10 full ${:?}", api_response)).await?;
    Ok(())
}

#[command]
pub async fn mov(ctx: &Context, msg: &Message, _args: Args) -> CommandResult {
    let api_response = api_c10_mov().await?;
    msg.channel_id.say(&ctx.http, format!("C10 fund movements {} |", api_response)).await?;
    Ok(())
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

pub async fn api_general() -> Result<ApiFundsGeneral> {
    let api_response = reqwest::get("https://api.invictuscapital.com/v2/funds")
        .await?
        .json::<ApiFundsGeneral>()
        .await?;
    // println!("{:?}", api_response);

    // let funds: funds = api_response;
    Ok(api_response)
}

pub async fn api_c10_full() -> Result<Crypto10> {
    let api_response = reqwest::get("https://api.invictuscapital.com/v2/funds/crypto10/nav")
        .await?
        .json::<Crypto10>()
        .await?;
    // println!("{:?}", api_response);

    // let funds: funds = api_response;
    Ok(api_response)
}

pub async fn api_c10_mov() -> Result<String> {
    let time_ranges = vec!["1", "12", "24"];
    let mut movements = String::new();

    for range in time_ranges {
        let fund_movement = reqwest::get(format!("https://api.invictuscapital.com/v2/funds/crypto10/movement?range={}h", range))
        .await?
        .json::<FundMovement>()
        .await?;
        movements.push_str(&format!("| {}h {}% ", range, fund_movement.percentage));
    }
    
    Ok(movements)
}

pub async fn api_c10_mov_time(timeframe: String) -> Result<String> {
    let fund_movement = reqwest::get(format!("https://api.invictuscapital.com/v2/funds/crypto10/movement?range={}h", timeframe))
        .await?
        .json::<FundMovement>()
        .await?;
    Ok(fund_movement.percentage)
}
#[derive(Debug, Deserialize, Serialize)]
pub struct FundMovement {
    status: String,
    percentage: String
}
