use std::collections::HashMap;

use askama::Template;
use axum::{extract::Query, http::StatusCode, response::IntoResponse};
use obws::{
    requests::scenes::SceneId,
    responses::{scene_items::SceneItem, scenes::Scene},
    Client,
};
use tracing::info;

use crate::{config::SiteConfig, obs::ObsClient, routes::index::{get_scene_elements, ElementsTemplate}, HtmlTemplate};



pub async fn get(Query(params): Query<HashMap<String, String>>) -> impl IntoResponse {
    let obs = ObsClient::get().await.unwrap();
    let obs = obs.as_ref().unwrap();

    let elements = get_scene_elements(params.get("scene").unwrap().clone(), obs).await.ok();

    HtmlTemplate(ElementsTemplate {
        elements,
        selected_element: Some(SiteConfig::get().await.unwrap().obs.webcam_element)
    })
}
