use askama::Template;
use axum::response::IntoResponse;
use crate::HtmlTemplate;

pub async fn render() -> impl IntoResponse {
    let template = AnotherPageTemplate {};
    HtmlTemplate(template)
}

#[derive(Template)]
#[template(path = "another-page.html")]
struct AnotherPageTemplate;
