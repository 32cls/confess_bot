use crate::{Data, persistence::fetch_channel_id};
use futures::{StreamExt, stream};
use log::{error, info};
use poise::serenity_prelude::{Context as SerenityContext, CreateMessage, GenericChannelId};
use rand::{RngExt, distr::Uniform};
use redis::AsyncCommands;
use std::time::Duration;
use tokio::time;

const INTERVAL_IN_MINUTES: u64 = 5;

async fn confessions_task(ctx: &SerenityContext) {
    let mut conn = ctx.data::<Data>().redis_client.clone();
    let items: Vec<String> = match conn.lrange("confessions:queue", 0, -1).await {
        Ok(items) => items,
        Err(e) => {
            error!("Redis lrange error: {}", e);
            return;
        }
    };

    let count = items.len();

    if count == 0 {
        info!("No confession to send");
        return;
    }

    let chance = count as f64 / 100.0;
    let should_post = rand::random::<f64>() < chance.min(1.0);

    if !should_post {
        info!("Chance roll failed for {} confession(s)", count);
        return;
    }

    let idx;
    {
        let mut rng = rand::rng();
        let range = Uniform::new(0, count).unwrap();
        idx = rng.sample(range);
    }

    let confession = items[idx].clone();

    let _: isize = conn
        .lrem("confessions:queue", 1, &confession)
        .await
        .unwrap_or_else(|e| {
            error!("Redis lrem error: {}", e);
            -1
        });

    info!("Posting confession at index #{}", idx);
    send_confession_message(&ctx, confession).await;
}

async fn send_confession_message(ctx: &SerenityContext, confession: String) {
    let channel_id = fetch_channel_id(ctx).await;
    let builder = CreateMessage::new().content(confession);
    let _ = ctx
        .http
        .send_message(
            GenericChannelId::new(channel_id.get()),
            Vec::new(),
            &builder,
        )
        .await;
}

pub async fn start_confession_scheduler(ctx: &SerenityContext) {
    info!(
        "Starting scheduler to run every {} minute(s)",
        INTERVAL_IN_MINUTES
    );
    let interval = time::interval(Duration::from_secs(INTERVAL_IN_MINUTES * 60));
    let forever = stream::unfold(interval, |mut interval| async {
        interval.tick().await;
        confessions_task(ctx).await;
        Some(((), interval))
    });

    forever.for_each(|_| async {}).await;
}
