use rand::distributions::Alphanumeric;
use rand::{thread_rng, Rng};
use serenity::prelude::*;
use sqlx::{Error, MySql, Pool, Row};
use std::sync::Arc;

pub struct DatabasePool;

impl TypeMapKey for DatabasePool {
    type Value = Arc<Pool<MySql>>;
}

pub async fn ensure_cat_schema(pool: &Pool<MySql>) -> Result<(), Error> {
    let statements = [
        "ALTER TABLE collected_cats ADD COLUMN IF NOT EXISTS nickname varchar(80) CHARACTER SET utf8mb4 COLLATE utf8mb4_unicode_ci DEFAULT NULL",
        "ALTER TABLE collected_cats ADD COLUMN IF NOT EXISTS personality varchar(80) CHARACTER SET utf8mb4 COLLATE utf8mb4_unicode_ci DEFAULT NULL",
        "ALTER TABLE collected_cats ADD COLUMN IF NOT EXISTS mood varchar(120) CHARACTER SET utf8mb4 COLLATE utf8mb4_unicode_ci DEFAULT NULL",
        "ALTER TABLE collected_cats ADD COLUMN IF NOT EXISTS location varchar(20) NOT NULL DEFAULT 'home'",
        "ALTER TABLE collected_cats ADD COLUMN IF NOT EXISTS original_owner_id int(11) DEFAULT NULL",
        "ALTER TABLE collected_cats ADD COLUMN IF NOT EXISTS refuge_by_user_id int(11) DEFAULT NULL",
        "ALTER TABLE collected_cats ADD COLUMN IF NOT EXISTS moved_to_refuge_at datetime DEFAULT NULL",
        "ALTER TABLE collected_cats ADD COLUMN IF NOT EXISTS is_favorite tinyint(1) NOT NULL DEFAULT 0",
        "ALTER TABLE collected_cats MODIFY nickname varchar(80) CHARACTER SET utf8mb4 COLLATE utf8mb4_unicode_ci DEFAULT NULL",
        "ALTER TABLE collected_cats MODIFY personality varchar(80) CHARACTER SET utf8mb4 COLLATE utf8mb4_unicode_ci DEFAULT NULL",
        "ALTER TABLE collected_cats MODIFY mood varchar(120) CHARACTER SET utf8mb4 COLLATE utf8mb4_unicode_ci DEFAULT NULL",
        "UPDATE collected_cats SET original_owner_id = user_id WHERE original_owner_id IS NULL",
        "UPDATE collected_cats SET location = 'home' WHERE location IS NULL OR location = ''",
        "CREATE TABLE IF NOT EXISTS cat_memories (
            id int(11) NOT NULL AUTO_INCREMENT,
            cat_id int(11) NOT NULL,
            user_id int(11) DEFAULT NULL,
            memory_type varchar(50) NOT NULL,
            description text NOT NULL,
            created_at datetime NOT NULL DEFAULT current_timestamp(),
            PRIMARY KEY (id),
            KEY idx_cat_id (cat_id),
            KEY idx_user_id (user_id)
        ) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_unicode_ci",
        "CREATE TABLE IF NOT EXISTS cat_event_history (
            id int(11) NOT NULL AUTO_INCREMENT,
            channel_id bigint(20) UNSIGNED NOT NULL,
            event_kind varchar(30) NOT NULL,
            theme varchar(80) DEFAULT NULL,
            started_at datetime NOT NULL DEFAULT current_timestamp(),
            PRIMARY KEY (id),
            KEY idx_started_at (started_at)
        ) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_unicode_ci",
    ];

    for statement in statements {
        sqlx::query(statement).execute(pool).await?;
    }

    Ok(())
}

pub async fn add_log(pool: &Pool<MySql>, event_type: &str, description: &str) -> Result<(), Error> {
    sqlx::query("INSERT INTO logs (event_type, description) VALUES (?, ?)")
        .bind(event_type)
        .bind(description)
        .execute(pool)
        .await?;

    Ok(())
}

