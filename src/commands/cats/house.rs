//! Maison, refuge et personnalisation des chats.

use serenity::framework::standard::{macros::command, Args, CommandResult};
use serenity::model::prelude::*;
use serenity::prelude::*;

use crate::database::{
    get_cat_by_id, get_cat_memories,
    get_cat_server_stats, get_daily_cat_count, get_refuge_cats,
    get_user_by_discord_id, get_user_cat_counts, get_user_cats, move_cat_to_refuge, set_cat_nickname, set_favorite_cat,
    DatabasePool,
};
use crate::retention::{
    get_equipped_decorations, is_cat_on_expedition, record_activity,
};

use super::*;

#[command]
#[description = "Affiche ta maison ou celle d'un membre"]
pub async fn house(ctx: &Context, msg: &Message) -> CommandResult {
    show_house(ctx, msg, false).await
}

#[command]
#[description = "Visite la maison d'un membre"]
pub async fn visite(ctx: &Context, msg: &Message) -> CommandResult {
    show_house(ctx, msg, true).await
}

pub(super) async fn show_house(ctx: &Context, msg: &Message, is_visit: bool) -> CommandResult {
    let data = ctx.data.read().await;
    let pool = data
        .get::<DatabasePool>()
        .expect("Impossible d'obtenir le pool");

    let target = msg
        .mentions
        .first()
        .cloned()
        .unwrap_or_else(|| msg.author.clone());
    let user = match get_user_by_discord_id(pool, target.id.0).await {
        Ok(Some(user)) => user,
        Ok(None) => {
            msg.channel_id
                .say(&ctx.http, "Cette personne n'a pas encore de profil.")
                .await
                .ok();
            return Ok(());
        }
        Err(_) => {
            msg.channel_id
                .say(&ctx.http, "Erreur avec la base de données")
                .await
                .ok();
            return Ok(());
        }
    };

    let daily_total = get_daily_cat_count(pool, user.id).await.unwrap_or(0);
    let counts = get_user_cat_counts(pool, user.id)
        .await
        .unwrap_or(crate::database::CatCounts {
            home: 0,
            refuge: 0,
            total: 0,
        });
    let cats = get_user_cats(pool, user.id).await.unwrap_or_default();
    let decorations = get_equipped_decorations(pool, user.id)
        .await
        .unwrap_or_default();

    let title = if is_visit && target.id != msg.author.id {
        format!("🏠 Visite chez {}", target.name)
    } else {
        format!("🏠 Maison de {}", target.name)
    };

    let mut response = format!(
        "{}\n🐱 Daily cats classiques: {}\n🏠 {} résidents vivent ici\n🐾 {} résidents confiés au refuge\n📊 Total résidents accueillis: {}\n",
        title,
        daily_total,
        counts.home,
        counts.refuge,
        counts.total
    );

    if cats.is_empty() {
        response += "\nAucun résident ne vit encore ici.";
    } else {
        if let Some(favorite) = cats.iter().find(|cat| cat.is_favorite) {
            response += &format!("\n⭐ Favori: {}\n", format_cat_scene(favorite));
        }

        response += "\n";
        for cat in cats.iter().take(8) {
            response += &format!("• {}\n", format_cat_scene(cat));
        }

        if cats.len() > 8 {
            response += &format!(
                "... et {} autres résidents se reposent dans la maison.\n",
                cats.len() - 8
            );
        }
    }

    if !decorations.is_empty() {
        response += &format!("\n\n🪴 Décorations : {}", decorations.join(", "));
    }

    if is_visit && target.id != msg.author.id {
        if let Ok(Some(visitor)) = get_user_by_discord_id(pool, msg.author.id.0).await {
            record_activity(pool, visitor.id, "visit").await.ok();
        }
    }

    msg.channel_id.say(&ctx.http, response).await.ok();
    Ok(())
}

#[command]
#[description = "Affiche les chats confiés au refuge"]
pub async fn refuge(ctx: &Context, msg: &Message) -> CommandResult {
    let data = ctx.data.read().await;
    let pool = data
        .get::<DatabasePool>()
        .expect("Impossible d'obtenir le pool");

    match get_refuge_cats(pool, 12).await {
        Ok(cats) if cats.is_empty() => {
            msg.channel_id
                .say(&ctx.http, "🐾 Le refuge est vide pour le moment.")
                .await
                .ok();
        }
        Ok(cats) => {
            let mut response = format!(
                "🐾 **Refuge du serveur**\n{} résidents y vivent actuellement.\n\n",
                cats.len()
            );
            for cat in cats {
                response += &format!(
                    "• {} — {}\n",
                    format_cat_identity(&cat),
                    format_cat_scene(&cat)
                );
            }
            msg.channel_id.say(&ctx.http, response).await.ok();
        }
        Err(_) => {
            msg.channel_id
                .say(&ctx.http, "Erreur lors de la récupération du refuge.")
                .await
                .ok();
        }
    }

    Ok(())
}

