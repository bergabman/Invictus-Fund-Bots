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
pub async fn mov(ctx: &Context, msg: &Message, _args: Args) -> CommandResult {
    let api_response = invictus_api::api_c10_mov().await?;
    msg.channel_id.say(&ctx.http, format!("C10 fund movements {} |", api_response)).await?;
    Ok(())
}

#[command]
pub async fn stats(ctx: &Context, msg: &Message, _args: Args) -> CommandResult {
    let mut api_response = invictus_api::api_c10_pie().await?;
    let fund_net_value = invictus_api::api_c10_full().await?;
    api_response.remove_zero_asset();
    let mut summary = String::from(format!("**Fund Net Value**: {}\n", fund_net_value.net_fund_value()));
    for asset in api_response.assets {
        summary.push_str(&format!("**{}**: {}%\n", asset.ticker, asset.percentage));
    }
    msg.channel_id.say(&ctx.http, format!("{}", summary)).await?;
    Ok(())
}