pub async fn get_user_by_discord_id(
    pool: &Pool<MySql>,
    discord_id: u64,
) -> Result<Option<User>, Error> {
    let discord_id_str = discord_id.to_string();
    let row = sqlx::query(
        "SELECT id, id_utilisateur, pseudo, score FROM utilisateurs WHERE id_utilisateur = ?",
    )
    .bind(&discord_id_str)
    .fetch_optional(pool)
    .await?;

    match row {
        Some(row) => {
            let id: i64 = row.get("id");
            let idutil: String = row.get("id_utilisateur");
            let pseudo: String = row.get("pseudo");
            let score: i32 = row.get("score");

            Ok(Some(User {
                id,
                id_utilisateur: idutil,
                pseudo,
                score,
            }))
        }
        None => Ok(None),
    }
}

pub async fn new_user(pool: &Pool<MySql>, discord_id: u64, pseudo: &str) -> Result<i64, Error> {
    let discord_id_str = discord_id.to_string();
    let result =
        sqlx::query("INSERT INTO utilisateurs (id_utilisateur, pseudo, score) VALUES (?, ?, 0)")
            .bind(&discord_id_str)
            .bind(pseudo)
            .execute(pool)
            .await?;

    Ok(result.last_insert_id() as i64)
}

pub async fn get_guild(pool: &Pool<MySql>, guild_id: u64) -> Result<Option<Guild>, Error> {
    let row = sqlx::query("SELECT id, name FROM guilds WHERE id = ?")
        .bind(guild_id as i64)
        .fetch_optional(pool)
        .await?;

    match row {
        Some(row) => {
            let id: i64 = row.get("id");
            let name: String = row.get("name");

            Ok(Some(Guild { id, name }))
        }
        None => Ok(None),
    }
}

pub async fn add_guild(pool: &Pool<MySql>, guild_id: u64, name: &str) -> Result<i64, Error> {
    let result = sqlx::query("INSERT INTO guilds (id, name) VALUES (?, ?)")
        .bind(guild_id as i64)
        .bind(name)
        .execute(pool)
        .await?;

    Ok(result.last_insert_id() as i64)
}

pub async fn new_message(
    pool: &Pool<MySql>,
    user_id: i64,
    content: &str,
    guild_id: u64,
) -> Result<i64, Error> {
    let result = sqlx::query("INSERT INTO message (userId, message, id_guild) VALUES (?, ?, ?)")
        .bind(user_id)
        .bind(content)
        .bind(guild_id as i64)
        .execute(pool)
        .await?;

    Ok(result.last_insert_id() as i64)
}

pub async fn new_message_delete(
    pool: &Pool<MySql>,
    user_id: i64,
    content: &str,
    guild_id: u64,
) -> Result<i64, Error> {
    let result =
        sqlx::query("INSERT INTO message_delete (userId, message, id_guild) VALUES (?, ?, ?)")
            .bind(user_id)
            .bind(content)
            .bind(guild_id as i64)
            .execute(pool)
            .await?;

    Ok(result.last_insert_id() as i64)
}

pub async fn new_message_edit(
    pool: &Pool<MySql>,
    user_id: i64,
    old_content: &str,
    new_content: &str,
    guild_id: u64,
) -> Result<i64, Error> {
    let result = sqlx::query(
        "INSERT INTO message_edit (userId, message, new_message, id_guild) VALUES (?, ?, ?, ?)",
    )
    .bind(user_id)
    .bind(old_content)
    .bind(new_content)
    .bind(guild_id as i64)
    .execute(pool)
    .await?;

    Ok(result.last_insert_id() as i64)
}

pub async fn new_token(pool: &Pool<MySql>, user_id: i64) -> Result<String, Error> {
    let token: String = thread_rng()
        .sample_iter(&Alphanumeric)
        .take(32)
        .map(char::from)
        .collect();

    sqlx::query("INSERT INTO tokens (userId, token) VALUES (?, ?)")
        .bind(user_id)
        .bind(&token)
        .execute(pool)
        .await?;

    Ok(token)
}

pub async fn add_user_to_guild(
    pool: &Pool<MySql>,
    user_id: i64,
    guild_id: u64,
) -> Result<i64, Error> {
    // Utilise INSERT IGNORE pour éviter les erreurs de doublons
    let result =
        sqlx::query("INSERT IGNORE INTO utilisateur_guilds (id_user, id_guild) VALUES (?, ?)")
            .bind(user_id)
            .bind(guild_id as i64)
            .execute(pool)
            .await?;

    // Si aucune ligne n'a été insérée (doublon), on retourne 0
    // Sinon on retourne l'ID de la nouvelle ligne
    Ok(result.last_insert_id() as i64)
}

