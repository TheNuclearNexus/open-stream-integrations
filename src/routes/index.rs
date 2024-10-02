use std::sync::Arc;

use crate::{config::SiteConfig, obs::ObsClient, routes::AppState, HtmlTemplate};
use askama::Template;
use axum::{
    extract::State,
    response::{Html, IntoResponse},
};
use obws::{requests::scenes::SceneId, responses::{scene_items::{SceneItem, SceneItemTransform}, scenes::Scene}, Client};
use once_cell::sync::Lazy;

pub async fn get_scene_elements(
    scene: String,
    obs: &Client,
) -> anyhow::Result<Vec<SceneItem>> {
    let elements = obs.scene_items().list(SceneId::Name(&scene)).await?;

    Ok(elements)
}

async fn get_obs_info(config: &SiteConfig) -> anyhow::Result<ObsInfo> {
    let obs = ObsClient::get().await?;
    let obs = obs.as_ref().unwrap();
    let scenes = obs.scenes().list().await?.scenes;

    let elements = get_scene_elements(config.obs.main_scene.clone(), obs).await.ok();

    Ok(ObsInfo {
        scenes: scenes,
        connected: true,
        elements: elements,
    })
}

pub async fn get_config_info() -> (SiteConfig, ObsInfo) {
    let config = SiteConfig::get().await.unwrap();
    let obs_info = get_obs_info(&config).await.unwrap_or(NULL_INFO.clone());

    (config, obs_info)
} 

pub async fn render(State(_): State<Arc<AppState>>) -> impl IntoResponse {
    let (config, obs_info) = get_config_info().await;

    let template = IndexTemplate {
        config,
        obs: obs_info
    };
    HtmlTemplate(template)
}

#[derive(Template)]
#[template(path = "index.html")]
struct IndexTemplate {
    config: SiteConfig,
    obs: ObsInfo
}

#[derive(Template)]
#[template(path = "index/obs.html")]
pub(crate) struct ObsTemplate {
    pub config: SiteConfig,
    pub obs: ObsInfo
}

#[derive(Template)]
#[template(
    source = r#"{% import "index/elements.html" as elements %}{% call elements::elements(elements, selected_element.as_ref()) %}"#,
    ext = "html"
)]
pub(crate) struct ElementsTemplate {
    pub elements: Option<Vec<SceneItem>>,
    pub selected_element: Option<String>
}



static NULL_INFO: Lazy<ObsInfo> = Lazy::new(|| ObsInfo::default());

#[derive(Clone, Default)]
pub struct ObsInfo {
    pub scenes: Vec<Scene>,
    pub connected: bool,
    pub elements: Option<Vec<SceneItem>>,
}