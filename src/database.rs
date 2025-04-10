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
    let result = sqlx::query("INSERT INTO utilisateur_guilds (id_user, id_guild) VALUES (?, ?)")
        .bind(user_id)
        .bind(guild_id as i64)
        .execute(pool)
        .await?;
    
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