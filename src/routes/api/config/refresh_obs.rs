use axum::response::IntoResponse;
use tracing::debug;

use crate::{obs::ObsClient, routes::index::{get_config_info, ObsTemplate}, HtmlTemplate};


pub async fn post() -> impl IntoResponse {
    debug!("Refreshing obs");
    let _ = ObsClient::load().await;
    debug!("Refresh done");
    
    let (config, obs_info) = get_config_info().await;

    HtmlTemplate(ObsTemplate {
        config,
        obs: obs_info
    })
}