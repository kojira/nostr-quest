use rand::Rng;
use rusqlite::{Connection, Error, Result};
use std::error::Error as StdError;
use std::fmt;

// ユーザーの情報を保持する構造体
pub struct User {
    pub user_id: i32,
    pub npub: String,
    pub level: i32,
    pub experience: i32,
    pub gold: i32,
    pub current_hp: i32,
    pub max_hp: i32,
    pub current_mp: i32,
    pub max_mp: i32,
    pub attack: i32,
    pub defense: i32,
    pub agility: i32,
    pub luck: i32,
}

#[derive(Debug)]
pub(crate) struct UserAlreadyExistsError {
    npub: String,
}

impl fmt::Display for UserAlreadyExistsError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "User with npub '{}' already exists", self.npub)
    }
}

impl StdError for UserAlreadyExistsError {}

#[derive(Debug)]
struct UserNotFoundError {
    npub: String,
}

impl fmt::Display for UserNotFoundError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "User with npub '{}' not found", self.npub) // エラーメッセージ
    }
}

impl StdError for UserNotFoundError {}

fn distribute_bonus_points() -> (i32, i32, i32, i32, i32, i32) {
    let mut rng = rand::thread_rng();
    let bonus_points = rng.gen_range(5..31); // ボーナスポイントの総数
    let mut remaining_points = bonus_points; // 残っているポイント

    // 各属性のボーナスポイント
    let mut hp_bonus = 0;
    let mut mp_bonus = 0;
    let mut agility_bonus = 0;
    let mut luck_bonus = 0;
    let mut attack_bonus = 0;
    let mut defense_bonus = 0;

    // 残りのポイントがゼロになるまでループ
    while remaining_points > 0 {
        // ランダムに属性を選び、ポイントを振り分ける
        let attribute = rng.gen_range(0..6); // 0: HP, 1: MP, 2: Agility, 3: Luck 4:attack 5: defense
        let points = rng.gen_range(1..=remaining_points); // 割り当てるポイント

        match attribute {
            0 => hp_bonus += points,      // HPにポイントを追加
            1 => mp_bonus += points,      // MPにポイントを追加
            2 => agility_bonus += points, // Agilityにポイントを追加
            3 => luck_bonus += points,    // Luckにポイントを追加
            4 => attack_bonus += points,  // attackにポイントを追加
            5 => defense_bonus += points, // defenseにポイントを追加
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
        luck_bonus,
    ) // 各属性のボーナスポイントを返す
}

pub fn add_user(conn: &Connection, npub: &str) -> Result<User, Box<dyn StdError>> {
    let (hp_bonus, mp_bonus, attack_bonus, defense_bonus, agility_bonus, luck_bonus) =
        distribute_bonus_points();

    if let Err(e) = conn.execute(
        "INSERT INTO users (npub, current_hp, max_hp, current_mp, max_mp, attack, defense, agility, luck)
       VALUES (?1, ?2, ?2, ?3, ?3, ?4, ?5, ?6, ?7)",
        rusqlite::params![
            npub,
            10 + hp_bonus,
            0 + mp_bonus,
            3 + attack_bonus,
            3 + defense_bonus,
            3 + agility_bonus,
            luck_bonus,
        ],
    ) {
        if let Error::SqliteFailure(err, Some(msg)) = &e {
            if err.code == rusqlite::ErrorCode::ConstraintViolation {
                if msg.contains("UNIQUE constraint failed") {
                    return Err(Box::new(UserAlreadyExistsError {
                        npub: npub.to_string(),
                    }));
                }
            }
        }
        return Err(Box::new(e));
    }

    let user = conn.query_row(
        "SELECT 
          user_id,
          npub,
          level,
          experience,
          gold,
          current_hp,
          max_hp,
          current_mp,
          max_mp,
          attack,
          defense,
          agility,
          luck
      FROM users WHERE npub = ?1",
        rusqlite::params![npub],
        |row| {
            Ok(User {
                user_id: row.get(0)?,
                npub: row.get(1)?,
                level: row.get(2)?,
                experience: row.get(3)?,
                gold: row.get(4)?,
                current_hp: row.get(5)?,
                max_hp: row.get(6)?,
                current_mp: row.get(7)?,
                max_mp: row.get(8)?,
                attack: row.get(9)?,
                defense: row.get(10)?,
                agility: row.get(11)?,
                luck: row.get(12)?,
            })
        },
    )?;

    Ok(user)
}

pub fn get_user_by_npub(conn: &Connection, npub: &str) -> Result<User, Box<dyn StdError>> {
    let result = conn.query_row(
        "SELECT 
            user_id,
            npub,
            level,
            experience,
            gold,
            current_hp,
            max_hp,
            current_mp,
            max_mp,
            attack,
            defense,
            agility,
            luck
        FROM users WHERE npub = ?1",
        rusqlite::params![npub],
        |row| {
            Ok(User {
                user_id: row.get(0)?,
                npub: row.get(1)?,
                level: row.get(2)?,
                experience: row.get(3)?,
                gold: row.get(4)?,
                current_hp: row.get(5)?,
                max_hp: row.get(6)?,
                current_mp: row.get(7)?,
                max_mp: row.get(8)?,
                attack: row.get(9)?,
                defense: row.get(10)?,
                agility: row.get(11)?,
                luck: row.get(12)?,
            })
        },
    );

    // ユーザーが見つからなかった場合
    if let Err(rusqlite::Error::QueryReturnedNoRows) = result {
        return Err(Box::new(UserNotFoundError {
            npub: npub.to_string(),
        }));
    }

    result.map_err(|e| Box::new(e) as Box<dyn StdError>)
}

pub fn update_user(conn: &Connection, user: &User) -> Result<(), Box<dyn StdError>> {
    conn.execute(
        "UPDATE users 
       SET 
           npub = ?1,
           level = ?2,
           experience = ?3,
           gold = ?4,
           current_hp = ?5,
           max_hp = ?6,
           current_mp = ?7,
           max_mp = ?8,
           attack = ?9,
           defense = ?10,
           agility = ?11,
           luck = ?12 
       WHERE user_id = ?13",
        rusqlite::params![
            user.npub,
            user.level,
            user.experience,
            user.gold,
            user.current_hp,
            user.max_hp,
            user.current_mp,
            user.max_mp,
            user.attack,
            user.defense,
            user.agility,
            user.luck,
            user.user_id
        ],
    )?;

    Ok(())
}

// ユーザーを削除
pub fn delete_user(conn: &Connection, user_id: i32) -> Result<(), Box<dyn StdError>> {
    conn.execute(
        "DELETE FROM users WHERE user_id = ?1",
        rusqlite::params![user_id],
    )?;

    Ok(())
}
