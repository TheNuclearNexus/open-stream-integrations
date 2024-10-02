use askama::Template;
use tracing::info;

use crate::{database::Database, routes::api::sse::{SseEvent, BROADCAST}, HtmlTemplate};

use super::MessageContext;


pub async fn handle(context: MessageContext, args: Vec<&str>) {

    let mut db = Database::connect().await;
    let mut user = db.get_user(&context).await;

    info!("{:?}", user);

    context.send(format!("@%username%, you have {} points!", user.points).as_str()).await;
}

