use log::error;
use poise::serenity_prelude::ChannelId;
use poise::serenity_prelude::Context as SerenityContext;
use redis::AsyncCommands;

use crate::Data;

pub async fn fetch_channel_id(ctx: &SerenityContext) -> ChannelId {
    let mut conn = ctx.data::<Data>().redis_client.clone();
    let channel_id: u64 = match conn.get("confessions:channel").await {
        Ok(res) => res,
        Err(e) => {
            error!("Failed to fetch channel id: {}", e);
            return ChannelId::default();
        }
    };
    ChannelId::new(channel_id)
}
