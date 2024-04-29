use rusqlite::{Connection, Result};

fn create_user_table(conn: &Connection) -> Result<()> {
    if let Err(e) = conn.execute(
        "CREATE TABLE IF NOT EXISTS users (
            user_id INTEGER PRIMARY KEY AUTOINCREMENT,
            npub TEXT UNIQUE,
            level INTEGER DEFAULT 1,
            experience INTEGER DEFAULT 0,
            gold INTEGER DEFAULT 0,
            current_hp INTEGER DEFAULT 10,
            max_hp INTEGER DEFAULT 10,
            current_mp INTEGER DEFAULT 0,
            max_mp INTEGER DEFAULT 0,
            attack INTEGER DEFAULT 3,
            defense INTEGER DEFAULT 2,
            agility INTEGER DEFAULT 2,
            luck INTEGER DEFAULT 0
        )",
        [],
    ) {
        eprintln!("Error create_user_table: {:?}", e);
        return Err(e);
    }

    Ok(())
}

fn create_monster_master_table(conn: &Connection) -> Result<()> {
    if let Err(e) = conn.execute(
        "CREATE TABLE IF NOT EXISTS monster_master (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            level INTEGER DEFAULT 1,
            status INTEGER DEFAULT 1,
            name TEXT,
            picture TEXT,
            attack INTEGER,
            defense INTEGER,
            agility INTEGER,
            experience_reward INTEGER,
            gold_reward INTEGER,
            hp INTEGER DEFAULT 10,
            mp INTEGER DEFAULT 5
        )",
        [],
    ) {
        eprintln!("Error create_monster_master_table: {:?}", e);
        return Err(e);
    }

    Ok(())
}

fn create_monster_table(conn: &Connection) -> Result<()> {
    if let Err(e) = conn.execute(
        "CREATE TABLE IF NOT EXISTS monsters (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            monster_id INTEGER,
            level INTEGER DEFAULT 1,
            status INTEGER DEFAULT 1,
            name TEXT,
            picture TEXT,
            attack INTEGER,
            defense INTEGER,
            agility INTEGER,
            experience_reward INTEGER,
            gold_reward INTEGER,
            hp INTEGER DEFAULT 10,
            mp INTEGER DEFAULT 5,
            defeat_user_id INTEGER DEFAULT -1
        )",
        [],
    ) {
        eprintln!("Error create_monster_master_table: {:?}", e);
        return Err(e);
    }

    Ok(())
}

fn create_battle_results_table(conn: &Connection) -> Result<()> {
    if let Err(e) = conn.execute(
        "CREATE TABLE IF NOT EXISTS battle_results (
            battle_id INTEGER PRIMARY KEY AUTOINCREMENT,
            user_id INTEGER,
            monster_id INTEGER,
            victory BOOLEAN,
            experience_gain INTEGER,
            gold_gain INTEGER,
            battle_log TEXT,
            FOREIGN KEY (user_id) REFERENCES users (user_id),
            FOREIGN KEY (monster_id) REFERENCES monsters (monster_id)
        )",
        [],
    ) {
        eprintln!("Error create_battle_results_table: {:?}", e);
        return Err(e);
    }

    Ok(())
}

fn create_items_table(conn: &Connection) -> Result<()> {
    if let Err(e) = conn.execute(
        "CREATE TABLE IF NOT EXISTS items (
                item_id INTEGER PRIMARY KEY AUTOINCREMENT,
                name TEXT,
                type TEXT,
                attack_bonus INTEGER DEFAULT 0,
                defense_bonus INTEGER DEFAULT 0,
                agility_bonus INTEGER DEFAULT 0
            )",
        [],
    ) {
        eprintln!("Error create_items_table: {:?}", e);
        return Err(e);
    }

    Ok(())
}

pub fn connect() -> Result<Connection> {
    let conn = Connection::open("quest.db")?;

    let _ = create_user_table(&conn);
    let _ = create_monster_master_table(&conn);
    let _ = create_monster_table(&conn);
    let _ = create_battle_results_table(&conn);
    let _ = create_items_table(&conn);

    Ok(conn)
}
