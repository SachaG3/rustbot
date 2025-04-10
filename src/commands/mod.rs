use serenity::framework::standard::macros::group;
use serenity::framework::StandardFramework;
use serenity::prelude::*;
use serenity::model::prelude::*;
use serenity::framework::standard::CommandResult;
use serenity::model::channel::{Message, ReactionType};

use crate::database::{get_user_by_discord_id};

pub mod basic;
pub mod games;
pub mod profile;
pub mod spam;
pub mod valorant;

pub async fn wait_for_reaction(
    ctx: &Context,
    msg: &Message,
    user_id: UserId,
    reaction_type: ReactionType,
    timeout_seconds: u64,
) -> bool {
    let _message_id = msg.id;
    
    msg.react(&ctx.http, reaction_type.clone()).await.ok();
    
    // Version simplifiée sans collector
    let start_time = std::time::Instant::now();
    let timeout_duration = std::time::Duration::from_secs(timeout_seconds);
    
    while start_time.elapsed() < timeout_duration {
        // Vérifier les réactions toutes les secondes
        tokio::time::sleep(std::time::Duration::from_secs(1)).await;
        
        if let Ok(reactions) = msg.reaction_users(&ctx.http, reaction_type.clone(), None, None).await {
            if reactions.iter().any(|u| u.id == user_id) {
                return true;
            }
        }
    }
    
    false
}

pub async fn wait_for_message(
    ctx: &Context,
    channel_id: ChannelId,
    user_id: UserId,
    timeout_seconds: u64,
) -> Option<Message> {
    // Version simplifiée sans collector
    let start_time = std::time::Instant::now();
    let timeout_duration = std::time::Duration::from_secs(timeout_seconds);
    
    while start_time.elapsed() < timeout_duration {
        // Vérifier les messages toutes les secondes
        tokio::time::sleep(std::time::Duration::from_secs(2)).await;
        
        // Lire les derniers messages
        if let Ok(messages) = channel_id.messages(&ctx.http, |retriever| retriever.limit(10)).await {
            // Vérifier si un des messages récents est de l'utilisateur attendu
            for message in messages {
                if message.author.id == user_id && message.timestamp.timestamp() as u64 > 
                   (std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_secs() - timeout_seconds) {
                    return Some(message);
                }
            }
        }
    }
    
    None
}

pub async fn get_user_by_mention(
    ctx: &Context,
    msg: &Message,
    mention: &str,
) -> CommandResult<User> {
    let pool = {
        let data = ctx.data.read().await;
        data.get::<crate::database::DatabasePool>()
            .expect("Impossible d'obtenir le pool de la base de données")
            .clone()
    };

    let user_id = if mention.starts_with("<@") && mention.ends_with(">") {
        let id = mention.trim_start_matches("<@").trim_start_matches("!").trim_end_matches(">");
        match id.parse::<u64>() {
            Ok(id) => id,
            Err(_) => {
                msg.reply(&ctx.http, "ID d'utilisateur invalide.").await?;
                return Err("ID d'utilisateur invalide".into());
            }
        }
    } else {
        match mention.parse::<u64>() {
            Ok(id) => id,
            Err(_) => {
                msg.reply(&ctx.http, "ID d'utilisateur invalide.").await?;
                return Err("ID d'utilisateur invalide".into());
            }
        }
    };

    let user_data = get_user_by_discord_id(&pool, user_id).await?;
    
    match user_data {
        Some(_) => {
            match UserId(user_id).to_user(&ctx).await {
                Ok(user) => Ok(user),
                Err(_) => {
                    msg.reply(&ctx.http, "Impossible de trouver cet utilisateur.").await?;
                    Err("Utilisateur introuvable".into())
                }
            }
        }
        None => {
            msg.reply(&ctx.http, "Cet utilisateur n'est pas enregistré dans la base de données.").await?;
            Err("Utilisateur non enregistré".into())
        }
    }
} 