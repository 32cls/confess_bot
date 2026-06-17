mod commands;
mod persistence;
mod scheduler;

use dotenv::dotenv;
use log::{error, info};
use poise::{
    PrefixFrameworkOptions,
    serenity_prelude::{
        self as serenity, Context as SerenityContext, EventHandler, FullEvent, async_trait,
    },
};
use serenity::GuildId;
use std::sync::Arc;

type Error = Box<dyn std::error::Error + Send + Sync>;
pub type Context<'a> = poise::Context<'a, Data, Error>;

pub struct Data {
    pub redis_client: redis::aio::ConnectionManager,
}

struct Handler {
    commands: Vec<serenity::CreateCommand<'static>>,
}

#[async_trait]
impl EventHandler for Handler {
    async fn dispatch(&self, ctx: &SerenityContext, event: &FullEvent) {
        match event {
            FullEvent::Ready { data_about_bot, .. } => {
                info!("{} is ready!", data_about_bot.user.name);
                let guild_id: GuildId = std::env::var("GUILD_ID")
                    .unwrap()
                    .parse::<u64>()
                    .expect("Error while parsing guild id")
                    .into();

                if let Err(e) = guild_id.set_commands(&ctx.http, &self.commands).await {
                    error!("Failed to register commands: {}", e);
                }
                info!("Successfully registered slash commands!");
                scheduler::start_confession_scheduler(ctx).await;
            }
            _ => {}
        }
    }
}

#[tokio::main]
async fn main() {
    dotenv().ok();
    tracing_subscriber::fmt::init();

    let options = poise::FrameworkOptions {
        commands: vec![commands::set_channel(), commands::confess()],
        prefix_options: PrefixFrameworkOptions {
            mention_as_prefix: true,
            ..PrefixFrameworkOptions::default()
        },
        ..Default::default()
    };

    let commands = poise::builtins::create_application_commands(&options.commands);

    let token = serenity::Token::from_env("DISCORD_TOKEN").unwrap();
    let intents =
        serenity::GatewayIntents::non_privileged() | serenity::GatewayIntents::MESSAGE_CONTENT;

    let framework = poise::Framework::builder().options(options).build();

    let redis_client = match redis::Client::open(
        std::env::var("REDIS_URL").unwrap_or_else(|_| "redis://localhost:6379".to_string()),
    ) {
        Ok(client) => client,
        Err(e) => {
            error!("Failed to create Redis client: {}", e);
            std::process::exit(1);
        }
    };
    let redis_conn = match redis::aio::ConnectionManager::new(redis_client).await {
        Ok(conn) => conn,
        Err(e) => {
            error!("Failed to connect to Redis: {}", e);
            std::process::exit(1);
        }
    };

    let data = Arc::new(Data {
        redis_client: redis_conn,
    });

    let client = serenity::ClientBuilder::new(token, intents)
        .data(data.clone())
        .event_handler(Arc::new(Handler { commands }))
        .framework(Box::new(framework))
        .await;

    client.unwrap().start().await.unwrap();
}
