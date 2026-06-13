use dotenv::dotenv;
use poise::serenity_prelude::{self as serenity};
use serenity::{ChannelId, CreateMessage, GuildId};
use std::sync::Mutex;

type Error = Box<dyn std::error::Error + Send + Sync>;
type Context<'a> = poise::Context<'a, Data, Error>;

// User data, which is stored and accessible in all command invocations
pub struct Data {
    channel_id: Mutex<ChannelId>,
}

#[poise::command(
    slash_command,
    install_context = "User",
    interaction_context = "Guild",
    description_localized("en-US", "Sets the channel on which the Confessor will respond")
)]
async fn set_channel(
    ctx: Context<'_>,
    #[description = "Server text channel"]
    #[channel_types("Text")]
    channel: serenity::GuildChannel,
) -> Result<(), Error> {
    {
        let mut channel_id = ctx.data().channel_id.lock().unwrap();
        *channel_id = channel.id.clone();
    };
    ctx.reply(format!("Set channel {}!", channel.name)).await?;
    Ok(())
}

#[poise::command(
    slash_command,
    install_context = "User",
    interaction_context = "Guild",
    description_localized("en-US", "Make your confession")
)]
async fn confess(
    ctx: Context<'_>,
    #[description = "Your confession"] confession: String,
) -> Result<(), Error> {
    ctx.defer_ephemeral().await?;
    let builder = CreateMessage::new().content(confession);
    let id = ctx.data().channel_id.lock().unwrap().clone();
    let _ = id.send_message(&ctx.http(), builder).await;
    ctx.reply("Your confession was heard.").await?;
    Ok(())
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
            commands: vec![set_channel(), confess()],
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
                Ok(Data {
                    channel_id: Mutex::new(ChannelId::new(1)),
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

    client.unwrap().start().await.unwrap()
}
