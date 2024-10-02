use std::{fs::{File, OpenOptions}, io::{BufReader, Write}};

use axum::async_trait;
use tracing::debug;
use twitch_irc::login::{TokenStorage, UserAccessToken};

#[derive(Debug)]
pub struct CustomTokenStorage {
    pub(crate) storage_location: String
}

#[async_trait]
impl TokenStorage for CustomTokenStorage {
    type LoadError = anyhow::Error; // or some other error
    type UpdateError = anyhow::Error;

    async fn load_token(&mut self) -> anyhow::Result<UserAccessToken> {
        debug!("Loaded existing token");
        let file = File::open(self.storage_location.as_str()).expect("failed to open");
        let reader = BufReader::new(file);
        let token: UserAccessToken = serde_json::from_reader(reader).expect("failed to deserialize");
        Ok(token)
    }

    async fn update_token(&mut self, token: &UserAccessToken) -> anyhow::Result<()> {
        debug!("Updated token");
        let json = serde_json::to_string(token)?;
        let mut file = OpenOptions::new().write(true).create(true).open(self.storage_location.as_str())?;
        file.write_all(json.as_bytes())?;
        Ok(())
    }
}
