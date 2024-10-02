use std::sync::Arc;

use askama::Template;
use axum::{extract::State, response::IntoResponse};

use crate::{events::spin_wheel, routes::AppState, HtmlTemplate};
use rand::Rng;

pub async fn get(State(state): State<Arc<AppState>>, // Get the shared state
) -> impl IntoResponse {
    let mut items = "/*".to_owned();
    let mut games = state.games.lock().unwrap();

    for g in games.iter() {
        items = format!(
            "{{
            label: '{}'
        }},{}",
            g, items
        );
    }

    items = format!("*/{}", items);
    let winning_item = rand::thread_rng().gen_range(0..games.len());
    games.remove(winning_item);

    HtmlTemplate(WheelTemplate { 
        items,
        winning_item
    })
}

pub async fn post() {
    spin_wheel().await;
}

#[derive(Template)]
#[template(path = "wheel.html")]
struct WheelTemplate {
    items: String,
    winning_item: usize
}
