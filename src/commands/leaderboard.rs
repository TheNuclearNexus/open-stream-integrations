use tracing::debug;

use crate::database::Database;

use super::MessageContext;

static EMOJIS: [&str; 3] = ["🥇", "🥈", "🥉"];

pub async fn handle(context: MessageContext, _args: Vec<&str>) -> anyhow::Result<()> {
    let db = Database::connect().await;

    let results = db
        .client
        .query(
            "
        SELECT username, platform, points FROM app_user
        ORDER BY points DESC
        LIMIT 3;
    ",
            &[],
        )
        .await?;

    debug!("{:?}", results);

    let data= results
        .iter()
        .map(|r| (r.get(0), r.get(1), r.get(2)));

    debug!("{:?}", data.clone().collect::<Vec<(&str, &str, i32)>>());

    let data: Vec<String> = data
        .zip(0..results.len())
        .map(|(r, i): ((&str, &str, i32), usize)| 
            format!("{} {}: {}", EMOJIS[i], r.0, r.2))
        .collect();

    debug!("{:?}", data.join(""));

    context.send(format!("Leaderboard: {}", data.join(" | ")).as_str()).await;

    Ok(())
}
