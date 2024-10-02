use askama::Template; // Import Template trait from askama for HTML templating
use axum::{
    // Import axum crate for building web applications
    http::StatusCode,
    response::{Html, IntoResponse, Response}
};
use tracing::{debug, warn};
 // Import Deserialize trait from serde for deserializing form data
use std::time::SystemTime; // Import Arc for thread-safe reference counting
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt}; // Import traits from tracing_subscriber for initializing tracing
use dotenvy;

use crate::{config::SiteConfig, obs::ObsClient};

pub mod routes;
pub mod twitch;
pub mod commands;
pub mod database;
pub mod events;
pub mod config;
pub mod obs;
pub mod polls;



#[tokio::main]
async fn main() -> anyhow::Result<()> {
    dotenvy::dotenv().ok();

    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "wheel_website=debug".into()),
        )
        .with(tracing_subscriber::fmt::layer())
        .init();

    SiteConfig::load().await?;
    debug!("Site config loaded.");
    let result = ObsClient::load().await;
    if result.is_ok() {
        debug!("OBS Websocket loaded.");
    } else {
        warn!("OBS failed to connect, {}", result.err().unwrap())
    }

    tokio::select! {
        _ = twitch::setup_twitch() => {
            println!("twitch finished early!")
        }
        _ = routes::setup_router() => {
            println!("router finished early!")
        }
        _ = events::setup_events() => {
            println!("events finished early!")
        }
        _ = obs::setup_obs() => {
            println!("obs finished early")
        }
    };


    Ok(())
}

pub fn get_system_time() -> u64 {
    SystemTime::now()
        .duration_since(SystemTime::UNIX_EPOCH)
        .unwrap()
        .as_secs()
}


/// A wrapper type that we'll use to encapsulate HTML parsed by askama into valid HTML for axum to serve.
pub struct HtmlTemplate<T>(T);

/// Allows us to convert Askama HTML templates into valid HTML for axum to serve in the response.
impl<T> IntoResponse for HtmlTemplate<T>
where
    T: Template,
{
    fn into_response(self) -> Response {
        // Attempt to render the template with askama
        match self.0.render() {
            // If we're able to successfully parse and aggregate the template, serve it
            Ok(html) => Html(html).into_response(),
            // If we're not, return an error or some bit of fallback HTML
            Err(err) => (
                StatusCode::INTERNAL_SERVER_ERROR,
                format!("Failed to render template. Error: {}", err),
            )
                .into_response(),
        }
    }
}
