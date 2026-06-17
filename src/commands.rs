use crate::{Context, Error};
use log::info;
use poise::serenity_prelude::{self as serenity};
use redis::AsyncCommands;

#[poise::command(
    slash_command,
    install_context = "User",
    interaction_context = "Guild",
    description_localized("en-US", "Sets the channel on which the Confessor will respond")
)]
pub async fn set_channel(
    ctx: Context<'_>,
    #[description = "Server text channel"]
    #[channel_types("Text")]
    input_channel: serenity::GuildChannel,
) -> Result<(), Error> {
    let channel_id = input_channel.id.get();
    let mut conn = ctx.data().redis_client.clone();
    let _: () = conn
        .set("confessions:channel", channel_id.to_string())
        .await?;
    info!(
        "Setting channel {} as channel for confessions",
        input_channel.base.name
    );
    ctx.reply(format!("Set channel {}!", input_channel.base.name))
        .await?;
    Ok(())
}

#[poise::command(
    slash_command,
    install_context = "User",
    interaction_context = "Guild",
    description_localized("en-US", "Make your confession")
)]
pub async fn confess(
    ctx: Context<'_>,
    #[description = "Your confession"] confession: String,
) -> Result<(), Error> {
    ctx.defer_ephemeral().await?;
    let mut conn = ctx.data().redis_client.clone();
    let _: () = conn.lpush("confessions:queue", confession).await?;
    info!("A new confession was inscribed.");
    ctx.reply("Your confession was heard.").await?;
    Ok(())
}
