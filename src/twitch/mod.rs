mod storage;

use std::env::var;

use tracing::debug;
use tracing::info;
use twitch_irc::login::RefreshingLoginCredentials;
use twitch_irc::message::ServerMessage;
use twitch_irc::ClientConfig;
use twitch_irc::SecureTCPTransport;
use twitch_irc::TwitchIRCClient;

use crate::commands;
use crate::commands::MessageContext;
use storage::CustomTokenStorage;

pub type TwitchCredentials = RefreshingLoginCredentials<CustomTokenStorage>;

async fn handle_message(message: ServerMessage, client: TwitchIRCClient<SecureTCPTransport, TwitchCredentials>) {
    match message {
        ServerMessage::Privmsg(message) => {
            let _ = commands::handle(MessageContext::Twitch((message.sender, client)), message.message_text).await;
        }
        _ => {}
    }
}

pub static CHANNEL_LOGIN: &str = "the_nuclear_nexus";


pub async fn setup_twitch() {
    // default configuration is to join chat as anonymous.
    let storage = CustomTokenStorage {
        storage_location: var("TOKEN_LOCATION").unwrap()
    };

    debug!("Created storage");

    let credentials = RefreshingLoginCredentials::init(var("TWITCH_CLIENT_ID").unwrap(), var("TWITCH_CLIENT_SECRET").unwrap(), storage);
    
    debug!("Created credentials");

    let config = ClientConfig::new_simple(credentials);
    debug!("Created config");
    let (mut incoming_messages, client) =
        TwitchIRCClient::<SecureTCPTransport, TwitchCredentials>::new(config);

    debug!("Created client");
    
    // first thing you should do: start consuming incoming messages,
    // otherwise they will back up.
    let c2 = client.clone();
    let join_handle = tokio::spawn(async move {
        while let Some(message) = incoming_messages.recv().await {
            // debug!("{:?}", message);
            handle_message(message, c2.clone()).await;
        }
    });

    // join a channel
    // This function only returns an error if the passed channel login name is malformed,
    // so in this simple case where the channel name is hardcoded we can ignore the potential
    // error with `unwrap`.
    client.join(CHANNEL_LOGIN.to_owned()).unwrap();
    info!("Joined the chat");
    // keep the tokio executor alive.
    // If you return instead of waiting the background task will exit.
    join_handle.await.unwrap();
    debug!("Exited handle");
}