pub async fn update_user_score(pool: &Pool<MySql>, user_id: i64, points: i32) -> Result<(), Error> {
    sqlx::query("UPDATE utilisateurs SET score = score + ? WHERE id = ?")
        .bind(points)
        .bind(user_id)
        .execute(pool)
        .await?;

    Ok(())
}

pub async fn get_user_score(pool: &Pool<MySql>, user_id: i64) -> Result<i32, Error> {
    let row = sqlx::query("SELECT score FROM utilisateurs WHERE id = ?")
        .bind(user_id)
        .fetch_one(pool)
        .await?;

    let score: i32 = row.get("score");

    Ok(score)
}

pub async fn get_user_by_username(
    pool: &Pool<MySql>,
    username: &str,
) -> Result<Option<User>, Error> {
    let row =
        sqlx::query("SELECT id, id_utilisateur, pseudo, score FROM utilisateurs WHERE pseudo = ?")
            .bind(username)
            .fetch_optional(pool)
            .await?;

    match row {
        Some(row) => {
            let id: i64 = row.get("id");
            let idutil: String = row.get("id_utilisateur");
            let pseudo: String = row.get("pseudo");
            let score: i32 = row.get("score");

            Ok(Some(User {
                id,
                id_utilisateur: idutil,
                pseudo,
                score,
            }))
        }
        None => Ok(None),
    }
}

pub async fn add_daily_cat(pool: &Pool<MySql>, user_id: i64) -> Result<i64, Error> {
    let result = sqlx::query("INSERT INTO daily_cats (user_id, created_at) VALUES (?, NOW())")
        .bind(user_id)
        .execute(pool)
        .await?;

    Ok(result.last_insert_id() as i64)
}

pub async fn add_daily_cat_with_date(
    pool: &Pool<MySql>,
    user_id: i64,
    date: &str,
) -> Result<i64, Error> {
    let result = sqlx::query("INSERT INTO daily_cats (user_id, created_at) VALUES (?, ?)")
        .bind(user_id)
        .bind(date)
        .execute(pool)
        .await?;

    Ok(result.last_insert_id() as i64)
}

pub async fn has_daily_cat_on_date(
    pool: &Pool<MySql>,
    user_id: i64,
    date: &str,
) -> Result<bool, Error> {
    let row =
        sqlx::query("SELECT 1 FROM daily_cats WHERE user_id = ? AND DATE(created_at) = ? LIMIT 1")
            .bind(user_id)
            .bind(date)
            .fetch_optional(pool)
            .await?;
    Ok(row.is_some())
}

pub async fn get_daily_cat_count(pool: &Pool<MySql>, user_id: i64) -> Result<i64, Error> {
    let row = sqlx::query("SELECT COUNT(*) as cnt FROM daily_cats WHERE user_id = ?")
        .bind(user_id)
        .fetch_one(pool)
        .await?;
    let cnt: i64 = row.get("cnt");
    Ok(cnt)
}

pub async fn get_daily_cat_count_today(pool: &Pool<MySql>) -> Result<i64, Error> {
    let row = sqlx::query("SELECT COUNT(*) as cnt FROM daily_cats WHERE DATE(created_at) = CURDATE()")
        .fetch_one(pool)
        .await?;
    Ok(row.get("cnt"))
}

pub async fn debug_daily_cats(pool: &Pool<MySql>, user_id: i64) -> Result<Vec<String>, Error> {
    let rows = sqlx::query(
        "SELECT created_at FROM daily_cats WHERE user_id = ? ORDER BY created_at DESC LIMIT 5",
    )
    .bind(user_id)
    .fetch_all(pool)
    .await?;

    let mut dates = Vec::new();
    for row in rows {
        // Utiliser le bon type selon ta structure de table (TIMESTAMP)
        let date: chrono::DateTime<chrono::Utc> = row.get("created_at");
        dates.push(date.format("%Y-%m-%d %H:%M:%S").to_string());
    }
    Ok(dates)
}

pub async fn has_daily_cat_today(pool: &Pool<MySql>, user_id: i64) -> Result<bool, Error> {
    let row = sqlx::query(
        "SELECT 1 FROM daily_cats WHERE user_id = ? AND DATE(created_at) = CURDATE() LIMIT 1",
    )
    .bind(user_id)
    .fetch_optional(pool)
    .await?;
    Ok(row.is_some())
}

