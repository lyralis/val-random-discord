use crate::agent::{self, Role};
use crate::{Context, Error};
use poise::CreateReply;
use poise::serenity_prelude as serenity;

/// 抽選対象のエージェント一覧を表示する
#[poise::command(slash_command)]
pub async fn agents(ctx: Context<'_>) -> Result<(), Error> {
    let all = agent::all_agents();
    let patch = agent::patch();

    let mut controllers = Vec::new();
    let mut duelists = Vec::new();
    let mut initiators = Vec::new();
    let mut sentinels = Vec::new();

    for a in all {
        match a.role {
            Role::Controller => controllers.push(a.name.as_str()),
            Role::Duelist => duelists.push(a.name.as_str()),
            Role::Initiator => initiators.push(a.name.as_str()),
            Role::Sentinel => sentinels.push(a.name.as_str()),
        }
    }

    let embed = serenity::CreateEmbed::new()
        .title(format!("エージェント一覧 (パッチ {})", patch))
        .description(format!("抽選対象: **{}** エージェント", all.len()))
        .field(
            format!("コントローラー ({})", controllers.len()),
            controllers.join(", "),
            false,
        )
        .field(
            format!("デュエリスト ({})", duelists.len()),
            duelists.join(", "),
            false,
        )
        .field(
            format!("イニシエーター ({})", initiators.len()),
            initiators.join(", "),
            false,
        )
        .field(
            format!("センチネル ({})", sentinels.len()),
            sentinels.join(", "),
            false,
        )
        .color(0xfd4556);

    ctx.send(CreateReply::default().embed(embed)).await?;

    Ok(())
}
