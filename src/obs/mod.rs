use std::{env::var, thread, time::Duration};

use anyhow::bail;
use futures_util::{pin_mut, StreamExt};
use obws::{
    requests::{scene_items::Id, scenes::SceneId},
    responses::scene_items::SceneItemTransform,
    Client,
};
use once_cell::sync::Lazy;
use tokio::{
    sync::{RwLock, RwLockReadGuard},
    task::{AbortHandle, JoinHandle},
};
use tracing::debug;
use tracing_subscriber::field::debug;

use crate::{config::SiteConfig, events};

static CLIENT: Lazy<RwLock<Option<Box<Client>>>> = Lazy::new(|| RwLock::new(None));
static ABORT: Lazy<RwLock<Option<AbortHandle>>> = Lazy::new(|| RwLock::new(None));

pub struct ObsClient {}

impl ObsClient {
    pub async fn load() -> anyhow::Result<()> {
        debug!("Load obs");
        let mut rw = CLIENT.write().await;

        match rw.take() {
            Some(mut client) => {
                debug!("Disconnecting the old client");
                client.disconnect().await;
                debug!("Old client disconnected");
            }
            None => {
                debug!("No old client");
            }
        };

        let config = SiteConfig::get().await?;
        let address = config.obs.address;
        let address = address.split(":").collect::<Vec<&str>>();
        let host = address[0];
        let port = address[1].parse::<u16>()?;
        let password = var("OBS_PASSWORD").ok();

        let client: Client = Client::connect(host, port, password).await?;

        *rw = Some(Box::new(client));

        match ABORT.read().await.as_ref() {
            Some(abort) => {
                debug!("aborting current obs event stream");
                abort.abort();
            }
            None => {
                debug!("no old event stream")
            }
        }

        Ok(())
    }

    pub async fn get<'a>() -> anyhow::Result<RwLockReadGuard<'a, Option<Box<Client>>>> {
        let rw = CLIENT.read().await;

        if rw.is_none() {
            anyhow::bail!("Client not connected");
        }

        Ok(rw)
    }
}

pub async fn get_webcam_transform() -> anyhow::Result<SceneItemTransform> {
    let obs = ObsClient::get().await?;
    let Some(obs) = obs.as_ref() else {
        bail!("Client is not connected")
    };
    let config = SiteConfig::get().await?;

    let scene = SceneId::Name(config.obs.main_scene.as_str());
    let element = config.obs.webcam_element;

    let item_id = obs
        .scene_items()
        .id(Id {
            scene,
            source: &element,
            search_offset: None,
        })
        .await?;

    let transform = obs.scene_items().transform(scene, item_id).await?;
    
    Ok(transform)
}

fn create_join_handle() -> JoinHandle<()> {
    let join_handle = tokio::spawn(async {
        // let rw = ObsClient::get().await;
        // let Ok(rw) = rw else {
        //     debug!("client not connected");
        //     thread::sleep(Duration::from_secs(1));
        //     return;
        // };
        // let client = rw.as_ref().unwrap();
        // let events = client.events().unwrap();
        // pin_mut!(events);

        // drop(rw);

        // while let Some(event) = events.next().await {
        //     debug!("{:?}", event);
        //     match event {
        //         obws::events::Event::SceneItemTransformChanged { scene, item_id, transform } => {
        //             debug!("{scene:?}");
        //         },
        //         _ => {}
        //     }
        // }

        let mut last_transform = SceneItemTransform::default();

        loop {

            let Ok(transform) = get_webcam_transform().await else {
                thread::sleep(Duration::from_secs(1));
                return 
            };

            if transform != last_transform {
                debug!("{:?}", transform);
                let _ = events::update_webcam(transform.clone()).await;
                last_transform = transform;
            }
            thread::sleep(Duration::from_millis(1000 / 20))
        }
    });

    join_handle
}

pub async fn setup_obs() -> anyhow::Result<()> {
    loop {
        let join_handle = create_join_handle();
        debug!("Acquired new join handle!");

        let mut abort = ABORT.write().await;
        *abort = Some(join_handle.abort_handle());
        drop(abort);

        let _ = tokio::join!(join_handle);
        debug!("Dropped previous join handle");
    }

    Ok(())
}
