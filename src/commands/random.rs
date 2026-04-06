use crate::agent::{self, Agent, Role};
use crate::{Context, Error};
use poise::CreateReply;
use poise::serenity_prelude as serenity;
use std::time::Duration;

/// VALORANT エージェントをランダムに抽選する
#[poise::command(slash_command, subcommands("single", "multi"), subcommand_required)]
pub async fn random(_: Context<'_>) -> Result<(), Error> {
    Ok(())
}

async fn autocomplete_agent_name(_ctx: Context<'_>, partial: &str) -> Vec<String> {
    let (prefix, current) = match partial.rsplit_once(',') {
        Some((before, after)) => (format!("{},", before), after.trim()),
        None => (String::new(), partial.trim()),
    };

    let already_selected: Vec<String> = prefix
        .split(',')
        .map(|s| s.trim().to_lowercase())
        .filter(|s| !s.is_empty())
        .collect();

    let current_lower = current.to_lowercase();

    agent::agent_names()
        .into_iter()
        .filter(|name| !already_selected.contains(&name.to_lowercase()))
        .filter(|name| name.to_lowercase().contains(&current_lower))
        .map(|name| {
            if prefix.is_empty() {
                name
            } else {
                format!("{} {}", prefix, name)
            }
        })
        .collect()
}

fn build_single_embed(agent: &Agent) -> serenity::CreateEmbed {
    serenity::CreateEmbed::new()
        .title(&agent.name)
        .url(agent.tracker_url())
        .thumbnail(agent.icon_url())
        .field("ロール", role_display_name(agent.role), true)
        .color(role_color(agent.role))
}

fn build_multi_embed(agents: &[&Agent]) -> serenity::CreateEmbed {
    let description = agents
        .iter()
        .enumerate()
        .map(|(i, a)| {
            format!(
                "{}. [{}]({}) ({})",
                i + 1,
                a.name,
                a.tracker_url(),
                role_display_name(a.role)
            )
        })
        .collect::<Vec<_>>()
        .join("\n");

    serenity::CreateEmbed::new()
        .title("ランダムエージェント抽選")
        .description(description)
        .color(0xfd4556)
}

fn reroll_button(custom_id: &str) -> Vec<serenity::CreateActionRow> {
    vec![serenity::CreateActionRow::Buttons(vec![
        serenity::CreateButton::new(custom_id)
            .style(serenity::ButtonStyle::Secondary)
            .label("再抽選"),
    ])]
}

fn disabled_reroll_button(custom_id: &str) -> Vec<serenity::CreateActionRow> {
    vec![serenity::CreateActionRow::Buttons(vec![
        serenity::CreateButton::new(custom_id)
            .style(serenity::ButtonStyle::Secondary)
            .label("再抽選")
            .disabled(true),
    ])]
}

fn role_display_name(role: Role) -> &'static str {
    match role {
        Role::Controller => "コントローラー",
        Role::Duelist => "デュエリスト",
        Role::Initiator => "イニシエーター",
        Role::Sentinel => "センチネル",
    }
}

fn role_color(role: Role) -> serenity::Color {
    match role {
        Role::Controller => serenity::Color::from_rgb(215, 130, 55),
        Role::Duelist => serenity::Color::from_rgb(232, 80, 90),
        Role::Initiator => serenity::Color::from_rgb(52, 165, 130),
        Role::Sentinel => serenity::Color::from_rgb(68, 150, 200),
    }
}

/// エージェントを1人ランダムで抽選する
#[poise::command(slash_command)]
async fn single(
    ctx: Context<'_>,
    #[description = "抽選から除外するエージェント (カンマ区切りで複数指定可)"]
    #[autocomplete = "autocomplete_agent_name"]
    #[rename = "ignore-characters"]
    ignore_characters: Option<String>,
    #[description = "抽選するロール"] role: Option<Role>,
) -> Result<(), Error> {
    let ignore_list: Vec<String> = ignore_characters
        .as_deref()
        .unwrap_or("")
        .split(',')
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty())
        .collect();
    let filtered = agent::filter_agents(role, &ignore_list);

    if filtered.is_empty() {
        ctx.say("条件に一致するエージェントがいません。").await?;
        return Ok(());
    }

    let selected = agent::pick_random(&filtered, 1);
    let agent = selected[0];

    let reroll_id = format!("{}_reroll", ctx.id());
    let embed = build_single_embed(agent);
    let components = reroll_button(&reroll_id);

    let reply = ctx
        .send(CreateReply::default().embed(embed).components(components))
        .await?;

    while let Some(press) = {
        let id = reroll_id.clone();
        serenity::collector::ComponentInteractionCollector::new(ctx)
            .filter(move |press| press.data.custom_id == id)
            .timeout(Duration::from_secs(120))
            .await
    } {
        let new_selected = agent::pick_random(&filtered, 1);
        let new_agent = new_selected[0];
        let new_embed = build_single_embed(new_agent);

        press
            .create_response(
                ctx.serenity_context(),
                serenity::CreateInteractionResponse::UpdateMessage(
                    serenity::CreateInteractionResponseMessage::new().embed(new_embed),
                ),
            )
            .await?;
    }

    let reroll_id = format!("{}_reroll", ctx.id());
    let _ = reply
        .edit(
            ctx,
            CreateReply::default().components(disabled_reroll_button(&reroll_id)),
        )
        .await;

    Ok(())
}

/// 複数エージェントをランダムで抽選する
#[poise::command(slash_command)]
async fn multi(
    ctx: Context<'_>,
    #[description = "チームメンバーの人数 (1-5)"]
    #[rename = "team-member-size"]
    #[min = 1]
    #[max = 5]
    team_member_size: u8,
) -> Result<(), Error> {
    let all = agent::filter_agents(None, &[]);
    let count = team_member_size as usize;

    if count > all.len() {
        ctx.say(format!(
            "エージェント数 ({}) を超える人数は指定できません。",
            all.len()
        ))
        .await?;
        return Ok(());
    }

    let selected = agent::pick_random(&all, count);

    let reroll_id = format!("{}_reroll", ctx.id());
    let embed = build_multi_embed(&selected);
    let components = reroll_button(&reroll_id);

    let reply = ctx
        .send(CreateReply::default().embed(embed).components(components))
        .await?;

    while let Some(press) = {
        let id = reroll_id.clone();
        serenity::collector::ComponentInteractionCollector::new(ctx)
            .filter(move |press| press.data.custom_id == id)
            .timeout(Duration::from_secs(120))
            .await
    } {
        let new_selected = agent::pick_random(&all, count);
        let new_embed = build_multi_embed(&new_selected);

        press
            .create_response(
                ctx.serenity_context(),
                serenity::CreateInteractionResponse::UpdateMessage(
                    serenity::CreateInteractionResponseMessage::new().embed(new_embed),
                ),
            )
            .await?;
    }

    let reroll_id = format!("{}_reroll", ctx.id());
    let _ = reply
        .edit(
            ctx,
            CreateReply::default().components(disabled_reroll_button(&reroll_id)),
        )
        .await;

    Ok(())
}
