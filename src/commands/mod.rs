use twitch_irc::{message::TwitchUserBasics, SecureTCPTransport, TwitchIRCClient};

use crate::twitch::{TwitchCredentials, CHANNEL_LOGIN};


pub mod claim;
pub mod spin;
pub mod balance;
pub mod leaderboard;

const PREFIX: &'static str = "$";

pub async fn handle(sender: MessageContext, content: String) {
    if !content.starts_with(PREFIX) {
        return;
    }

    let mut args: Vec<&str> = content[1..].split(" ").collect();

    let command = args[0];
    args.remove(0);

    let _ = match command {
        "claim" => claim::handle(sender, args).await,
        "spin" => spin::handle(sender, args).await,
        "balance" => balance::handle(sender, args).await,
        "leaderboard" => leaderboard::handle(sender, args).await,
        _ => Ok(())
    };
}

pub enum MessageContext {
    Twitch((TwitchUserBasics, TwitchIRCClient<SecureTCPTransport, TwitchCredentials>)),
    YouTube(String)
}

impl MessageContext {
    pub async fn send(self: &Self, message: &str) {
        match self {
            MessageContext::Twitch((sender, client)) => {
                let message = message.replace("%username%", &sender.name);
                let _ = client.say(CHANNEL_LOGIN.to_owned(), message).await;
            },
            MessageContext::YouTube(_) => todo!(),
        }
    }

    pub fn user_id(self: &Self) -> String {
        match self {
            MessageContext::Twitch((sender, _)) => sender.id.clone(),
            MessageContext::YouTube(_) => todo!(),
        }
    }
}