#[command]
#[description = "Confie un chat au refuge"]
pub async fn refuge_donner(ctx: &Context, msg: &Message, mut args: Args) -> CommandResult {
    let cat_id = match args.single::<i32>() {
        Ok(id) => id,
        Err(_) => {
            msg.channel_id
                .say(&ctx.http, "Usage: `^^refuge_donner <id_chat>`")
                .await
                .ok();
            return Ok(());
        }
    };

    let data = ctx.data.read().await;
    let pool = data
        .get::<DatabasePool>()
        .expect("Impossible d'obtenir le pool");
    let user = match get_user_by_discord_id(pool, msg.author.id.0).await {
        Ok(Some(user)) => user,
        _ => {
            msg.channel_id
                .say(&ctx.http, "Tu n'as pas encore de profil.")
                .await
                .ok();
            return Ok(());
        }
    };

    let cat = match get_cat_by_id(pool, cat_id).await {
        Ok(Some(cat)) if cat.user_id == user.id as i32 && cat.location == "home" => cat,
        Ok(Some(_)) => {
            msg.channel_id
                .say(&ctx.http, "Ce chat ne vit pas chez toi.")
                .await
                .ok();
            return Ok(());
        }
        _ => {
            msg.channel_id
                .say(&ctx.http, "Chat introuvable.")
                .await
                .ok();
            return Ok(());
        }
    };

    if is_cat_on_expedition(pool, cat_id).await.unwrap_or(false) {
        msg.channel_id
            .say(
                &ctx.http,
                "Ce chat est en promenade. Il pourra rejoindre le refuge après son retour sain et sauf.",
            )
            .await
            .ok();
        return Ok(());
    }

    if move_cat_to_refuge(pool, cat_id, user.id).await.is_err() {
        msg.channel_id
            .say(&ctx.http, "Impossible de confier ce chat au refuge.")
            .await
            .ok();
        return Ok(());
    }

    msg.channel_id
        .say(
            &ctx.http,
            format!(
        "🐾 Tu as confié {} au refuge. Il ne vit plus chez toi, mais il garde son existence.",
        format_cat_identity(&cat)
    ),
        )
        .await
        .ok();

    Ok(())
}

#[command]
#[description = "Ajoute ou retire le surnom d'un chat"]
pub async fn surnom(ctx: &Context, msg: &Message) -> CommandResult {
    let mut parts = msg.content.splitn(3, char::is_whitespace);
    let _command = parts.next();
    let cat_id = match parts.next().and_then(|value| value.parse::<i32>().ok()) {
        Some(id) => id,
        None => {
            msg.channel_id
                .say(
                    &ctx.http,
                    "Usage: `^^surnom <id_chat> <surnom>` ou `^^surnom <id_chat> reset`",
                )
                .await
                .ok();
            return Ok(());
        }
    };

    let nickname = parts.next().map(str::trim).unwrap_or("");
    if nickname.is_empty() {
        msg.channel_id
            .say(&ctx.http, "Indique un surnom, ou `reset` pour le retirer.")
            .await
            .ok();
        return Ok(());
    }

    if nickname.chars().count() > 40 {
        msg.channel_id
            .say(&ctx.http, "Le surnom doit faire 40 caractères maximum.")
            .await
            .ok();
        return Ok(());
    }

    let data = ctx.data.read().await;
    let pool = data
        .get::<DatabasePool>()
        .expect("Impossible d'obtenir le pool");
    let user = match get_user_by_discord_id(pool, msg.author.id.0).await {
        Ok(Some(user)) => user,
        _ => {
            msg.channel_id
                .say(&ctx.http, "Tu n'as pas encore de profil.")
                .await
                .ok();
            return Ok(());
        }
    };

    let new_nickname = if nickname.eq_ignore_ascii_case("reset") {
        None
    } else {
        Some(nickname)
    };

    if set_cat_nickname(pool, cat_id, user.id, new_nickname)
        .await
        .is_err()
    {
        msg.channel_id
            .say(&ctx.http, "Impossible de modifier le surnom.")
            .await
            .ok();
        return Ok(());
    }

    let message = match new_nickname {
        Some(value) => format!("Surnom enregistré: **{}**.", value),
        None => "Surnom retiré.".to_string(),
    };
    msg.channel_id.say(&ctx.http, message).await.ok();

    Ok(())
}

