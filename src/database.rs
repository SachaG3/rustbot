use serenity::prelude::*;
use sqlx::{MySql, Pool, Error, Row};
use rand::{thread_rng, Rng};
use rand::distributions::Alphanumeric;
use std::sync::Arc;

pub struct DatabasePool;

impl TypeMapKey for DatabasePool {
    type Value = Arc<Pool<MySql>>;
}

pub async fn add_log(pool: &Pool<MySql>, event_type: &str, description: &str) -> Result<(), Error> {
    sqlx::query("INSERT INTO logs (event_type, description) VALUES (?, ?)")
        .bind(event_type)
        .bind(description)
        .execute(pool)
        .await?;
    
    Ok(())
}

pub async fn get_user_by_discord_id(pool: &Pool<MySql>, discord_id: u64) -> Result<Option<User>, Error> {
    let discord_id_str = discord_id.to_string();
    let row = sqlx::query("SELECT id, id_utilisateur, pseudo, score FROM utilisateurs WHERE id_utilisateur = ?")
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
        },
        None => Ok(None),
    }
}

pub async fn new_user(pool: &Pool<MySql>, discord_id: u64, pseudo: &str) -> Result<i64, Error> {
    let discord_id_str = discord_id.to_string();
    let result = sqlx::query("INSERT INTO utilisateurs (id_utilisateur, pseudo, score) VALUES (?, ?, 0)")
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
        },
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

pub async fn new_message(pool: &Pool<MySql>, user_id: i64, content: &str, guild_id: u64) -> Result<i64, Error> {
    let result = sqlx::query("INSERT INTO message (userId, message, id_guild) VALUES (?, ?, ?)")
        .bind(user_id)
        .bind(content)
        .bind(guild_id as i64)
        .execute(pool)
        .await?;
    
    Ok(result.last_insert_id() as i64)
}

pub async fn new_message_delete(pool: &Pool<MySql>, user_id: i64, content: &str, guild_id: u64) -> Result<i64, Error> {
    let result = sqlx::query("INSERT INTO message_delete (userId, message, id_guild) VALUES (?, ?, ?)")
        .bind(user_id)
        .bind(content)
        .bind(guild_id as i64)
        .execute(pool)
        .await?;
    
    Ok(result.last_insert_id() as i64)
}

