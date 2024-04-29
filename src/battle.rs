use rand::Rng;
use rusqlite::{Connection, Result};

use crate::{
    monsters::{self, Monster},
    users::{self, User},
    util,
};

pub struct BattleResult {
    pub user_id: i32,
    pub monster_id: i32,
    pub victory: bool,
    pub experience_gain: i32,
    pub gold_gain: i32,
    pub battle_log: String,
}

fn should_dodge(defense_agility: i32, attack_agility: i32) -> bool {
    let agility_difference = defense_agility - attack_agility; // 素早さの差
    let dodge_probability = 0.1 + (agility_difference as f64 / 100.0).max(0.0).min(1.0); // 確率を正規化
    println!(
        "agility_difference:{} 確率:{}",
        agility_difference, dodge_probability
    );
    let mut rng = rand::thread_rng();

    rng.gen::<f64>() < dodge_probability // 確率に基づいて判定
}

pub fn simulate_battle(conn: &Connection, user: &User, monster: &Monster) -> BattleResult {
    let mut battle_log = String::new();
    let mut rng = rand::thread_rng();
    let mut user_hp = user.current_hp;
    let mut monster_hp = monster.hp;
    let npub1 = util::get_npub1(user.npub.clone()).unwrap();

    println!("{}が現れた！", monster.name);

    battle_log.push_str(&format!(
        "{}\n{}が現れた！\n",
        monster.picture, monster.name
    ));

    let mut turn = 20;

    while user_hp > 0 && monster_hp > 0 && turn > 0 {
        println!("user_hp:{} monster_hp:{}", user_hp, monster_hp);
        // ユーザーの攻撃
        battle_log.push_str(&format!("nostr:{} のこうげき！\n", npub1));
        if !should_dodge(
            monster.agility,
            user.agility + rng.gen_range(0..user.luck + 1),
        ) {
            let attack =
                rng.gen_range(user.attack..user.attack + 2) + rng.gen_range(0..user.luck + 1);
            println!("attack:{}", attack);
            let damage = (attack - monster.defense).max(0);
            monster_hp -= damage;
            battle_log.push_str(&format!(
                "{} に {} のダメージをあたえた！\n",
                monster.name, damage
            ));
        } else {
            battle_log.push_str(&format!("{}はひらりとかわした！\n", monster.name));
        }

        // モンスターの攻撃
        if monster_hp > 0 {
            battle_log.push_str(&format!("{} のこうげき！\n", monster.name));
            if !should_dodge(user.agility, monster.agility) {
                let attack = rng.gen_range(monster.attack..monster.attack + 2);
                let damage = (attack - user.defense).max(0);
                user_hp -= damage;
                battle_log.push_str(&format!(
                    "nostr:{} は {} のダメージをうけた！ HP:{}/{}\n",
                    npub1, damage, user_hp, user.max_hp
                ));
            } else {
                battle_log.push_str(&format!("nostr:{}はひらりとかわした！\n", npub1));
            }
        }
        turn -= 1;
    }

    // 戦闘結果の決定
    if user_hp > 0 && monster_hp > 0 {
        battle_log.push_str(&format!("{}はにげだした！", monster.name));
        BattleResult {
            user_id: user.user_id,
            monster_id: monster.id,
            victory: false,
            experience_gain: 0,
            gold_gain: 0,
            battle_log,
        }
    } else {
        let victory = user_hp > 0;
        let experience_gain = if victory {
            monster.experience_reward
        } else {
            0
        };
        let gold_gain = if victory { monster.gold_reward } else { 0 };

        if victory {
            battle_log.push_str(&format!("{} を倒した！\n", monster.name));
            battle_log.push_str(&format!(
                "経験値 {} と {} GOLD を手に入れた！\n",
                experience_gain, gold_gain
            ));
            let result = monsters::defeat_monster(&conn, monster, user.user_id);
            if result.is_err() {
                println!("Error defeat monster: {:?}", result.err().unwrap());
            } else {
                let next_exp = user.experience + experience_gain;
                let level = util::level_from_experience(next_exp.try_into().unwrap());
                println!("next_exp:{} level:{}", next_exp, level);
                if level > user.level.try_into().unwrap() {
                    battle_log.push_str(&format!("nostr:{} はレベルがあがった！\n", npub1));
                }

                let user_update = User {
                    user_id: user.user_id,
                    npub: user.npub.clone(),
                    level: level.try_into().unwrap(),
                    experience: user.experience + experience_gain,
                    gold: user.gold + gold_gain,
                    current_hp: user_hp,
                    max_hp: user.max_hp,
                    current_mp: user.current_mp,
                    max_mp: user.max_mp,
                    attack: user.attack,
                    defense: user.defense,
                    agility: user.agility,
                    luck: user.luck,
                };
                let result = users::update_user(&conn, &user_update);
                if result.is_err() {
                    println!("Error update user: {:?}", result.err().unwrap());
                }
            }
        } else {
            battle_log.push_str(&format!(
                "nostr:{}はしんでしまった！\nGOLDが半分になってしまった！\n",
                npub1
            ));
            let user_update = User {
                user_id: user.user_id,
                npub: user.npub.clone(),
                level: user.level,
                experience: user.experience,
                gold: user.gold / 2,
                current_hp: user.max_hp,
                max_hp: user.max_hp,
                current_mp: user.max_mp,
                max_mp: user.max_mp,
                attack: user.attack,
                defense: user.defense,
                agility: user.agility,
                luck: user.luck,
            };
            let result = users::update_user(&conn, &user_update);
            if result.is_err() {
                println!("Error update user: {:?}", result.err().unwrap());
            }
        }

        BattleResult {
            user_id: user.user_id,
            monster_id: monster.id,
            victory,
            experience_gain,
            gold_gain,
            battle_log,
        }
    }
}
