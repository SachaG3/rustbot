use serenity::framework::standard::{macros::command, CommandResult};
use serenity::model::prelude::*;
use serenity::prelude::*;
use std::time::Duration;
use tokio::time::timeout;
use tracing::error;

/// Nombre maximal de répétitions pour `rp`, afin d'éviter les abus.
const MAX_REPEAT: usize = 20;
/// Nombre maximal de répétitions pour `rpt` (le TTS est plus sensible).
const MAX_REPEAT_TTS: usize = 10;

#[command]
#[description = "Répète un message plusieurs fois"]
pub async fn rp(ctx: &Context, msg: &Message) -> CommandResult {
    repeat_message(ctx, msg, false).await
}

#[command]
#[description = "Répète un message plusieurs fois avec TTS"]
pub async fn rpt(ctx: &Context, msg: &Message) -> CommandResult {
    repeat_message(ctx, msg, true).await
}

/// Logique commune à `rp` et `rpt` : demande le message et le nombre de
/// répétitions, puis le renvoie (avec ou sans TTS).
async fn repeat_message(ctx: &Context, msg: &Message, tts: bool) -> CommandResult {
    let Some(content_to_repeat) = ask(ctx, msg, "Choisis le mot que tu veux répéter").await else {
        return Ok(());
    };

    let Some(count_text) = ask(ctx, msg, "Choisis le nombre de fois que tu veux le répéter").await
    else {
        return Ok(());
    };

    let count: usize = match count_text.parse() {
        Ok(num) => num,
        Err(_) => {
            say(ctx, msg, "Veuillez entrer un nombre valide et réitérer la commande.").await;
            return Ok(());
        }
    };

    let safe_count = count.min(if tts { MAX_REPEAT_TTS } else { MAX_REPEAT });

    for _ in 0..safe_count {
        if let Err(why) = msg
            .channel_id
            .send_message(&ctx.http, |m| m.content(&content_to_repeat).tts(tts))
            .await
        {
            error!("Erreur lors de l'envoi du message répété: {:?}", why);
            break;
        }
    }

    Ok(())
}

/// Pose une question et renvoie le contenu de la réponse de l'auteur,
/// ou `None` (après l'avoir signalé) si l'envoi échoue ou si personne ne répond.
async fn ask(ctx: &Context, msg: &Message, question: &str) -> Option<String> {
    if let Err(why) = msg.channel_id.say(&ctx.http, question).await {
        error!("Erreur lors de l'envoi du message: {:?}", why);
        return None;
    }

    match get_user_response(ctx, msg, 30.0).await {
        Some(response) => Some(response.content),
        None => {
            say(ctx, msg, "Veuillez réitérer la commande.").await;
            None
        }
    }
}

async fn say(ctx: &Context, msg: &Message, text: &str) {
    if let Err(why) = msg.channel_id.say(&ctx.http, text).await {
        error!("Erreur lors de l'envoi du message: {:?}", why);
    }
}

async fn get_user_response(ctx: &Context, msg: &Message, timeout_seconds: f32) -> Option<Message> {
    timeout(
        Duration::from_secs_f32(timeout_seconds),
        crate::commands::wait_for_message(ctx, msg.channel_id, msg.author.id, 600),
    )
    .await
    .unwrap_or_default()
}