#[derive(Debug, Clone)]
pub struct CollectedCat {
    pub id: i32,
    pub user_id: i32,
    pub name: String,
    pub nickname: Option<String>,
    pub breed: String,
    pub color: String,
    pub age_months: i32,
    pub rarity_score: i32,
    pub obtained_at: chrono::NaiveDateTime,
    pub personality: Option<String>,
    pub mood: Option<String>,
    pub location: String,
    pub original_owner_id: Option<i32>,
    pub refuge_by_user_id: Option<i32>,
    pub is_favorite: bool,
}

#[derive(Debug, Clone)]
pub struct CatCounts {
    pub home: i64,
    pub refuge: i64,
    pub total: i64,
}

#[derive(Debug, Clone)]
pub struct CatMemory {
    pub description: String,
    pub created_at: chrono::NaiveDateTime,
}

pub async fn add_collected_cat(
    pool: &Pool<MySql>,
    user_id: i64,
    name: &str,
    breed: &str,
    color: &str,
    age_months: i32,
    rarity_score: i32,
) -> Result<i32, Error> {
    let personality = random_cat_personality();
    let mood = random_cat_mood();
    let result = sqlx::query(
        "INSERT INTO collected_cats (user_id, name, breed, color, age_months, rarity_score, obtained_at, personality, mood, location, original_owner_id) VALUES (?, ?, ?, ?, ?, ?, NOW(), ?, ?, 'home', ?)"
    )
        .bind(user_id as i32)
        .bind(name)
        .bind(breed)
        .bind(color)
        .bind(age_months)
        .bind(rarity_score)
        .bind(personality)
        .bind(mood)
        .bind(user_id as i32)
        .execute(pool)
        .await?;

    let cat_id = result.last_insert_id() as i32;
    add_cat_memory(
        pool,
        cat_id,
        Some(user_id),
        "arrival",
        &format!("{} a rejoint son foyer.", name),
    )
    .await
    .ok();

    Ok(cat_id)
}

pub async fn add_collected_cat_with_date(
    pool: &Pool<MySql>,
    user_id: i64,
    name: &str,
    breed: &str,
    color: &str,
    age_months: i32,
    rarity_score: i32,
    obtained_at: &str,
) -> Result<i32, Error> {
    let personality = random_cat_personality();
    let mood = random_cat_mood();
    let result = sqlx::query(
        "INSERT INTO collected_cats (user_id, name, breed, color, age_months, rarity_score, obtained_at, personality, mood, location, original_owner_id) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, 'home', ?)"
    )
        .bind(user_id as i32)
        .bind(name)
        .bind(breed)
        .bind(color)
        .bind(age_months)
        .bind(rarity_score)
        .bind(obtained_at)
        .bind(personality)
        .bind(mood)
        .bind(user_id as i32)
        .execute(pool)
        .await?;

    let cat_id = result.last_insert_id() as i32;
    add_cat_memory(
        pool,
        cat_id,
        Some(user_id),
        "arrival",
        &format!("{} a rejoint son foyer.", name),
    )
    .await
    .ok();

    Ok(cat_id)
}

pub async fn get_user_cats(pool: &Pool<MySql>, user_id: i64) -> Result<Vec<CollectedCat>, Error> {
    let rows = sqlx::query(
        "SELECT id, user_id, name, nickname, breed, color, age_months, rarity_score, obtained_at, personality, mood, location, original_owner_id, refuge_by_user_id, is_favorite FROM collected_cats WHERE user_id = ? AND location = 'home' ORDER BY is_favorite DESC, rarity_score DESC, obtained_at DESC"
    )
        .bind(user_id as i32)
        .fetch_all(pool)
        .await?;

    let mut cats = Vec::new();
    for row in rows {
        cats.push(CollectedCat {
            id: row.get("id"),
            user_id: row.get("user_id"),
            name: row.try_get("name").unwrap_or_else(|_| "Chat".to_string()), // Nom par défaut si NULL
            nickname: row.try_get::<Option<String>, _>("nickname").unwrap_or(None),
            breed: row.get("breed"),
            color: row.get("color"),
            age_months: row.get("age_months"),
            rarity_score: row.get("rarity_score"),
            obtained_at: row.get("obtained_at"),
            personality: row
                .try_get::<Option<String>, _>("personality")
                .unwrap_or(None),
            mood: row.try_get::<Option<String>, _>("mood").unwrap_or(None),
            location: row
                .try_get("location")
                .unwrap_or_else(|_| "home".to_string()),
            original_owner_id: row
                .try_get::<Option<i32>, _>("original_owner_id")
                .unwrap_or(None),
            refuge_by_user_id: row
                .try_get::<Option<i32>, _>("refuge_by_user_id")
                .unwrap_or(None),
            is_favorite: row
                .try_get::<i8, _>("is_favorite")
                .map(|value| value != 0)
                .unwrap_or(false),
        });
    }

    Ok(cats)
}

