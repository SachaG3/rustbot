use serenity::framework::standard::{macros::command, CommandResult};
use serenity::model::prelude::*;
use serenity::prelude::*;
use tracing::error;

use crate::database::{DatabasePool, get_user_by_discord_id, new_user, add_daily_cat, get_daily_cat_count, has_daily_cat_today};

#[command]
#[description = "Gagne un Daily Cat"]
pub async fn cat(ctx: &Context, msg: &Message) -> CommandResult {
    let data = ctx.data.read().await;
    let pool = data.get::<DatabasePool>().expect("Impossible d'obtenir le pool");

    let user = match get_user_by_discord_id(&pool, msg.author.id.0).await {
        Ok(Some(u)) => u,
        Ok(None) => {
            let user_id = new_user(&pool, msg.author.id.0, &msg.author.name).await?;
            crate::database::User { id: user_id, id_utilisateur: msg.author.id.0.to_string(), pseudo: msg.author.name.clone(), score: 0 }
        },
        Err(_) => {
            msg.channel_id.say(&ctx.http, "Erreur avec la base de données").await.ok();
            return Ok(());
        }
    };

    if has_daily_cat_today(&pool, user.id).await.unwrap_or(false) {
        msg.channel_id.say(&ctx.http, "Tu as déjà obtenu ton chat quotidien, reviens demain").await.ok();
        return Ok(());
    }

    if add_daily_cat(&pool, user.id).await.is_err() {
        msg.channel_id.say(&ctx.http, "Impossible d'ajouter le Cat").await.ok();
        return Ok(());
    }

    let total = match get_daily_cat_count(&pool, user.id).await {
        Ok(c) => c,
        Err(_) => {
            msg.channel_id.say(&ctx.http, "Erreur avec la base de données").await.ok();
            return Ok(());
        }
    };

    msg.channel_id.say(&ctx.http, format!("Tu as gagné un 🐱 ! Total: {}", total)).await.ok();

    Ok(())
}

#[command]
#[description = "Affiche le nombre de Daily Cats"]
pub async fn cats(ctx: &Context, msg: &Message) -> CommandResult {
    let data = ctx.data.read().await;
    let pool = data.get::<DatabasePool>().expect("Impossible d'obtenir le pool");

    match get_user_by_discord_id(&pool, msg.author.id.0).await {
        Ok(Some(user)) => {
            match get_daily_cat_count(&pool, user.id).await {
                Ok(total) => {
                    msg.channel_id.say(&ctx.http, format!("Tu as {} 🐱.", total)).await.ok();
                },
                Err(_) => {
                    msg.channel_id.say(&ctx.http, "Erreur avec la base de données").await.ok();
                }
            }
        },
        Ok(None) => {
            msg.channel_id.say(&ctx.http, "Tu n'as pas encore de profil. Utilise ^^NP pour en créer un").await.ok();
        },
        Err(_) => {
            msg.channel_id.say(&ctx.http, "Erreur avec la base de données").await.ok();
        }
    }

    Ok(())
} 