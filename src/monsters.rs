use rand::Rng;
use rusqlite::{Connection, Error, Result};
use std::error::Error as StdError;
use std::fmt;

// monsterの情報を保持する構造体
pub struct Monster {
    pub id: i32,
    pub level: i32,
    pub status: i32,
    pub name: String,
    pub picture: String,
    pub attack: i32,
    pub defense: i32,
    pub agility: i32,
    pub experience_reward: i32,
    pub gold_reward: i32,
    pub hp: i32,
    pub mp: i32,
    pub defeat_user_id: i32,
}

fn distribute_bonus_points(bonus_min: i32, bonus_max: i32) -> (i32, i32, i32, i32, i32) {
    let mut rng = rand::thread_rng();
    let bonus_points = rng.gen_range(bonus_min..bonus_max); // ボーナスポイントの総数
    let mut remaining_points = bonus_points; // 残っているポイント

    // 各属性のボーナスポイント
    let mut hp_bonus = 0;
    let mut mp_bonus = 0;
    let mut agility_bonus = 0;
    let mut attack_bonus = 0;
    let mut defense_bonus = 0;

    // 残りのポイントがゼロになるまでループ
    while remaining_points > 0 {
        // ランダムに属性を選び、ポイントを振り分ける
        let attribute = rng.gen_range(0..5); // 0: HP, 1: MP, 2: Agility, 3:attack 4: defense
        let points = rng.gen_range(1..=remaining_points); // 割り当てるポイント

        match attribute {
            0 => hp_bonus += points,      // HPにポイントを追加
            1 => mp_bonus += points,      // MPにポイントを追加
            2 => agility_bonus += points, // Agilityにポイントを追加
            3 => attack_bonus += points,  // attackにポイントを追加
            4 => defense_bonus += points, // defenseにポイントを追加
            _ => unreachable!(),          // 予期しない値を避ける
        }

        remaining_points -= points; // 残りのポイントを更新
    }

    (
        hp_bonus,
        mp_bonus,
        attack_bonus,
        defense_bonus,
        agility_bonus,
    ) // 各属性のボーナスポイントを返す
}

pub fn spawn_monster(
    conn: &Connection,
    monster_id: i32,
    amount: i32,
) -> Result<Monster, Box<dyn StdError>> {
    let monster = get_monster_by_id(conn, monster_id).unwrap();
    let mut rng = rand::thread_rng();

    for n in 0..amount {
        println!("index:{}", n);
        let (hp_bonus, mp_bonus, attack_bonus, defense_bonus, agility_bonus) =
            distribute_bonus_points(rng.gen_range(1..3), rng.gen_range(4..10));
        let result = conn.execute(
            "INSERT INTO monsters (
                    monster_id,
                    level,
                    status,
                    name,
                    picture,
                    attack,
                    defense,
                    agility,
                    experience_reward,
                    gold_reward,
                    hp,
                    mp)
               VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12)",
            rusqlite::params![
                monster_id,
                monster.level,
                monster.status,
                monster.name,
                monster.picture,
                monster.attack + attack_bonus,
                monster.defense + defense_bonus,
                monster.agility + agility_bonus,
                monster.experience_reward,
                monster.gold_reward,
                monster.hp + hp_bonus,
                monster.mp + mp_bonus
            ],
        );
        if result.is_err() {
            println!("Error inserting monster: {:?}", result.err().unwrap());
        }
    }
    Ok(monster)
}

pub fn get_monster_by_id(conn: &Connection, id: i32) -> Result<Monster, Box<dyn StdError>> {
    let result = conn.query_row(
        "SELECT
          id,
          level,
          status,
          name,
          picture,
          attack,
          defense,
          agility,
          experience_reward,
          gold_reward,
          hp,
          mp
        FROM monster_master WHERE id = ?1 and status = 1",
        rusqlite::params![id],
        |row| {
            Ok(Monster {
                id: row.get(0)?,
                level: row.get(1)?,
                status: row.get(2)?,
                name: row.get(3)?,
                picture: row.get(4)?,
                attack: row.get(5)?,
                defense: row.get(6)?,
                agility: row.get(7)?,
                experience_reward: row.get(8)?,
                gold_reward: row.get(9)?,
                hp: row.get(10)?,
                mp: row.get(11)?,
                defeat_user_id: -1,
            })
        },
    );

    result.map_err(|e| Box::new(e) as Box<dyn StdError>)
}

// モンスターテーブルからランダムにモンスターを取得する関数
pub fn get_random_monster(conn: &Connection) -> Result<Option<Monster>, Box<dyn StdError>> {
    // モンスターテーブルの行数を取得
    let mut statement =
        conn.prepare("SELECT * FROM monsters WHERE status=1 ORDER BY RANDOM() LIMIT 1")?;
    let mut monster_iter = statement.query_map([], |row| {
        Ok(Monster {
            id: row.get(0)?,
            level: row.get(2)?,
            status: row.get(3)?,
            name: row.get(4)?,
            picture: row.get(5)?,
            attack: row.get(6)?,
            defense: row.get(7)?,
            agility: row.get(8)?,
            experience_reward: row.get(9)?,
            gold_reward: row.get(10)?,
            hp: row.get(11)?,
            mp: row.get(12)?,
            defeat_user_id: row.get(13)?,
        })
    })?;

    let monster = monster_iter.next().transpose()?; // 最初のモンスターを取得

    Ok(monster)
}

pub fn defeat_monster(
    conn: &Connection,
    monster: &Monster,
    user_id: i32,
) -> Result<(), Box<dyn StdError>> {
    conn.execute(
        "UPDATE monsters 
        SET 
            status = ?1,
            hp = ?2,
            mp = ?3,
            defeat_user_id = ?4
        WHERE id = ?5",
        rusqlite::params![2, monster.hp, monster.mp, user_id, monster.id],
    )?;

    Ok(())
}
