pub mod elements;
pub mod refresh_obs;

use axum::{body::{Body, Bytes}, extract::{FromRequest, Request}, http::StatusCode, response::{Html, IntoResponse, Response}, Json};
use serde::{Deserialize, Serialize};
use tracing::{error, info, debug};

use crate::{config::SiteConfig, obs::ObsClient};


async fn handle_post(body: String) -> anyhow::Result<()> {
    // let _ = SiteConfig::save(&payload).await;
    let config: SiteConfig = serde_json::from_str(body.as_str())?;
    let prev_config = SiteConfig::get().await?;
    
    SiteConfig::save(&config).await?;

    debug!("Saved config, reloading obs client");
    ObsClient::load().await?;
    debug!("Reloaded obs client");
    
    Ok(())
}

pub async fn post(body: String) -> impl IntoResponse {
    let result = handle_post(body).await;

    if result.is_err() {
        let error = result.err().unwrap();
        error!("{:?}", error);
        let response = (StatusCode::INTERNAL_SERVER_ERROR, "Failed to save config.").into_response();
        return response;
    }

    Html(r#"Saved."#).into_response()
}