use serenity::framework::standard::{macros::command, CommandResult};
use serenity::model::prelude::*;
use serenity::prelude::*;
use tracing::error;
use std::time::Duration;
use tokio::time::timeout;

#[command]
#[description = "Répète un message plusieurs fois"]
pub async fn rp(ctx: &Context, msg: &Message) -> CommandResult {
    if let Err(why) = msg.channel_id.say(&ctx.http, "Choisis le mot que tu veux répéter").await {
        error!("Erreur lors de l'envoi du message: {:?}", why);
        return Ok(());
    }
    
    let user_msg = match get_user_response(ctx, msg, 30.0).await {
        Some(response) => response,
        None => {
            if let Err(why) = msg.channel_id.say(&ctx.http, "Veuillez réitérer la commande.").await {
                error!("Erreur lors de l'envoi du message: {:?}", why);
            }
            return Ok(());
        }
    };
    
    let content_to_repeat = user_msg.content.clone();
    
    if let Err(why) = msg.channel_id.say(&ctx.http, "Choisis le nombre de fois que tu veux le répéter").await {
        error!("Erreur lors de l'envoi du message: {:?}", why);
        return Ok(());
    }
    
    let count_msg = match get_user_response(ctx, msg, 30.0).await {
        Some(response) => response,
        None => {
            if let Err(why) = msg.channel_id.say(&ctx.http, "Veuillez réitérer la commande.").await {
                error!("Erreur lors de l'envoi du message: {:?}", why);
            }
            return Ok(());
        }
    };
    
    let count: usize = match count_msg.content.parse() {
        Ok(num) => num,
        Err(_) => {
            if let Err(why) = msg.channel_id.say(&ctx.http, "Veuillez entrer un nombre valide et réitérer la commande.").await {
                error!("Erreur lors de l'envoi du message: {:?}", why);
            }
            return Ok(());
        }
    };
    
    // Limiter le nombre de répétitions à 20 pour éviter les abus
    let safe_count = count.min(20);
    
    for _ in 0..safe_count {
        if let Err(why) = msg.channel_id.say(&ctx.http, &content_to_repeat).await {
            error!("Erreur lors de l'envoi du message répété: {:?}", why);
            break;
        }
    }
    
    Ok(())
}

#[command]
#[description = "Répète un message plusieurs fois avec TTS"]
pub async fn rpt(ctx: &Context, msg: &Message) -> CommandResult {
    if let Err(why) = msg.channel_id.say(&ctx.http, "Choisis le mot que tu veux répéter").await {
        error!("Erreur lors de l'envoi du message: {:?}", why);
        return Ok(());
    }
    
    let user_msg = match get_user_response(ctx, msg, 30.0).await {
        Some(response) => response,
        None => {
            if let Err(why) = msg.channel_id.say(&ctx.http, "Veuillez réitérer la commande.").await {
                error!("Erreur lors de l'envoi du message: {:?}", why);
            }
            return Ok(());
        }
    };
    
    let content_to_repeat = user_msg.content.clone();
    
    if let Err(why) = msg.channel_id.say(&ctx.http, "Choisis le nombre de fois que tu veux le répéter").await {
        error!("Erreur lors de l'envoi du message: {:?}", why);
        return Ok(());
    }
    
    let count_msg = match get_user_response(ctx, msg, 30.0).await {
        Some(response) => response,
        None => {
            if let Err(why) = msg.channel_id.say(&ctx.http, "Veuillez réitérer la commande.").await {
                error!("Erreur lors de l'envoi du message: {:?}", why);
            }
            return Ok(());
        }
    };
    
    let count: usize = match count_msg.content.parse() {
        Ok(num) => num,
        Err(_) => {
            if let Err(why) = msg.channel_id.say(&ctx.http, "Veuillez entrer un nombre valide et réitérer la commande.").await {
                error!("Erreur lors de l'envoi du message: {:?}", why);
            }
            return Ok(());
        }
    };
    
    // Limiter le nombre de répétitions à 10 pour éviter les abus (TTS est plus sensible)
    let safe_count = count.min(10);
    
    for _ in 0..safe_count {
        if let Err(why) = msg.channel_id.send_message(&ctx.http, |m| {
            m.content(&content_to_repeat).tts(true)
        }).await {
            error!("Erreur lors de l'envoi du message TTS répété: {:?}", why);
            break;
        }
    }
    
    Ok(())
}

async fn get_user_response(ctx: &Context, msg: &Message, timeout_seconds: f32) -> Option<Message> {
    let author_id = msg.author.id;
    let channel_id = msg.channel_id;
    
    // Collecteur pour attendre le message
    let fut = timeout(
        Duration::from_secs_f32(timeout_seconds),
        collect_message(ctx, author_id, channel_id),
    ).await;
    
    match fut {
        Ok(message) => message,
        Err(_) => None,
    }
}

async fn collect_message(ctx: &Context, author_id: UserId, channel_id: ChannelId) -> Option<Message> {
    // Utiliser wait_for_message du module commands à la place
    crate::commands::wait_for_message(ctx, channel_id, author_id, 600).await
} 