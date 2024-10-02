use tracing::info;

use crate::{config::SiteConfig, database::Database, events::spin_wheel};

use super::MessageContext;

pub async fn handle(context: MessageContext, _args: Vec<&str>) -> anyhow::Result<()> {
    let mut db = Database::connect().await;
    let mut user = db.get_user(&context).await;

    info!("{:?}", user);

    let config = SiteConfig::get().await?;

    if user.points < config.wheel.cost && context.user_id() != "408189224".to_owned() {
        context
            .send(format!("@%username% you do not have {} points!", config.wheel.cost).as_str())
            .await;
        return Ok(());
    }

    if user.points >= config.wheel.cost {
        user.points -= config.wheel.cost;
    }

    let _ = db.set_user(&user).await;

    context.send("@%username% spun the wheel!").await;

    spin_wheel().await;
    Ok(())
}
