use invictus_api::*;
use serenity::framework::standard::{macros::command, Args, CommandResult};
use serenity::model::prelude::*;
use serenity::prelude::*;

use thousands::Separable;

#[command]
pub async fn nav(ctx: &Context, msg: &Message, mut args: Args) -> CommandResult {
    
    let mut fund_to_check = String::new();
    let api_response = api_general().await?;
    let funds_general_raw = api_response.data;
    let mut fund_found = false;

    if args.len() == 0 {
        fund_to_check  = "crypto10".to_string();
    } else if args.len() == 1 {
        let arg = args.single::<String>()?;
        match normalize_fund_name(&arg) {
            Ok(checked_name) => {
                fund_to_check = checked_name;
            }
            Err(_) => {
                msg.reply(&ctx.http, "Unknown fund").await?;
                return Ok(())
            }
        };
    }
    
    for fund in funds_general_raw {
        if fund.name == fund_to_check {
            let mut nav = fund.nav_per_token;
            nav.truncate(5);
            msg.channel_id.say(&ctx.http, format!("***{} NAV:***\n**{}$**", fund_to_check, nav)).await?;
            fund_found = true;
        }
    }

    if !fund_found {
        msg.channel_id.say(&ctx.http, format!("Cannot find fund in received data")).await?;
    }
    Ok(())
}

#[allow(unused_assignments)]
#[command]
pub async fn stats(ctx: &Context, msg: &Message, mut args: Args) -> CommandResult {
    let mut fund_to_check = String::new();
    if args.len() == 0 {
        fund_to_check  = "crypto10".to_string();
    } else if args.len() == 1 {
        let arg = args.single::<String>()?;
        match normalize_fund_name(&arg) {
            Ok(checked_name) => {
                fund_to_check = checked_name;
            }
            Err(_) => {
                msg.reply(&ctx.http, "Unknown fund").await?;
                return Ok(())
            }
        };
    } else {
        msg.reply(&ctx.http, "Too many arguments, please check `-help`").await?;
        return Ok(())
    }
    
    let mut api_response = FundPie::fund_pie(&fund_to_check).await?;
    api_response.stablecoin_summary();
    api_response.remove_small_assets();
    let fund_nav = FundNav::fund_nav(&fund_to_check).await?;
    let fund_net_value = (fund_nav.net_asset_value().parse::<f64>().unwrap()) as i64;

    let mut summary = String::from(format!("*{}*\n", fund_to_check));
    summary.push_str(&format!("**Fund Net Value**: ${}\n", fund_net_value.separate_with_commas()));
    for asset in api_response.assets {
        let asset_usd = (asset.value.parse::<f64>().unwrap()) as i64;
        summary.push_str(&format!("**{} {}%** ${}\n", asset.ticker, asset.percentage, asset_usd.separate_with_commas()));
    }
    msg.channel_id.say(&ctx.http, format!("{}", summary)).await?;
    Ok(())
}

#[command]
pub async fn perf(ctx: &Context, msg: &Message, mut args: Args) -> CommandResult {
    
    let default_ranges = vec!["1h", "12h", "24h", "1w", "4w", "52w"];
    let mut fund_name = String::new();
    let mut range = String::new();
    let mut return_message = String::new();
    if args.len() == 0 {
        fund_name  = "crypto10".to_string();
        for range in default_ranges {
            let api_response = fund_perf(&fund_name, range).await?;
            return_message.push_str(&format!("**{} {}%**\n", range, api_response))
        }
    } else if args.len() == 1 {
        let arg = args.single::<String>()?;
        match normalize_fund_name(&arg) {
            Ok(checked_name) => {
                fund_name = checked_name;
                for range in default_ranges {
                    let api_response = fund_perf(&fund_name, range).await?;
                    return_message.push_str(&format!("**{} {}%**\n", range, api_response))
                }
            }
            Err(_) => {
                fund_name  = "crypto10".to_string();
                let api_response = fund_perf(&fund_name, &arg).await?;
                return_message.push_str(&format!("**{} {}%**\n", range, api_response));
            }
        };
    } else if args.len() == 2 {
        fund_name  = args.single::<String>()?;
        range  = args.single::<String>()?;

        match normalize_fund_name(&fund_name) {
            Ok(checked_name) => fund_name = checked_name,
            Err(_) => {
                msg.reply_ping(&ctx.http, format!("Sorry I didn't understand *{}*\n", fund_name)).await?;
                return Ok(())
            }
        };

        let api_response = fund_perf(&fund_name, &range).await?;
        return_message.push_str(&format!("**{} {}%**\n", range, api_response))
    }
    
    msg.channel_id.say(&ctx.http, format!("*{}*\n{}", fund_name, return_message)).await?;
    Ok(())

}

