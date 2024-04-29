mod battle;
mod commands;
mod config;
mod db;
mod gpt;
mod users;
mod monsters;
mod util;
use dotenv::dotenv;
use nostr_sdk::prelude::*;
use std::env;
use std::{fs::File, str::FromStr};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    dotenv().ok();
    println!("start");
    let file = File::open("config.yml")?;
    let config: config::AppConfig = serde_yaml::from_reader(file)?;
    let conn = db::connect()?;

    let bot_secret_key = env::var("BOT_SECRETKEY").expect("BOT_SECRETKEY is not set");
    let bot_public_key = env::var("BOT_PUBLICKEY").expect("BOT_PUBLICKEY is not set");

    let my_keys = Keys::from_str(&bot_secret_key)?;

    // Create new client
    let client = Client::new(&my_keys);
    for item in config.relay_servers.read.iter() {
        client.add_relay(item.clone()).await?;
    }
    println!("add_relay");

    // Connect to relays
    client.connect().await;
    println!("client.connect");

    let subscription = Filter::new()
        .kinds([Kind::TextNote, Kind::EncryptedDirectMessage].to_vec())
        .since(Timestamp::now());

    client.subscribe(vec![subscription], None).await;
    println!("subscribe");
    let mut notifications = client.notifications();
    while let Ok(notification) = notifications.recv().await {
        if let RelayPoolNotification::Event {
            relay_url: _,
            subscription_id: _,
            event,
        } = notification
        {
            if event.pubkey.to_string() != bot_public_key {
                if event.kind == Kind::TextNote || event.kind() == Kind::EncryptedDirectMessage {
                    let follower =
                        util::is_follower(&event.pubkey.to_string(), &bot_public_key).await?;
                    if follower {
                        commands::command_handler(&config, &conn, my_keys.clone(), &event).await?;
                    }
                } else {
                    println!("{:?}", event);
                }
            }
        }
    }

    Ok(())
}
