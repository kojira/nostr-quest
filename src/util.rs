use crate::config;
use nostr_sdk::prelude::*;
use std::fs::File;
use std::thread;
use std::time::Duration;

pub async fn is_follower(user_pubkey: &str, bot_pubkey: &str) -> Result<bool> {
    let file = File::open("config.yml")?;
    let config: config::AppConfig = serde_yaml::from_reader(file)?;
    let my_keys: Keys = Keys::generate();
    let client = Client::new(my_keys);
    for item in config.relay_servers.read.iter() {
        client.add_relay(item.clone()).await?;
    }
    client.connect().await;
    let publickey = PublicKey::from_hex(user_pubkey).unwrap();

    let filter = Filter::new()
        .authors([publickey].to_vec())
        .kinds([nostr_sdk::Kind::ContactList].to_vec())
        .limit(1);

    let events = client
        .get_events_of(vec![filter], Some(Duration::from_secs(5)))
        .await?;

    let detect = events.first().map_or(false, |first_event: &Event| {
        first_event.tags.iter().any(|tag| {
            let tags_slice = tag.as_vec();
            if tags_slice.len() >= 2 {
                return tags_slice[0] == "p" && tags_slice[1] == bot_pubkey;
            }
            false
        })
    });

    Ok(detect)
}

pub fn extract_mention(bot_names: &Vec<String>, event: &Event) -> Result<bool> {
    let mut has_mantion = false;
    for _name in bot_names {
        let words: Vec<String> = event
            .content
            .split_whitespace()
            .map(|word| word.to_string())
            .collect();

        if words.len() > 0 && (event.content.contains(_name)) {
            println!("name:{}", _name);
            has_mantion = true;
            break;
        }
        if _name.len() == 64 {
            for _tag in event.tags.iter() {
                if _tag.as_vec().len() > 1 {
                    if _tag.as_vec()[0].len() == 1 {
                        if _tag.as_vec()[0].starts_with('p') {
                            if _tag.as_vec()[1].to_string() == *_name {
                                has_mantion = true;
                                break;
                            }
                        }
                    }
                }
            }
        }
    }

    Ok(has_mantion)
}

pub async fn send_to(config: &config::AppConfig, secret_key: &str, text: &str) -> Result<()> {
    let bot_keys = Keys::parse(secret_key)?;
    let client_temp = Client::new(&bot_keys);
    for item in config.relay_servers.write.iter() {
        client_temp.add_relay(item.clone()).await.unwrap();
    }
    client_temp.connect().await;
    let tags: Vec<Tag> = vec![];
    let event_id = client_temp
        .publish_text_note(format!("{}", text), tags)
        .await?;
    println!("publish_text_note! eventId:{}", event_id);
    thread::sleep(Duration::from_secs(10));
    client_temp.shutdown().await?;
    Ok(())
}

pub async fn reply_to(
    config: &config::AppConfig,
    event: Event,
    secret_key: &str,
    text: &str,
) -> Result<Event> {
    if event.kind == Kind::EncryptedDirectMessage {
      let bot_keys = Keys::parse(secret_key)?;
      let client_temp = Client::new(&bot_keys);
      for item in config.relay_servers.write.iter() {
          client_temp.add_relay(item.clone()).await.unwrap();
      }
      client_temp.connect().await;
      client_temp
      .send_direct_msg(event.author(), text, Some(event.id()))
      .await?;      
    } else {
      let event_copy = reply_to_by_event_id_pubkey(config, event.id, event.pubkey, secret_key, text).await?;
      return Ok(event_copy);
    }
    let event_copy = event.clone();
    Ok(event_copy)
}

pub async fn reply_to_by_event_id_pubkey(
    config: &config::AppConfig,
    reply_event_id: EventId,
    reply_pubkey: PublicKey,
    secret_key: &str,
    text: &str,
) -> Result<Event> {
    let bot_keys = Keys::parse(secret_key)?;
    let client_temp = Client::new(&bot_keys);
    for item in config.relay_servers.write.iter() {
        client_temp.add_relay(item.clone()).await.unwrap();
    }
    client_temp.connect().await;

    let event = EventBuilder::text_note(
        text,
        [Tag::event(reply_event_id), Tag::public_key(reply_pubkey)],
    )
    .to_event(&bot_keys)
    .unwrap();
    let event_copy = event.clone();
    client_temp.send_event(event).await?;

    println!("publish_text_note!");
    thread::sleep(Duration::from_secs(1));
    client_temp.shutdown().await?;
    Ok(event_copy)
}

pub fn get_npub1(npub :String) -> Result<String> {
  let publickey = PublicKey::from_hex(npub).unwrap();
  Ok(publickey.to_bech32().unwrap())
}

pub fn experience_for_level(level: u32) -> f64 {
  let k = 5.0; // 定数
  let a = 1.5; // 増加率
  k * (level as f64).powf(a) // 次のレベルに必要な経験値を計算
}

pub fn experience_all_for_level(level: u32) -> u32 {
  let k = 5.0;  // 定数
  let a = 1.5;  // 増加率
  let mut total_experience = 0.0;  // 累積経験値

  // レベル1から指定されたレベルまでの経験値を累積
  for lvl in 1..level {
      total_experience += k * (lvl as f64).powf(a);  // 累積経験値を加算
  }

  total_experience.round() as u32  // 次のレベルに必要な総経験値を返す
}

pub fn level_from_experience(exp: u32) -> u32 {
  let mut level = 1;
  let mut required_exp = experience_for_level(level);

  // 現在のレベルで必要な経験値が、与えられた経験値を超えるまで繰り返す
  while required_exp <= exp.into() {
      level += 1;
      required_exp += experience_for_level(level); // 次のレベルで必要な経験値を加算
  }

  level - 1 // 最終的に達成したレベルを返す
}
