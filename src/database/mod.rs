use tokio_postgres::{Client, NoTls};

use crate::{
    commands::MessageContext,
    polls::{Poll, PollChoice},
};

pub struct Database {
    pub client: Client,
}

impl Database {
    pub async fn connect() -> Self {
        let (client, connection) = tokio_postgres::connect(
            "postgresql://dboperator:operatorpass123@localhost:5243/postgres",
            NoTls,
        )
        .await
        .unwrap();

        tokio::spawn(async move {
            if let Err(e) = connection.await {
                eprintln!("connection error: {}", e);
            }
        });

        let _ = client
            .batch_execute(
                "
            
        CREATE TABLE IF NOT EXISTS app_user(
            id VARCHAR PRIMARY KEY,
            username VARCHAR NOT NULL,
            platform VARCHAR NOT NULL,
            points INT NOT NULL
        );

        CREATE TABLE IF NOT EXISTS polls(
            id VARCHAR PRIMARY KEY,
            title VARCHAR NOT NULL,
            start_time INT NOT NULL,
            duration INT NOT NULL
        );

        CREATE TABLE IF NOT EXISTS poll_choices(
            owner_id VARCHAR PRIMARY KEY,
            index INT NOT NULL,
            name VARCHAR NOT NULL
        );

        CREATE TABLE IF NOT EXISTS poll_votes(
            poll_id VARCHAR PRIMARY KEY,
            user_id VARCHAR NOT NULL,
            choice_index INT NOT NULL
        );
    ",
            )
            .await
            .expect("Failed to create table!");

        let db: Database = Database { client };

        db
    }

    async fn add_user(self: &mut Self, id: String, username: String, platform: &str) {
        let _ = self
            .client
            .execute(
                "
            INSERT INTO app_user (id, username, platform, points) VALUES ($1, $2, $3, 0)
        ",
                &[&id, &username, &platform],
            )
            .await
            .expect("Failed to insert user");
    }

    pub async fn get_user(self: &mut Self, context: &MessageContext) -> AppUser {
        let (id, username, platform) = match context {
            MessageContext::Twitch((sender, _)) => {
                let id = sender.id.clone();
                let username = sender.name.clone();
                (id, username, "TWITCH")
            }
            MessageContext::YouTube(_) => todo!(),
        };

        let matches = self
            .client
            .query(
                "
            SELECT id, username, platform, points FROM app_user
            WHERE id = $1 and platform = $2;
        ",
                &[&id.as_str(), &platform],
            )
            .await
            .expect("failed to query database");

        if matches.len() == 0 {
            self.add_user(id.clone(), username.clone(), platform).await;

            AppUser {
                id: id,
                username: username,
                platform: if platform == "TWITCH" {
                    Platform::Twitch
                } else {
                    Platform::Youtube
                },
                points: 0,
            }
        } else {
            let data = &matches[0];

            let id = data.get(0);
            let username = data.get(1);
            let platform = data.get(2);
            let points = data.get(3);

            AppUser {
                id: id,
                username: username,
                platform: Platform::from(platform),
                points: points,
            }
        }
    }

    pub async fn set_user(self: &mut Self, data: &AppUser) {
        let _ = self
            .client
            .execute(
                "
            UPDATE app_user SET points = $1 WHERE id = $2
        ",
                &[&data.points, &data.id],
            )
            .await;
    }

    pub async fn get_poll(self: &mut Self, poll_id: String) -> anyhow::Result<Poll> {
        let poll_row = self
            .client
            .query_one(
                "SELECT id, title, start_time, duration FROM polls WHERE id = $1",
                &[&poll_id],
            )
            .await?;
        let poll_choices = self
            .client
            .query(
                "SELECT name FROM poll_choices WHERE owner_id = $1 ORDER BY index",
                &[&poll_id],
            )
            .await?;

        Ok(Poll {
            id: poll_id,
            title: poll_row.get(1),
            start_time: poll_row.get(2),
            duration: poll_row.get(3),
            choices: poll_choices
                .iter()
                .map(|r| PollChoice { name: r.get(0) })
                .collect(),
        })
    }

    pub async fn add_poll(self: &mut Self, poll: Poll) -> anyhow::Result<()> {
        let _ = self
            .client
            .execute(
                "INSERT INTO polls (id, title, start_time, duration) VALUES ($1, $2, $3, $4)",
                &[&poll.id, &poll.title, &poll.start_time, &poll.duration],
            )
            .await?;

        for i in 0..poll.choices.len() {
            let choice = &poll.choices[i];

            let _ = self.client.execute(
                "INSERT INTO poll_choices (owner_id, index, name) VALUES ($1, $2, $3)",
                &[&poll.id, &(i as i32), &choice.name],
            );
        }

        Ok(())
    }
}

#[derive(Debug)]
pub struct AppUser {
    id: String,
    username: String,
    platform: Platform,
    pub points: i32,
}

#[derive(Debug)]
pub enum Platform {
    Twitch,
    Youtube,
}

impl Platform {
    pub fn from(val: &str) -> Platform {
        match val.to_uppercase().as_str() {
            "TWITCH" => Platform::Twitch,
            "YOUTUBE" => Platform::Youtube,
            _ => panic!("Invalid string passed to Platform::from"),
        }
    }

    pub fn as_str(self: &Self) -> &str {
        match self {
            Platform::Twitch => "TWITCH",
            Platform::Youtube => "YOUTUBE",
        }
    }
}
