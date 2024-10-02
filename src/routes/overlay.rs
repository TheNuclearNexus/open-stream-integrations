
use askama::Template;
use axum::response::IntoResponse;
use obws::responses::scene_items::SceneItemTransform;
use crate::{config::SiteConfig, obs::get_webcam_transform, HtmlTemplate};

pub async fn render() -> impl IntoResponse {
    let template = OverlayTemplate {
        config: SiteConfig::get().await.unwrap(),
        webcam_transform: get_webcam_transform().await.ok()
    };
    HtmlTemplate(template)
}

#[derive(Template)]
#[template(path = "overlay.html")]
struct OverlayTemplate {
    webcam_transform: Option<SceneItemTransform>,
    config: SiteConfig
}
