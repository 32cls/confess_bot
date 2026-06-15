mod confess;

use dotenv::dotenv;
use poise::serenity_prelude::{self as serenity};
use serenity::{ChannelId, GuildId};
use std::sync::Mutex;

type Error = Box<dyn std::error::Error + Send + Sync>;
type Context<'a> = poise::Context<'a, Data, Error>;

pub struct Data {
    channel_id: Mutex<ChannelId>,
    confessions: Mutex<Vec<String>>,
}

#[tokio::main]
async fn main() {
    dotenv().ok();
    tracing_subscriber::fmt::init();
    let guild_id = std::env::var("GUILD_ID")
        .unwrap()
        .parse::<u64>()
        .expect("Error while parsing guild id");

    let framework = poise::Framework::builder()
        .options(poise::FrameworkOptions {
            commands: vec![confess::set_channel(), confess::confess()],
            on_error: |error| {
                Box::pin(async move {
                    println!("what the hell");
                    match error {
                        poise::FrameworkError::ArgumentParse { error, .. } => {
                            if let Some(error) = error.downcast_ref::<serenity::RoleParseError>() {
                                println!("Found a RoleParseError: {:?}", error);
                            } else {
                                println!("Not a RoleParseError :(");
                            }
                        }
                        other => poise::builtins::on_error(other).await.unwrap(),
                    }
                })
            },
            ..Default::default()
        })
        .setup(move |ctx, _ready, framework| {
            Box::pin(async move {
                poise::builtins::register_in_guild(
                    ctx,
                    &framework.options().commands,
                    GuildId::new(guild_id),
                )
                .await?;
                confess::start_confession_scheduler(ctx.data.clone(), ctx.http.clone());
                Ok(Data {
                    channel_id: Mutex::new(ChannelId::new(1)),
                    confessions: Mutex::new(Vec::new()),
                })
            })
        })
        .build();

    let token = std::env::var("DISCORD_TOKEN").unwrap();
    let intents =
        serenity::GatewayIntents::non_privileged() | serenity::GatewayIntents::MESSAGE_CONTENT;

    let client = serenity::ClientBuilder::new(token, intents)
        .framework(framework)
        .await;

    client.unwrap().start().await.unwrap();
}
