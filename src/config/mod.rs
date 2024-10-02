use std::{env::var, fmt::Display, fs::File, io::BufReader, str::FromStr };

use anyhow::anyhow;
use once_cell::sync::Lazy;
use postgres::config;
use serde::{Deserialize, Serialize, Deserializer};
use tokio::{fs::OpenOptions, io::AsyncWriteExt, sync::RwLock};

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct SiteConfig {
    pub claims: ClaimCodeConfig,
    pub wheel: WheelConfig,
    pub obs: ObsConfig,
    pub twitch: TwitchConfig,
}

static CONFIG: Lazy<RwLock<Option<SiteConfig>>> = Lazy::new(|| RwLock::new(None));

impl SiteConfig {
    pub async fn create() -> anyhow::Result<SiteConfig> {
        let config = SiteConfig::default();
        Self::save(&config).await?;
        Ok(config)
    }

    pub async fn save(config: &SiteConfig) -> anyhow::Result<()> {
        let mut rw = CONFIG.write().await;
        let mut file = OpenOptions::new()
            .write(true)
            .create(true)
            .open(var("SITE_CONFIG_PATH")?).await?;

        let json = serde_json::to_string(config)?;
        file.write(json.as_bytes()).await?;

        *rw = Some(config.clone());

        Ok(())
    }

    pub async fn get() -> anyhow::Result<SiteConfig> {
        // Get the already loaded value to avoid blocking for long
        let config = CONFIG.read().await;
        if config.is_some() {
            return Ok(config.clone().unwrap());
        }
        
        Err(anyhow!("Config not loaded"))
    }

    pub async fn load() -> anyhow::Result<SiteConfig> {
        let file = File::open(var("SITE_CONFIG_PATH")?);

        if file.is_err() {
            return Self::create().await
        }

        let file = file.unwrap();

        let reader = BufReader::new(file);
        let config: SiteConfig = serde_json::from_reader(reader)?;

        let mut rw = CONFIG.write().await;

        *rw = Some(config.clone());

        Ok(config)
    }
}

#[derive(Clone, Copy, Debug, Serialize, Deserialize)]
pub struct ClaimCodeConfig {
    #[serde(deserialize_with = "deserialize_number_from_string")]
    pub interval: i32,
    #[serde(deserialize_with = "deserialize_number_from_string")]
    pub points_per: i32,
}
impl Default for ClaimCodeConfig {
    fn default() -> Self {
        ClaimCodeConfig {
            interval: 15,
            points_per: 200,
        }
    }
}

#[derive(Clone, Copy, Debug, Serialize, Deserialize)]
pub struct WheelConfig {
    #[serde(deserialize_with = "deserialize_number_from_string")]
    pub cost: i32,
}
impl Default for WheelConfig {
    fn default() -> Self {
        WheelConfig {
            cost: 600,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ObsConfig {
    pub address: String,
    pub main_scene: String,
    pub webcam_element: String,
}

impl Default for ObsConfig {
    fn default() -> Self {
        ObsConfig {
            address: "localhost:4455".to_owned(),
            main_scene: "".to_owned(),
            webcam_element: "".to_owned()
        }
    }
} 

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TwitchConfig {
    pub channel: String,
}

impl Default for TwitchConfig {
    fn default() -> Self {
        TwitchConfig {
            channel: "the_nuclear_nexus".to_owned(),
        }
    }
}

// Stolen from https://github.com/iddm/serde-aux/blob/master/src/field_attributes.rs#L305
pub fn deserialize_number_from_string<'de, T, D>(deserializer: D) -> Result<T, D::Error>
where
    D: Deserializer<'de>,
    T: FromStr + serde::Deserialize<'de>,
    <T as FromStr>::Err: Display,
{
    #[derive(Deserialize)]
    #[serde(untagged)]
    enum StringOrInt<T> {
        String(String),
        Number(T),
    }

    match StringOrInt::<T>::deserialize(deserializer)? {
        StringOrInt::String(s) => s.parse::<T>().map_err(serde::de::Error::custom),
        StringOrInt::Number(i) => Ok(i),
    }
}