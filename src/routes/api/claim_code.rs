
use askama::Template;
use axum::response::IntoResponse;

use crate::{events::{generate_claim_code, CLAIM_CODE}, HtmlTemplate};

pub async fn get() -> impl IntoResponse {

    let claim_code = CLAIM_CODE.lock().await.code;

    HtmlTemplate(ClaimCodeTemplate { 
        claim_code
    })
}

pub async fn post() {
    generate_claim_code().await;
}


#[derive(Template)]
#[template(path = "claim_code.html")]
struct ClaimCodeTemplate {
    claim_code: i32
}
