use crate::database::Database;

use super::MessageContext;


pub async fn handle(context: MessageContext, _args: Vec<&str>) -> anyhow::Result<()> {

    let mut db = Database::connect().await;
    let user = db.get_user(&context).await;

    context.send(format!("@%username%, you have {} points!", user.points).as_str()).await;
    Ok(())
}