#[command]
#[description = "Choisit ton chat favori"]
pub async fn favori(ctx: &Context, msg: &Message, mut args: Args) -> CommandResult {
    let cat_id = match args.single::<i32>() {
        Ok(id) => id,
        Err(_) => {
            msg.channel_id
                .say(&ctx.http, "Usage: `^^favori <id_chat>`")
                .await
                .ok();
            return Ok(());
        }
    };

    let data = ctx.data.read().await;
    let pool = data
        .get::<DatabasePool>()
        .expect("Impossible d'obtenir le pool");
    let user = match get_user_by_discord_id(pool, msg.author.id.0).await {
        Ok(Some(user)) => user,
        _ => {
            msg.channel_id
                .say(&ctx.http, "Tu n'as pas encore de profil.")
                .await
                .ok();
            return Ok(());
        }
    };

    match get_cat_by_id(pool, cat_id).await {
        Ok(Some(cat)) if cat.user_id == user.id as i32 && cat.location == "home" => {
            set_favorite_cat(pool, user.id, cat_id).await?;
            msg.channel_id
                .say(
                    &ctx.http,
                    format!(
                        "⭐ {} est maintenant ton chat favori.",
                        cat_display_name(&cat)
                    ),
                )
                .await
                .ok();
        }
        _ => {
            msg.channel_id
                .say(&ctx.http, "Ce chat ne vit pas chez toi.")
                .await
                .ok();
        }
    }

    Ok(())
}

#[command]
#[description = "Affiche la fiche d'un chat"]
pub async fn chat(ctx: &Context, msg: &Message, mut args: Args) -> CommandResult {
    let cat_id = match args.single::<i32>() {
        Ok(id) => id,
        Err(_) => {
            msg.channel_id
                .say(&ctx.http, "Usage: `^^chat <id_chat>`")
                .await
                .ok();
            return Ok(());
        }
    };

    let data = ctx.data.read().await;
    let pool = data
        .get::<DatabasePool>()
        .expect("Impossible d'obtenir le pool");

    let cat = match get_cat_by_id(pool, cat_id).await {
        Ok(Some(cat)) => cat,
        _ => {
            msg.channel_id
                .say(&ctx.http, "Chat introuvable.")
                .await
                .ok();
            return Ok(());
        }
    };

    let location = if cat.location == "refuge" {
        "Refuge"
    } else {
        "Maison"
    };
    let mut response = format!(
        "{}\nRace: {}\nCouleur: {}\nAge: {}\nRareté: {}\nTempérament: {}\nLieu actuel: {}",
        format_cat_identity(&cat),
        cat.breed,
        cat.color,
        format_age(cat.age_months),
        cat.rarity_score,
        cat.personality.as_deref().unwrap_or("mysterieux"),
        location
    );

    if let Ok(memories) = get_cat_memories(pool, cat.id, 4).await {
        if !memories.is_empty() {
            response += "\n\nSouvenirs:";
            for memory in memories {
                response += &format!("\n• {}", memory.description);
            }
        }
    }

    msg.channel_id.say(&ctx.http, response).await.ok();
    Ok(())
}

#[command]
#[description = "Affiche des statistiques sur les chats"]
pub async fn catstats(ctx: &Context, msg: &Message) -> CommandResult {
    let data = ctx.data.read().await;
    let pool = data
        .get::<DatabasePool>()
        .expect("Impossible d'obtenir le pool");

    match get_cat_server_stats(pool).await {
        Ok((daily, home, refuge_count)) => {
            msg.channel_id.say(&ctx.http, format!(
                "📊 **Stats chats du serveur**\n🐱 Daily cats classiques: {}\n🏠 Résidents dans les maisons: {}\n🐾 Résidents au refuge: {}\n📊 Total résidents nommés: {}",
                daily,
                home,
                refuge_count,
                home + refuge_count
            )).await.ok();
        }
        Err(_) => {
            msg.channel_id
                .say(&ctx.http, "Impossible de récupérer les stats.")
                .await
                .ok();
        }
    }

    Ok(())
}
