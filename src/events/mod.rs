use std::{thread, time::Duration};

use askama::Template;
use obws::responses::scene_items::SceneItemTransform;
use once_cell::sync::Lazy;
use rand::Rng;
use tokio::sync::Mutex;
use tracing::{debug, info};

use crate::{
    config::SiteConfig,
    get_system_time,
    routes::api::sse::{SseEvent, BROADCAST},
    HtmlTemplate,
};

pub static CLAIM_CODE: Lazy<Mutex<ClaimCode>> = Lazy::new(|| {
    Mutex::new(ClaimCode {
        code: 0,
        time_created: 0,
    })
});



pub async fn generate_claim_code() {
    let tx = &BROADCAST.lock().await.0;
    let _ = tx.send(SseEvent {
        data: "".to_owned(),
        event: "claim_code".to_owned(),
    });

    let code = rand::thread_rng().gen_range(111111..999999);
    let time_created = get_system_time();

    let mut claim_code = CLAIM_CODE.lock().await;

    *claim_code = ClaimCode { code, time_created };

    info!("Claim code generated")
}

pub async fn spin_wheel() {
    let tx = &BROADCAST.lock().await.0;

    let _ = tx.send(SseEvent {
        event: "wheel".to_owned(),
        data: "".to_owned(),
    });
}

pub async fn heartbeat() {
    let tx = &BROADCAST.lock().await.0;

    let _ = tx.send(SseEvent {
        event: "heartbeat".to_owned(),
        data: "".to_owned()
    });
}

pub async fn update_webcam(transform: SceneItemTransform) -> anyhow::Result<()> {
    let tx = &BROADCAST.lock().await.0;

    let html = WebcamTemplate {
        config: SiteConfig::get().await?,
        transform,
    }
    .to_string().replace("\n", "").replace("\r", "");

    debug!("{html:?}");

    let res = tx.send(SseEvent {
        event: "webcam".to_owned(),
        data: html
    });

    debug!("{res:?}");

    Ok(())
}

#[derive(Template)]
#[template(path = "overlay/webcam.html")]
struct WebcamTemplate {
    config: SiteConfig,
    transform: SceneItemTransform,
}

pub async fn setup_events() {
    info!("Event generator started!");

    tokio::spawn(async move {
        let mut minutes = 14;
        loop {
            let config = SiteConfig::get().await;
            
            let interval = if config.is_err() {
                15
            } else {
                config.unwrap().claims.interval
            };

            if minutes % interval == 0 {
                generate_claim_code().await;
            }
            minutes = (minutes + 1) % 15;
            thread::sleep(Duration::from_secs(60));
        }
    })
    .await
    .unwrap();
}

pub struct ClaimCode {
    pub code: i32,
    pub time_created: u64,
}