pub async fn new_message_edit(pool: &Pool<MySql>, user_id: i64, old_content: &str, new_content: &str, guild_id: u64) -> Result<i64, Error> {
    let result = sqlx::query("INSERT INTO message_edit (userId, message, new_message, id_guild) VALUES (?, ?, ?, ?)")
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

pub async fn add_user_to_guild(pool: &Pool<MySql>, user_id: i64, guild_id: u64) -> Result<i64, Error> {
    // Utilise INSERT IGNORE pour éviter les erreurs de doublons
    let result = sqlx::query("INSERT IGNORE INTO utilisateur_guilds (id_user, id_guild) VALUES (?, ?)")
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

pub async fn get_user_by_username(pool: &Pool<MySql>, username: &str) -> Result<Option<User>, Error> {
    let row = sqlx::query("SELECT id, id_utilisateur, pseudo, score FROM utilisateurs WHERE pseudo = ?")
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
        },
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

pub async fn add_daily_cat_with_date(pool: &Pool<MySql>, user_id: i64, date: &str) -> Result<i64, Error> {
    let result = sqlx::query("INSERT INTO daily_cats (user_id, created_at) VALUES (?, ?)")
        .bind(user_id)
        .bind(date)
        .execute(pool)
        .await?;

    Ok(result.last_insert_id() as i64)
}

pub async fn has_daily_cat_on_date(pool: &Pool<MySql>, user_id: i64, date: &str) -> Result<bool, Error> {
    let row = sqlx::query("SELECT 1 FROM daily_cats WHERE user_id = ? AND DATE(created_at) = ? LIMIT 1")
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

pub async fn debug_daily_cats(pool: &Pool<MySql>, user_id: i64) -> Result<Vec<String>, Error> {
    let rows = sqlx::query("SELECT created_at FROM daily_cats WHERE user_id = ? ORDER BY created_at DESC LIMIT 5")
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
    let row = sqlx::query("SELECT 1 FROM daily_cats WHERE user_id = ? AND DATE(created_at) = CURDATE() LIMIT 1")
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
    pub breed: String,
    pub color: String,
    pub age_months: i32,
    pub rarity_score: i32,
    pub obtained_at: chrono::NaiveDateTime,
}

pub async fn add_collected_cat(
    pool: &Pool<MySql>,
    user_id: i64,
    name: &str,
    breed: &str,
    color: &str,
    age_months: i32,
    rarity_score: i32
) -> Result<i32, Error> {
    let result = sqlx::query(
        "INSERT INTO collected_cats (user_id, name, breed, color, age_months, rarity_score, obtained_at) VALUES (?, ?, ?, ?, ?, ?, NOW())"
    )
        .bind(user_id as i32)
        .bind(name)
        .bind(breed)
        .bind(color)
        .bind(age_months)
        .bind(rarity_score)
        .execute(pool)
        .await?;

    Ok(result.last_insert_id() as i32)
}

pub async fn add_collected_cat_with_date(
    pool: &Pool<MySql>,
    user_id: i64,
    name: &str,
    breed: &str,
    color: &str,
    age_months: i32,
    rarity_score: i32,
    obtained_at: &str
) -> Result<i32, Error> {
    let result = sqlx::query(
        "INSERT INTO collected_cats (user_id, name, breed, color, age_months, rarity_score, obtained_at) VALUES (?, ?, ?, ?, ?, ?, ?)"
    )
        .bind(user_id as i32)
        .bind(name)
        .bind(breed)
        .bind(color)
        .bind(age_months)
        .bind(rarity_score)
        .bind(obtained_at)
        .execute(pool)
        .await?;

    Ok(result.last_insert_id() as i32)
}

pub async fn get_user_cats(pool: &Pool<MySql>, user_id: i64) -> Result<Vec<CollectedCat>, Error> {
    let rows = sqlx::query(
        "SELECT id, user_id, name, breed, color, age_months, rarity_score, obtained_at FROM collected_cats WHERE user_id = ? ORDER BY rarity_score DESC, obtained_at DESC"
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
            breed: row.get("breed"),
            color: row.get("color"),
            age_months: row.get("age_months"),
            rarity_score: row.get("rarity_score"),
            obtained_at: row.get("obtained_at"),
        });
    }
    
    Ok(cats)
}

pub async fn get_cat_by_id(pool: &Pool<MySql>, cat_id: i32) -> Result<Option<CollectedCat>, Error> {
    let row = sqlx::query(
        "SELECT id, user_id, name, breed, color, age_months, rarity_score, obtained_at FROM collected_cats WHERE id = ?"
    )
        .bind(cat_id)
        .fetch_optional(pool)
        .await?;
    
    match row {
        Some(row) => Ok(Some(CollectedCat {
            id: row.get("id"),
            user_id: row.get("user_id"),
            name: row.try_get("name").unwrap_or_else(|_| "Chat".to_string()), // Nom par défaut si NULL
            breed: row.get("breed"),
            color: row.get("color"),
            age_months: row.get("age_months"),
            rarity_score: row.get("rarity_score"),
            obtained_at: row.get("obtained_at"),
        })),
        None => Ok(None),
    }
}

pub async fn transfer_cat(pool: &Pool<MySql>, cat_id: i32, new_owner_id: i64) -> Result<(), Error> {
    sqlx::query("UPDATE collected_cats SET user_id = ? WHERE id = ?")
        .bind(new_owner_id as i32)
        .bind(cat_id)
        .execute(pool)
        .await?;
    
    Ok(())
}

pub async fn get_user_cat_count(pool: &Pool<MySql>, user_id: i64) -> Result<i64, Error> {
    let row = sqlx::query("SELECT COUNT(*) as cnt FROM collected_cats WHERE user_id = ?")
        .bind(user_id as i32)
        .fetch_one(pool)
        .await?;
    let cnt: i64 = row.get("cnt");
    Ok(cnt)
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