pub async fn get_cat_by_id(pool: &Pool<MySql>, cat_id: i32) -> Result<Option<CollectedCat>, Error> {
    let row = sqlx::query(
        "SELECT id, user_id, name, nickname, breed, color, age_months, rarity_score, obtained_at, personality, mood, location, original_owner_id, refuge_by_user_id, is_favorite FROM collected_cats WHERE id = ?"
    )
        .bind(cat_id)
        .fetch_optional(pool)
        .await?;

    match row {
        Some(row) => Ok(Some(CollectedCat {
            id: row.get("id"),
            user_id: row.get("user_id"),
            name: row.try_get("name").unwrap_or_else(|_| "Chat".to_string()), // Nom par défaut si NULL
            nickname: row.try_get::<Option<String>, _>("nickname").unwrap_or(None),
            breed: row.get("breed"),
            color: row.get("color"),
            age_months: row.get("age_months"),
            rarity_score: row.get("rarity_score"),
            obtained_at: row.get("obtained_at"),
            personality: row
                .try_get::<Option<String>, _>("personality")
                .unwrap_or(None),
            mood: row.try_get::<Option<String>, _>("mood").unwrap_or(None),
            location: row
                .try_get("location")
                .unwrap_or_else(|_| "home".to_string()),
            original_owner_id: row
                .try_get::<Option<i32>, _>("original_owner_id")
                .unwrap_or(None),
            refuge_by_user_id: row
                .try_get::<Option<i32>, _>("refuge_by_user_id")
                .unwrap_or(None),
            is_favorite: row
                .try_get::<i8, _>("is_favorite")
                .map(|value| value != 0)
                .unwrap_or(false),
        })),
        None => Ok(None),
    }
}

pub async fn transfer_cat(pool: &Pool<MySql>, cat_id: i32, new_owner_id: i64) -> Result<(), Error> {
    sqlx::query("UPDATE collected_cats SET user_id = ?, location = 'home', refuge_by_user_id = NULL, moved_to_refuge_at = NULL, is_favorite = 0 WHERE id = ?")
        .bind(new_owner_id as i32)
        .bind(cat_id)
        .execute(pool)
        .await?;

    Ok(())
}

pub async fn get_user_cat_count(pool: &Pool<MySql>, user_id: i64) -> Result<i64, Error> {
    let row = sqlx::query(
        "SELECT COUNT(*) as cnt FROM collected_cats WHERE user_id = ? AND location = 'home'",
    )
    .bind(user_id as i32)
    .fetch_one(pool)
    .await?;
    let cnt: i64 = row.get("cnt");
    Ok(cnt)
}

pub async fn get_user_refuge_cat_count(pool: &Pool<MySql>, user_id: i64) -> Result<i64, Error> {
    let row = sqlx::query("SELECT COUNT(*) as cnt FROM collected_cats WHERE location = 'refuge' AND refuge_by_user_id = ?")
        .bind(user_id as i32)
        .fetch_one(pool)
        .await?;
    Ok(row.get("cnt"))
}

pub async fn get_user_cat_counts(pool: &Pool<MySql>, user_id: i64) -> Result<CatCounts, Error> {
    let home = get_user_cat_count(pool, user_id).await?;
    let refuge = get_user_refuge_cat_count(pool, user_id).await?;
    Ok(CatCounts {
        home,
        refuge,
        total: home + refuge,
    })
}

pub async fn get_refuge_cats(pool: &Pool<MySql>, limit: i64) -> Result<Vec<CollectedCat>, Error> {
    let rows = sqlx::query(
        "SELECT id, user_id, name, nickname, breed, color, age_months, rarity_score, obtained_at, personality, mood, location, original_owner_id, refuge_by_user_id, is_favorite FROM collected_cats WHERE location = 'refuge' ORDER BY moved_to_refuge_at DESC, rarity_score DESC LIMIT ?"
    )
        .bind(limit)
        .fetch_all(pool)
        .await?;

    rows.into_iter().map(row_to_collected_cat).collect()
}

