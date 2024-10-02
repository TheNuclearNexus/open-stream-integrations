use axum::{
    // Import axum crate for building web applications
    routing::{get, post},
    Router,
};
// Import Deserialize trait from serde for deserializing form data
use std::sync::Arc; // Import Arc for thread-safe reference counting
use std::sync::Mutex; // Import Mutex for thread-safe mutual exclusion
use tower_http::services::ServeDir; // Import ServeDir for serving static files
use tracing::info;

use crate::config::SiteConfig; // Import info! macro from tracing for logging

pub mod another_page;
pub mod api;
pub mod index;
pub mod overlay;

// Define a struct to hold the application state
pub struct AppState {
    games: Mutex<Vec<&'static str>>,
}

pub async fn setup_router() {
    info!("router init...");

    let app_state = Arc::new(AppState {
        games: Mutex::new(vec![
            "Ultrakill",
            "Ghostrunner 2",
            "everhood",
            "hollow knight",
            "neon white",
            "half life",
            "Minecraft",
            "Risk of Rain 2",
            "Audio Trip",
            "Call of Duty",
            "Risk of Rain Returns",
            "Necrodancer",
            "Rythm Heaven Fever",
            "Lugi mans",
            "SUNSHINE",
            "Pokemon randomizer",
            "new moriah bros ds",
            "bloodborne",
            "son of war of dad of son"
        ]),
    });

    let assets_path = std::env::current_dir().unwrap();
    let router = Router::new()
        .route("/", get(index::render))
        .route("/overlay", get(overlay::render))
        .route("/sse", get(api::sse::handler))
        .route("/api/wheel", get(api::wheel::get))
        .route("/api/wheel", post(api::wheel::post))
        .route("/api/claim_code", get(api::claim_code::get))
        .route("/api/claim_code", post(api::claim_code::post))
        .route("/api/config", post(api::config::post))
        .route("/api/config/elements", get(api::config::elements::get))
        .route("/api/config/refresh_obs", post(api::config::refresh_obs::post))
        .nest_service(
            "/assets",
            ServeDir::new(format!("{}/assets", assets_path.to_str().unwrap())),
        )
        .with_state(app_state); // Pass the shared state to the router

    let port = 8086_u16;
    let listener = tokio::net::TcpListener::bind(format!("0.0.0.0:{}", port))
        .await
        .unwrap();

    info!("router init complete: now listening on port {}", port);

    let handle = tokio::spawn(async move {
        axum::serve(listener, router).await.unwrap();
    });

    handle.await.unwrap();
}
