use serenity::framework::standard::{macros::command, Args, CommandResult};
use serenity::model::prelude::*;
use serenity::prelude::*;
use crate::commands::invictus_api;
use thousands::Separable;

#[command]
pub async fn nav(ctx: &Context, msg: &Message, _args: Args) -> CommandResult {
    let api_response = invictus_api::api_general().await?;
    let funds_general_raw = api_response.data;
    let mut fund_found = false;
    for fund in funds_general_raw {
        if fund.name == "crypto10" {
            let mut nav = fund.nav_per_token;
            nav.truncate(5);
            msg.channel_id.say(&ctx.http, format!("*NAV:*\n**{}$**", nav)).await?;
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
    msg.channel_id.say(&ctx.http, format!("**Fund movements**\n{}", api_response)).await?;
    Ok(())
}

#[command]
pub async fn stats(ctx: &Context, msg: &Message, _args: Args) -> CommandResult {
    let mut api_response = invictus_api::api_c10_pie().await?;
    api_response.remove_zero_asset();

    let net_value = invictus_api::api_c10_full().await?;

    let fund_net_value: i64 = (net_value.net_fund_value().parse::<f64>().unwrap()) as i64;

    let mut summary = String::from(format!("**Fund Net Value**: ${}\n", fund_net_value.separate_with_commas()));
    for asset in api_response.assets {
        let asset_usd = (asset.value.parse::<f64>().unwrap()) as i64;
        summary.push_str(&format!("**{}**: {}% ${}\n", asset.ticker, asset.percentage, asset_usd.separate_with_commas()));
    }
    msg.channel_id.say(&ctx.http, format!("{}", summary)).await?;
    Ok(())
}

#[command]
pub async fn info(ctx: &Context, msg: &Message, _args: Args) -> CommandResult {

    let mut api_response = invictus_api::api_c10_pie().await?;
    api_response.remove_zero_asset();

    let net_value = invictus_api::api_c10_full().await?;

    let fund_net_value: i64 = (net_value.net_fund_value().parse::<f64>().unwrap()) as i64;


    let mut net_value = String::from(format!("**Fund Net Value**: ${}\n", fund_net_value.separate_with_commas()));
    let mut assets_vec = vec![];
    for asset in api_response.assets {
        let (one, two, three) = (format!("**{}**: {}% ${}\n", asset.ticker, asset.percentage, asset.value.separate_with_commas()), "", true);
        assets_vec.push((one, two, three));
    }
    let _ = msg.channel_id.send_message(&ctx.http, |m| {
        // m.content("test");
        m.embed(|mut e| {
            e.title(net_value);
            // e.description("With a description");
            e.thumbnail("https://cdn.discordapp.com/attachments/519973500535046148/831645350548734002/c10_.png");
            // e.image("https://cdn.discordapp.com/attachments/519973500535046148/831647125027684392/crypto10invictus.png");
            // e.fields(assets_vec);
            e.field("C10 Litepaper", "https://cdn.invictuscapital.com/whitepapers/c10-litepaper.pdf", false);
            e.field("ICAP litepaper for staking calculation", "https://cdn.invictuscapital.com/whitepapers/ICAP-Litepaper.pdf", false);
            e.field("ICAP dashboard by maaft:tm:", "https://www.invictusicap.org/", false);
            e.field("A hitchhiker's guide to a complete Invictus portfolio", "https://invictuscapital.com/en/article/a-hitchhikers-guide-to-a-complete-invictus-portfolio", false);
            
            e
        });
    
        m
    }).await?;
    // let mut api_response = invictus_api::api_c10_pie().await?;
    // api_response.remove_zero_asset();

    // let net_value = invictus_api::api_c10_full().await?;

    // let fund_net_value: i64 = (net_value.net_fund_value().parse::<f64>().unwrap()) as i64;

    // let mut summary = String::from(format!("**Fund Net Value**: $ {}\n", fund_net_value.separate_with_commas()));
    // for asset in api_response.assets {
    //     summary.push_str(&format!("**{}**: {}% ${}\n", asset.ticker, asset.percentage, asset.value.separate_with_commas()));
    // }
    // msg.channel_id.say(&ctx.http, format!("{}", summary)).await?;
    Ok(())
}