// #[command]
// pub async fn stake(ctx: &Context, msg: &Message, mut args: Args) -> CommandResult {
//     if !in_allowed_channels(&msg.channel_id.0 ) {
//         return Ok(())
//     }

//     if args.len() != 3 {
//         msg.reply_ping(&ctx.http, "I need 3 arguments for the calculation: <amount> <token> <length>").await?;
//         return Ok(())
//     }

//     msg.channel_id.say(&ctx.http, format!("** %** *()*\n",)).await?;
//     Ok(())

// }

#[command]
pub async fn info(ctx: &Context, msg: &Message, _args: Args) -> CommandResult {

    let fund_nav = FundNav::fund_nav("c10").await?;
    let net_value_raw: i64 = (fund_nav.net_asset_value().parse::<f64>().unwrap()) as i64;
    let net_value = String::from(format!("**Fund Net Value**: ${}\n", net_value_raw.separate_with_commas()));
    let _ = msg.channel_id.send_message(&ctx.http, |m| {
        m.embed(|e| {
            e.title(net_value);
            e.thumbnail("https://cdn.discordapp.com/attachments/754320391173832756/840839650376810506/c10_transparent.png");
            e.field("C10 litepaper", "https://cdn.invictuscapital.com/whitepapers/c10-litepaper.pdf", false);
            e.field("ICAP Litepaper", "https://cdn.invictuscapital.com/whitepapers/ICAP-Litepaper.pdf", false);
            e.field("ICAP dashboard by maaft:tm:", "https://www.invictusicap.org/", false);
            e.field("A hitchhiker's guide to a complete Invictus portfolio", "https://invictuscapital.com/en/article/a-hitchhikers-guide-to-a-complete-invictus-portfolio", false);
            e
        });
    
        m
    }).await?;
    Ok(())
}

#[command]
pub async fn help(ctx: &Context, msg: &Message, _args: Args) -> CommandResult {

    let mut perf_help = String::from("Fund performance, without arguments returns the past 1 year of C10 performance.\n");
    perf_help.push_str("`-perf <timerange>` returns the C10 fund preformance summary for the given timerange.\n");
    perf_help.push_str("`-perf <ticker>` returns the given fund preformance summary for the past 1 year.\n");
    perf_help.push_str("`-perf <ticker> <timerange>` returns the given fund performance summary for the given timerange.\n");
    perf_help.push_str("(ex):\n`-perf c20` for C20 fund performance summary of past 1 year\n");
    perf_help.push_str("`-perf c20 4w` for C20 fund performance summary of past 4 weeks \n");
    perf_help.push_str("`-perf iml 52w` for IML fund performance summary of past 52 weeks / 1 year \n");
    let _ = msg.channel_id.send_message(&ctx.http, |m| {
        m.embed(|e| {
            e.title("C10 bot help");
            e.description("C10 bot available commands, explanations and examples.");
            e.thumbnail("https://cdn.discordapp.com/attachments/754320391173832756/840839650376810506/c10_transparent.png");
            e.field("-help", "This help message.", false);
            e.field("-info", "Useful links.", false);
            e.field("-nav", "Current token value. \neg.: `-nav` `-nav <ticker>`", false);
            e.field("-stats", "Current fund asset allocation statistics. \neg.:`-stats` `-stats <ticker>`", false);
            e.field("-perf", perf_help, false);
            e
        });
        m
    }).await?;
    Ok(())
}

