mod confess;

use dotenv::dotenv;
use poise::serenity_prelude::{self as serenity, GuildChannel};
use serenity::GuildId;
use std::sync::Mutex;

type Error = Box<dyn std::error::Error + Send + Sync>;
type Context<'a> = poise::Context<'a, Data, Error>;

pub struct Data {
    channel: Mutex<GuildChannel>,
    confessions: Mutex<Vec<String>>,
}

#[poise::command(prefix_command, owners_only)]
async fn register_commands(ctx: Context<'_>) -> Result<(), Error> {
    let guild_id = GuildId::new(
        std::env::var("GUILD_ID")
            .unwrap()
            .parse::<u64>()
            .expect("Error while parsing guild id"),
    );

    let commands = &ctx.framework().options().commands;
    poise::builtins::register_in_guild(ctx.http(), commands, guild_id).await?;
    ctx.say("Successfully registered slash commands!").await?;
    confess::start_confession_scheduler(&ctx.data(), ctx.http()).await;
    Ok(())
}

#[tokio::main]
async fn main() {
    dotenv().ok();
    tracing_subscriber::fmt::init();

    let options = poise::FrameworkOptions {
        commands: vec![confess::set_channel(), confess::confess()],
        ..Default::default()
    };

    let token = serenity::Token::from_env("DISCORD_TOKEN").unwrap();
    let intents =
        serenity::GatewayIntents::non_privileged() | serenity::GatewayIntents::MESSAGE_CONTENT;

    let client = serenity::ClientBuilder::new(token, intents)
        .framework(Box::new(poise::Framework::new(options)))
        .await;

    client.unwrap().start().await.unwrap();
}
