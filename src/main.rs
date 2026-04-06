use std::collections::HashMap;

use anyhow::Context as _;
use poise::serenity_prelude as serenity;

mod agent;
mod commands;

pub struct Data {
    /// コマンド名 → コマンドID のマッピング (スラッシュコマンドメンション用)
    pub command_ids: HashMap<String, serenity::CommandId>,
}

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
            commands: vec![
                commands::help::help(),
                commands::agents::agents(),
                commands::random::random(),
            ],
            ..Default::default()
        })
        .setup(|ctx, ready, framework| {
            Box::pin(async move {
                tracing::info!("Logged in as {}", ready.user.name);

                let commands_builder = create_commands(framework.options());
                let registered = match std::env::var("DISCORD_GUILD_ID") {
                    Ok(guild_id) => {
                        let guild_id = serenity::GuildId::new(guild_id.parse()?);
                        let cmds = guild_id.set_commands(ctx, commands_builder).await?;
                        tracing::info!("Registered commands in guild {}", guild_id);
                        cmds
                    }
                    Err(_) => {
                        let cmds =
                            serenity::Command::set_global_commands(ctx, commands_builder).await?;
                        tracing::info!("Registered commands globally");
                        cmds
                    }
                };

                let command_ids: HashMap<String, serenity::CommandId> = registered
                    .into_iter()
                    .map(|cmd| (cmd.name, cmd.id))
                    .collect();

                Ok(Data { command_ids })
            })
        })
        .build();

    let mut client = serenity::ClientBuilder::new(token, intents)
        .framework(framework)
        .await?;

    client.start().await?;

    Ok(())
}

fn create_commands(options: &poise::FrameworkOptions<Data, Error>) -> Vec<serenity::CreateCommand> {
    let integration_types = vec![
        serenity::InstallationContext::Guild,
        serenity::InstallationContext::User,
    ];
    let contexts = vec![
        serenity::InteractionContext::Guild,
        serenity::InteractionContext::BotDm,
        serenity::InteractionContext::PrivateChannel,
    ];

    options
        .commands
        .iter()
        .filter_map(|cmd| {
            cmd.create_as_slash_command().map(|builder| {
                builder
                    .integration_types(integration_types.clone())
                    .contexts(contexts.clone())
            })
        })
        .collect()
}
