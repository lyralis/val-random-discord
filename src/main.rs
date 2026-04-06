use anyhow::Context as _;
use poise::serenity_prelude as serenity;

mod agent;
mod commands;

struct Data {}

type Error = Box<dyn std::error::Error + Send + Sync>;
type Context<'a> = poise::Context<'a, Data, Error>;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt::init();
    dotenvy::dotenv().ok();

    let token =
        std::env::var("DISCORD_TOKEN").context("DISCORD_TOKEN environment variable not set")?;
    let intents = serenity::GatewayIntents::non_privileged();

    let framework = poise::Framework::<Data, Error>::builder()
        .options(poise::FrameworkOptions {
            commands: vec![commands::random::random()],
            ..Default::default()
        })
        .setup(|ctx, ready, framework| {
            Box::pin(async move {
                tracing::info!("Logged in as {}", ready.user.name);

                let commands = &framework.options().commands;
                match std::env::var("DISCORD_GUILD_ID") {
                    Ok(guild_id) => {
                        let guild_id = serenity::GuildId::new(guild_id.parse()?);
                        poise::builtins::register_in_guild(ctx, commands, guild_id).await?;
                        tracing::info!("Registered commands in guild {}", guild_id);
                    }
                    Err(_) => {
                        poise::builtins::register_globally(ctx, commands).await?;
                        tracing::info!("Registered commands globally");
                    }
                }

                Ok(Data {})
            })
        })
        .build();

    let mut client = serenity::ClientBuilder::new(token, intents)
        .framework(framework)
        .await?;

    client.start().await?;

    Ok(())
}
