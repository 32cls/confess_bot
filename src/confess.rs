use crate::{Context, Data, Error};
use futures::{StreamExt, stream};
use poise::serenity_prelude::{self as serenity, GuildChannel, Http};
use rand::seq::IndexedRandom;
use serenity::CreateMessage;
use std::time::Duration;
use tokio::time;

const INTERVAL_IN_MINUTES: u64 = 10;

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
    {
        let data = ctx.data();
        let mut channel = data.channel.lock().unwrap();
        *channel = input_channel.clone();
    }
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
    ctx.data().confessions.lock().unwrap().push(confession);
    ctx.reply("Your confession was heard.").await?;
    Ok(())
}

async fn confessions_task(data: &Data, http: &Http) {
    let mut rng = rand::rng();
    let confessions = data.confessions.lock().unwrap().clone();
    let rnd_confession = confessions.choose(&mut rng);
    if rnd_confession.is_none() {
        println!("No confession to send");
        return;
    }
    let channel = data.channel.lock().unwrap().clone();
    send_confession_message(channel, http, rnd_confession.unwrap().into()).await;
}

async fn send_confession_message(channel: GuildChannel, http: &Http, confession: String) {
    let builder = CreateMessage::new().content(confession);
    let _ = channel.send_message(http, builder).await;
}

pub async fn start_confession_scheduler(data: &Data, http: &Http) {
    let interval = time::interval(Duration::from_mins(INTERVAL_IN_MINUTES));
    let forever = stream::unfold(interval, |mut interval| async {
        interval.tick().await;
        confessions_task(data, http).await;
        Some(((), interval))
    });

    forever.for_each(|_| async {}).await;
}
