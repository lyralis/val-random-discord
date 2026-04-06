use crate::{Context, Error};
use poise::CreateReply;
use poise::serenity_prelude as serenity;

/// コマンドヘルプを表示する
#[poise::command(slash_command)]
pub async fn help(ctx: Context<'_>) -> Result<(), Error> {
    let commands = &ctx.framework().options().commands;
    let command_ids = &ctx.data().command_ids;

    let mut lines = Vec::new();
    for cmd in commands {
        if cmd.subcommands.is_empty() {
            let mention = format_command_mention(&cmd.name, None, command_ids);
            let description = cmd.description.as_deref().unwrap_or("説明なし");
            lines.push(format!("{} — {}", mention, description));
        } else {
            for sub in &cmd.subcommands {
                let mention = format_command_mention(&cmd.name, Some(&sub.name), command_ids);
                let description = sub.description.as_deref().unwrap_or("説明なし");
                lines.push(format!("{} — {}", mention, description));
            }
        }
    }

    let embed = serenity::CreateEmbed::new()
        .title("コマンドヘルプ")
        .description(lines.join("\n"))
        .color(0xfd4556);

    ctx.send(CreateReply::default().embed(embed)).await?;

    Ok(())
}

fn format_command_mention(
    name: &str,
    subcommand: Option<&str>,
    command_ids: &std::collections::HashMap<String, serenity::CommandId>,
) -> String {
    match command_ids.get(name) {
        Some(id) => match subcommand {
            Some(sub) => format!("</{} {}:{}>", name, sub, id),
            None => format!("</{}:{}>", name, id),
        },
        None => match subcommand {
            Some(sub) => format!("/{} {}", name, sub),
            None => format!("/{}", name),
        },
    }
}
