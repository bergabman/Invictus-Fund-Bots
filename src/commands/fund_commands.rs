use serenity::framework::standard::{macros::command, Args, CommandResult};
use serenity::model::prelude::*;
use serenity::prelude::*;
use crate::commands::invictus_api;

#[command]
pub async fn nav(ctx: &Context, msg: &Message, _args: Args) -> CommandResult {
    let api_response = invictus_api::api_general().await?;
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
    let api_response = invictus_api::api_c10_full().await?;
    msg.channel_id.say(&ctx.http, format!("C10 full ${:?}", api_response)).await?;
    Ok(())
}

#[command]
pub async fn mov(ctx: &Context, msg: &Message, _args: Args) -> CommandResult {
    let api_response = invictus_api::api_c10_mov().await?;
    msg.channel_id.say(&ctx.http, format!("C10 fund movements {} |", api_response)).await?;
    Ok(())
}

pub async fn nav_status() -> String {
    let mut return_value = String::new();

    let api_response = match invictus_api::api_general().await {
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