pub async fn move_cat_to_refuge(
    pool: &Pool<MySql>,
    cat_id: i32,
    user_id: i64,
) -> Result<(), Error> {
    sqlx::query("UPDATE collected_cats SET location = 'refuge', refuge_by_user_id = ?, moved_to_refuge_at = NOW(), is_favorite = 0 WHERE id = ? AND user_id = ? AND location = 'home'")
        .bind(user_id as i32)
        .bind(cat_id)
        .bind(user_id as i32)
        .execute(pool)
        .await?;

    add_cat_memory(
        pool,
        cat_id,
        Some(user_id),
        "refuge",
        "Ce chat a été confié au refuge.",
    )
    .await
    .ok();
    Ok(())
}

pub async fn set_cat_nickname(
    pool: &Pool<MySql>,
    cat_id: i32,
    user_id: i64,
    nickname: Option<&str>,
) -> Result<(), Error> {
    sqlx::query(
        "UPDATE collected_cats SET nickname = ? WHERE id = ? AND user_id = ? AND location = 'home'",
    )
    .bind(nickname)
    .bind(cat_id)
    .bind(user_id as i32)
    .execute(pool)
    .await?;
    Ok(())
}

pub async fn set_favorite_cat(pool: &Pool<MySql>, user_id: i64, cat_id: i32) -> Result<(), Error> {
    let mut tx = pool.begin().await?;

    sqlx::query("UPDATE collected_cats SET is_favorite = 0 WHERE user_id = ?")
        .bind(user_id as i32)
        .execute(&mut *tx)
        .await?;

    sqlx::query("UPDATE collected_cats SET is_favorite = 1 WHERE id = ? AND user_id = ? AND location = 'home'")
        .bind(cat_id)
        .bind(user_id as i32)
        .execute(&mut *tx)
        .await?;

    tx.commit().await?;
    Ok(())
}

pub async fn get_cat_memories(
    pool: &Pool<MySql>,
    cat_id: i32,
    limit: i64,
) -> Result<Vec<CatMemory>, Error> {
    let rows = sqlx::query("SELECT description, created_at FROM cat_memories WHERE cat_id = ? ORDER BY created_at DESC LIMIT ?")
        .bind(cat_id)
        .bind(limit)
        .fetch_all(pool)
        .await?;

    Ok(rows
        .into_iter()
        .map(|row| CatMemory {
            description: row.get("description"),
            created_at: row.get("created_at"),
        })
        .collect())
}

pub async fn add_cat_memory(
    pool: &Pool<MySql>,
    cat_id: i32,
    user_id: Option<i64>,
    memory_type: &str,
    description: &str,
) -> Result<(), Error> {
    sqlx::query(
        "INSERT INTO cat_memories (cat_id, user_id, memory_type, description) VALUES (?, ?, ?, ?)",
    )
    .bind(cat_id)
    .bind(user_id.map(|id| id as i32))
    .bind(memory_type)
    .bind(description)
    .execute(pool)
    .await?;
    Ok(())
}

pub async fn give_refuge_cat_to_user(
    pool: &Pool<MySql>,
    cat_id: i32,
    user_id: i64,
) -> Result<(), Error> {
    sqlx::query("UPDATE collected_cats SET user_id = ?, location = 'home', moved_to_refuge_at = NULL, is_favorite = 0 WHERE id = ? AND location = 'refuge'")
        .bind(user_id as i32)
        .bind(cat_id)
        .execute(pool)
        .await?;

    add_cat_memory(
        pool,
        cat_id,
        Some(user_id),
        "adoption",
        "Ce chat a quitte le refuge pour rejoindre une nouvelle maison.",
    )
    .await
    .ok();
    Ok(())
}

pub async fn get_cat_server_stats(pool: &Pool<MySql>) -> Result<(i64, i64, i64), Error> {
    let daily_row = sqlx::query("SELECT COUNT(*) as cnt FROM daily_cats")
        .fetch_one(pool)
        .await?;
    let home_row =
        sqlx::query("SELECT COUNT(*) as cnt FROM collected_cats WHERE location = 'home'")
            .fetch_one(pool)
            .await?;
    let refuge_row =
        sqlx::query("SELECT COUNT(*) as cnt FROM collected_cats WHERE location = 'refuge'")
            .fetch_one(pool)
            .await?;

    Ok((
        daily_row.get("cnt"),
        home_row.get("cnt"),
        refuge_row.get("cnt"),
    ))
}

