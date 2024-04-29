use crate::battle;
use crate::config;
use crate::gpt;
use crate::monsters;
use crate::users;
use crate::util;
use nostr_sdk::prelude::*;
use rusqlite::Connection;

pub async fn command_handler(
    config: &config::AppConfig,
    conn: &Connection,
    my_keys: Keys,
    event: &Event,
) -> Result<bool> {
    println!("command_handler");
    let admin_pubkeys = &config.bot.admin_pubkeys;
    let bot_names = &config.bot.bot_names;
    let mut handled: bool = false;
    let secret_key = my_keys.secret_key().unwrap().to_string();
    let is_admin = admin_pubkeys.iter().any(|s| *s == event.pubkey.to_string());
    let has_mention = util::extract_mention(bot_names, event).unwrap();
    println!("has_mention:{}", has_mention);

    let mut message = event.content.clone();
    if event.kind() == Kind::EncryptedDirectMessage {
        if let Ok(msg) = nip04::decrypt(my_keys.secret_key()?, event.author_ref(), event.content())
        {
            message = msg;
        } else {
            println!("Impossible to decrypt direct message");
            return Ok(false);
        }
    }

    if message.starts_with(".guild join") {
        println!(".guild join");
        join_guild(config, &conn, event, &secret_key).await?;
    } else if message.contains(".status") {
        println!(".status");
        status(config, &conn, event, &secret_key).await?;
    } else if message.contains(".leveling") {
        println!(".leveling");
        leveling(config, &conn, event, &secret_key).await?;
        handled = true;
    }
    if is_admin {
        println!("admin");
        if message.contains(".add monster") {
            println!(".add monster");
            add_monster(config, &conn, event, &message, &secret_key).await?;
        } else if message.contains(".spawn") {
            println!(".spawn");
            spawn_monster(config, &conn, event, &message, &secret_key).await?;
        }
    }

    Ok(handled)
}

async fn join_guild(
    config: &config::AppConfig,
    conn: &Connection,
    event: &Event,
    secret_key: &str,
) -> Result<()> {
    let prompt = &config.bot.prompt;

    match users::add_user(&conn, &event.author().to_string()) {
        Ok(user) => {
            let reply = gpt::get_reply(
                &prompt,
                "初めてのギルド登録手続き、手際よく終わってありがとう。",
                50,
            )
            .await
            .unwrap();
            if reply.len() > 0 {
                let text = &format!(
              "\nあなたのステータスは以下の通りですわ。\nlevel:{}\nたいりょく:{}/{}\nまりょく:{}/{}\nちから:{}\nしゅびりょく:{}\nすばやさ{}\nうん:{}\nけいけんち:{}\nGOLD:{}",
              user.level,
              user.current_hp,
              user.max_hp,
              user.current_mp,
              user.max_mp,
              user.attack,
              user.defense,
              user.agility,
              user.luck,
              user.experience,
              user.gold,
          );
                let answer = &format!("{}{}", reply, text);
                util::reply_to(config, event.clone(), secret_key, &answer).await?;
            }
        }
        Err(e) => {
            if let Some(already_exists_error) = e.downcast_ref::<users::UserAlreadyExistsError>() {
                println!("Error: {}", already_exists_error);
                util::reply_to(
                    config,
                    event.clone(),
                    secret_key,
                    "あら、あなたはすでにギルドに登録済みですわよ。",
                )
                .await?;
            } else {
                eprintln!("Error adding user: {:?}", e); // その他のエラー
                util::reply_to(
                  config,
                  event.clone(),
                  secret_key,
                  "あら、何かシステムが異常なようですわ！急ぎマスターに報告して参ります！しばらくお待ちくださいまし。",
              )
              .await?;
            }
        }
    }

    Ok(())
}

