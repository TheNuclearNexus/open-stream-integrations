use tracing::info;

use crate::{config::SiteConfig, database::Database, events::CLAIM_CODE, get_system_time};

use super::MessageContext;




pub async fn handle(message: MessageContext, args: Vec<&str>) -> anyhow::Result<()> {
    if args.len() != 1 {
        return Ok(());
    }

    let code = args[0];
    info!(code);
    if code.len() != 6 {
        return Ok(());
    }
    
    let current_claim_code = CLAIM_CODE.lock().await;

    if current_claim_code.code.to_string() != code {
        return Ok(());
    }

    if get_system_time() - current_claim_code.time_created > 3 * 60 {
        return Ok(());
    }

    let mut db = Database::connect().await;
    let mut user = db.get_user(&message).await;

    info!("{:?}", user);

    user.points += SiteConfig::get().await?.claims.points_per;
    
    let _ = db.set_user(&user).await;

    message.send(format!("@%username% claimed the code, new balance is {}", &user.points).as_str()).await;

    Ok(())
}