pub async fn get_cat_event_count_last_7_days(pool: &Pool<MySql>) -> Result<i64, Error> {
    let row = sqlx::query("SELECT COUNT(*) as cnt FROM cat_event_history WHERE started_at >= DATE_SUB(NOW(), INTERVAL 7 DAY)")
        .fetch_one(pool)
        .await?;
    Ok(row.get("cnt"))
}

pub async fn record_cat_event_start(
    pool: &Pool<MySql>,
    channel_id: u64,
    event_kind: &str,
    theme: Option<&str>,
) -> Result<(), Error> {
    sqlx::query("INSERT INTO cat_event_history (channel_id, event_kind, theme) VALUES (?, ?, ?)")
        .bind(channel_id)
        .bind(event_kind)
        .bind(theme)
        .execute(pool)
        .await?;
    Ok(())
}

fn row_to_collected_cat(row: sqlx::mysql::MySqlRow) -> Result<CollectedCat, Error> {
    Ok(CollectedCat {
        id: row.get("id"),
        user_id: row.get("user_id"),
        name: row.try_get("name").unwrap_or_else(|_| "Chat".to_string()),
        nickname: row.try_get::<Option<String>, _>("nickname").unwrap_or(None),
        breed: row.get("breed"),
        color: row.get("color"),
        age_months: row.get("age_months"),
        rarity_score: row.get("rarity_score"),
        obtained_at: row.get("obtained_at"),
        personality: row
            .try_get::<Option<String>, _>("personality")
            .unwrap_or(None),
        mood: row.try_get::<Option<String>, _>("mood").unwrap_or(None),
        location: row
            .try_get("location")
            .unwrap_or_else(|_| "home".to_string()),
        original_owner_id: row
            .try_get::<Option<i32>, _>("original_owner_id")
            .unwrap_or(None),
        refuge_by_user_id: row
            .try_get::<Option<i32>, _>("refuge_by_user_id")
            .unwrap_or(None),
        is_favorite: row
            .try_get::<i8, _>("is_favorite")
            .map(|value| value != 0)
            .unwrap_or(false),
    })
}

fn random_cat_personality() -> &'static str {
    let personalities = [
        "calme",
        "curieux",
        "digne",
        "pot de colle",
        "timide",
        "joueur",
        "dramatique",
        "gourmand",
        "independant",
        "protecteur",
    ];
    personalities[thread_rng().gen_range(0..personalities.len())]
}

fn random_cat_mood() -> &'static str {
    let moods = [
        "dort dans un rayon de soleil",
        "surveille la fenetre",
        "inspecte une boite vide",
        "reclame des calins",
        "boude sans explication",
        "patrouille dans la maison",
        "s'etire avec beaucoup de dignite",
        "attend devant une gamelle vide",
        "observe tout le monde en silence",
        "a choisi le meilleur coussin",
        "joue avec une chaussette abandonnee",
        "fait semblant de ne pas entendre son nom",
        "se cache derriere un rideau",
        "regarde fixement un coin vide",
        "fait tomber un petit objet puis part",
        "se roule sur le tapis",
        "surveille les nouveaux residents",
        "s'endort a moitie assis",
        "chasse une ombre au sol",
        "renifle une tasse avec beaucoup de serieux",
        "s'installe la ou il gene le plus",
        "fait sa toilette avec une concentration totale",
        "observe la pluie contre la vitre",
        "vient saluer puis repart aussitot",
        "semble preparer une betise",
        "fait une course soudaine dans le couloir",
        "dort avec une patte sur les yeux",
        "regarde dehors comme s'il attendait quelqu'un",
        "ronronne discretement pres du canape",
        "a choisi une couverture et refuse de la partager",
    ];
    moods[thread_rng().gen_range(0..moods.len())]
}

#[derive(Debug)]
pub struct User {
    pub id: i64,
    pub id_utilisateur: String,
    pub pseudo: String,
    pub score: i32,
}

#[derive(Debug)]
pub struct Guild {
    pub id: i64,
    pub name: String,
}