async fn status(
    config: &config::AppConfig,
    conn: &Connection,
    event: &Event,
    secret_key: &str,
) -> Result<()> {
    match users::get_user_by_npub(&conn, &event.author().to_string()) {
        Ok(user) => {
            let next_exp = util::experience_all_for_level(user.level as u32 + 2) + 1;
            let answer = &format!(
              "あなたのステータスは以下の通りですわ。\nlevel:{}\nたいりょく:{}/{}\nまりょく:{}/{}\nちから:{}\nしゅびりょく:{}\nすばやさ{}\nうん:{}\nけいけんち:{}\nGOLD:{}\nつぎのlevelまで:{}",
              user.level,
              user.current_hp,
              user.max_hp,
              user.current_mp,
              user.max_mp,
              user.attack,
              user.defense,
              user.agility,
              user.luck,
              user.experience,
              user.gold,
              (next_exp - user.experience as u32) as i32,
            );
            util::reply_to(config, event.clone(), secret_key, &answer).await?;
        }
        Err(_) => {
            util::reply_to(
                config,
                event.clone(),
                secret_key,
                "あなたはまだギルドに登録されておられないようですわね。",
            )
            .await?;
        }
    }
    Ok(())
}

async fn leveling(
    config: &config::AppConfig,
    conn: &Connection,
    event: &Event,
    secret_key: &str,
) -> Result<()> {
    match users::get_user_by_npub(&conn, &event.author().to_string()) {
        Ok(user) => {
            if let Some(monster) = monsters::get_random_monster(&conn)? {
                let result = battle::simulate_battle(conn, &user, &monster);
                let message = if result.victory {
                    "ご無事で何よりでした。"
                } else {
                    "無茶をなさったようですね。こうして戻ってこられるのも不滅の鍵の冒険者の福音ですわね。"
                };
                let answer = format!(
                    "おかえりなさいまし。冒険日誌を見せてくださいね\n```\n{}```\n\n{}",
                    result.battle_log, message
                );
                util::reply_to(config, event.clone(), secret_key, &answer).await?;
            } else {
                util::reply_to(
                    config,
                    event.clone(),
                    secret_key,
                    "今はモンスターがいないようですわね。",
                )
                .await?;
            }
        }
        Err(_) => {
            util::reply_to(
                config,
                event.clone(),
                secret_key,
                "安全のため、ギルド登録なしの冒険は禁じられておりますわ。",
            )
            .await?;
        }
    }

    Ok(())
}

async fn add_monster(
    config: &config::AppConfig,
    conn: &Connection,
    event: &Event,
    message: &str,
    secret_key: &str,
) -> Result<()> {
    let lines: Vec<String> = message.lines().map(|line| line.to_string()).collect();

    println!("len:{}", lines.len());
    if lines.len() == 9 {
        conn.execute(
          "INSERT INTO monster_master (level, name, picture, attack, defense, agility, experience_reward, gold_reward) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)",
          rusqlite::params!(
            lines[1],
            lines[2],
            lines[3],
            lines[4],
            lines[5],
            lines[6],
            lines[7],
            lines[8],
          ),
      )?;
        util::reply_to(
            config,
            event.clone(),
            secret_key,
            &format!("{}をマスターに追加致しましたわ。", lines[2]),
        )
        .await?;
    }

    Ok(())
}

async fn spawn_monster(
    config: &config::AppConfig,
    conn: &Connection,
    event: &Event,
    message: &str,
    secret_key: &str,
) -> Result<()> {
    let lines: Vec<String> = message.lines().map(|line| line.to_string()).collect();
    println!("len:{}", lines.len());
    if lines.len() == 3 {
        let mut suceess = false;
        let monster_id = lines[1].parse::<i32>();
        if !monster_id.is_err() {
            let amount_op = lines[2].parse::<i32>();
            if !amount_op.is_err() {
                let amount = amount_op.unwrap();
                let monster = monsters::spawn_monster(conn, monster_id.unwrap(), amount).unwrap();
                util::reply_to(
                    config,
                    event.clone(),
                    secret_key,
                    &format!("{} を {}体召喚しましたわ。マスター。", monster.name, amount),
                )
                .await?;
                suceess = true;
            }
        }
        if !suceess {
            util::reply_to(
                config,
                event.clone(),
                secret_key,
                &format!("指示がおかしいようですわね。しっかりしてくださいね。"),
            )
            .await?;
        }
    }

    Ok(())
}
