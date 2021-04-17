use crate::ShardManagerContainer;
use serenity::prelude::*;
use serenity::model::prelude::*;
use serenity::framework::standard::{
    CommandResult,
    macros::command,
};
use crate::commands::invictus_api::api_c10_mov_time;
use tokio::time::{sleep, Duration};
use tracing::info;


#[command]
#[owners_only]
async fn quit(ctx: &Context, msg: &Message) -> CommandResult {
    let data = ctx.data.read().await;
    if let Some(manager) = data.get::<ShardManagerContainer>() {
        msg.reply(ctx, "Shutting down!").await?;
        manager.lock().await.shutdown_all().await;
    } else {
        msg.reply(ctx, "There was a problem getting the shard manager").await?;
        return Ok(());
    }
    Ok(())
}

#[command]
#[owners_only]
pub async fn play(ctx: &Context, _msg: &Message) -> CommandResult {

    loop {
        let mov_percent = match api_c10_mov_time("24".into()).await {
            Ok(percent) => percent,
            Err(e) => {
                info!("nick_nav api call failed\n{}", e.to_string());
                sleep(Duration::from_secs(300)).await;
                continue;
            },
        };

        ctx.set_activity(Activity::playing(format!("24h {}%", mov_percent))).await;
        sleep(Duration::from_secs(60)).await;
    }
}

