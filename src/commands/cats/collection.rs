//! Commandes de collection : `cat`, `mycats`, `cats` et `trade`.

use rand::{thread_rng, Rng};
use serenity::framework::standard::{macros::command, Args, CommandResult};
use serenity::model::prelude::*;
use serenity::prelude::*;

use crate::database::{
    add_collected_cat, add_daily_cat, get_cat_by_id, get_daily_cat_count, get_daily_cat_count_today,
    get_user_by_discord_id, get_user_cat_counts, get_user_cats,
    has_daily_cat_today, new_user, transfer_cat,
    DatabasePool,
};
use crate::retention::{
    is_cat_on_expedition,
    record_daily_cat_and_roll_challenge,
};

use super::*;

#[command]
#[description = "Gagne un Daily Cat (avec parfois un chat secret bonus !)"]
pub async fn cat(ctx: &Context, msg: &Message) -> CommandResult {
    let data = ctx.data.read().await;
    let pool = data
        .get::<DatabasePool>()
        .expect("Impossible d'obtenir le pool");

    let user = match get_user_by_discord_id(pool, msg.author.id.0).await {
        Ok(Some(u)) => u,
        Ok(None) => {
            let user_id = new_user(pool, msg.author.id.0, &msg.author.name).await?;
            crate::database::User {
                id: user_id,
                id_utilisateur: msg.author.id.0.to_string(),
                pseudo: msg.author.name.clone(),
                score: 0,
            }
        }
        Err(_) => {
            msg.channel_id
                .say(&ctx.http, "Erreur avec la base de données")
                .await
                .ok();
            return Ok(());
        }
    };

    if has_daily_cat_today(pool, user.id).await.unwrap_or(false) {
        let messages = [
            "Tu as déjà récupéré ton 🐱 du jour. Il dort maintenant dans un coin en prétendant ne pas te connaître.",
            "Ton 🐱 quotidien est déjà passé aujourd'hui. Il a laissé quelques poils sur le canapé avant de disparaître.",
            "Pas de deuxième 🐱 aujourd'hui. Celui de ce matin surveille déjà la maison avec beaucoup trop de sérieux.",
            "Tu as déjà eu ton 🐱 du jour. Reviens demain, le refuge aura peut-être une autre surprise.",
            "Le chat du jour est déjà chez toi. Il refuse catégoriquement de se dupliquer.",
            "Déjà fait pour aujourd'hui. Ton 🐱 réclame plutôt une pause et un coussin propre.",
        ];
        let message = messages[thread_rng().gen_range(0..messages.len())];
        msg.channel_id
            .say(&ctx.http, message)
            .await
            .ok();
        return Ok(());
    }

    // Donner le daily cat comme avant
    if add_daily_cat(pool, user.id).await.is_err() {
        msg.channel_id
            .say(&ctx.http, "Impossible d'ajouter le Cat")
            .await
            .ok();
        return Ok(());
    }

    let total = match get_daily_cat_count(pool, user.id).await {
        Ok(c) => c,
        Err(_) => {
            msg.channel_id
                .say(&ctx.http, "Erreur avec la base de données")
                .await
                .ok();
            return Ok(());
        }
    };

    // Message principal comme avant
    let mut response = format!("Tu as gagné un 🐱 ! Total: {}", total);

    // 70% de chance d'obtenir un chat secret EN PLUS (événement spécial première journée)
    let secret_cat_chance = {
        let mut rng = thread_rng();
        rng.gen_range(0..100) < 15
    };

    if secret_cat_chance {
        let secret_cat = Cat::generate_random();

        // Sauvegarder le chat secret en base
        match add_collected_cat(
            pool,
            user.id,
            &secret_cat.as_new_cat(),
        )
        .await
        {
            Ok(cat_id) => {
                let rarity_emoji = get_rarity_emoji(secret_cat.rarity_score);
                let temperament = match get_cat_by_id(pool, cat_id).await {
                    Ok(Some(cat)) => cat.personality.unwrap_or_else(|| "mysterieux".to_string()),
                    _ => "mysterieux".to_string(),
                };
                response += &format!(
                    "\n\n🏠 **Un chat errant a rejoint ton foyer !** 🏠\n{} **{}** (#{}) s'est installé chez toi !\nTempérament: **{}**\n\n💫 Utilise `^^house` pour voir qui vit dans ta maison !",
                    rarity_emoji,
                    secret_cat.format_description(),
                    cat_id,
                    temperament
                );
            }
            Err(_) => {
                // Pas grave si ça rate, on garde juste le daily cat
            }
        }
    }

    msg.channel_id.say(&ctx.http, response).await.ok();
    record_daily_cat_and_roll_challenge(ctx, msg, pool, user.id).await;
    if matches!(get_daily_cat_count_today(pool).await, Ok(1)) {
        maybe_trigger_cat_event(ctx, msg, pool).await;
    }

    Ok(())
}

#[command]
#[description = "Affiche ta collection de chats"]
pub async fn mycats(ctx: &Context, msg: &Message, mut args: Args) -> CommandResult {
    let data = ctx.data.read().await;
    let pool = data
        .get::<DatabasePool>()
        .expect("Impossible d'obtenir le pool");
    let requested_page = args.single::<usize>().unwrap_or(1).max(1);

    let user = match get_user_by_discord_id(pool, msg.author.id.0).await {
        Ok(Some(u)) => u,
        Ok(None) => {
            msg.channel_id
                .say(
                    &ctx.http,
                    "Tu n'as pas encore de profil. Utilise ^^np pour en créer un",
                )
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

    match get_user_cats(pool, user.id).await {
        Ok(cats) => {
            if cats.is_empty() {
                msg.channel_id.say(&ctx.http, "Aucun chat ne vit encore chez toi ! Utilise `^^cat` pour peut-être en adopter un.").await.ok();
            } else {
                let (response, page, total_pages) =
                    format_mycats_page(&msg.author.name, &cats, requested_page);

                if let Ok(page_message) = msg.channel_id.say(&ctx.http, response).await {
                    if total_pages > 1 {
                        handle_mycats_reactions(
                            ctx,
                            msg.author.id,
                            page_message,
                            msg.author.name.clone(),
                            cats,
                            page,
                            total_pages,
                        )
                        .await;
                    }
                }
            }
        }
        Err(_) => {
            msg.channel_id
                .say(&ctx.http, "Erreur lors de la récupération de ta collection")
                .await
                .ok();
        }
    }

    Ok(())
}

#[command]
#[description = "Affiche le nombre de Daily Cats"]
pub async fn cats(ctx: &Context, msg: &Message) -> CommandResult {
    let data = ctx.data.read().await;
    let pool = data
        .get::<DatabasePool>()
        .expect("Impossible d'obtenir le pool");

    match get_user_by_discord_id(pool, msg.author.id.0).await {
        Ok(Some(user)) => match get_daily_cat_count(pool, user.id).await {
            Ok(total) => {
                let counts = get_user_cat_counts(pool, user.id).await.ok();
                let mut response = format!("🐱 Daily cats classiques: {}", total);

                if let Some(counts) = counts {
                    response += &format!(
                            "\n🏠 Résidents chez toi: {}\n🐾 Résidents confiés au refuge: {}\n📊 Total résidents accueillis: {}",
                            counts.home,
                            counts.refuge,
                            counts.total
                        );
                }

                msg.channel_id.say(&ctx.http, response).await.ok();
            }
            Err(_) => {
                msg.channel_id
                    .say(&ctx.http, "Erreur avec la base de données")
                    .await
                    .ok();
            }
        },
        Ok(None) => {
            msg.channel_id
                .say(
                    &ctx.http,
                    "Tu n'as pas encore de profil. Utilise ^^np pour en créer un",
                )
                .await
                .ok();
        }
        Err(_) => {
            msg.channel_id
                .say(&ctx.http, "Erreur avec la base de données")
                .await
                .ok();
        }
    }

    Ok(())
}

#[command]
#[description = "Donne un de tes chats à un autre utilisateur : ^^trade @user chat_id"]
pub async fn trade(ctx: &Context, msg: &Message) -> CommandResult {
    let data = ctx.data.read().await;
    let pool = data
        .get::<DatabasePool>()
        .expect("Impossible d'obtenir le pool");

    let args: Vec<&str> = msg.content.split_whitespace().collect();
    if args.len() != 3 {
        msg.channel_id.say(&ctx.http, "Usage: `^^trade @utilisateur chat_id`\nExemple: `^^trade @John 15`\nCela donnera ton chat à cet utilisateur.").await.ok();
        return Ok(());
    }

    let user = match get_user_by_discord_id(pool, msg.author.id.0).await {
        Ok(Some(u)) => u,
        Ok(None) => {
            msg.channel_id
                .say(
                    &ctx.http,
                    "Tu n'as pas encore de profil. Utilise ^^np pour en créer un",
                )
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

    // Parse cat_id
    let cat_id = match args[2].parse::<i32>() {
        Ok(id) => id,
        Err(_) => {
            msg.channel_id
                .say(&ctx.http, "ID de chat invalide !")
                .await
                .ok();
            return Ok(());
        }
    };

    // Vérifier que le chat appartient à l'utilisateur
    match get_cat_by_id(pool, cat_id).await {
        Ok(Some(cat)) => {
            if cat.user_id != user.id as i32 {
                msg.channel_id
                    .say(&ctx.http, "Ce chat ne t'appartient pas !")
                    .await
                    .ok();
                return Ok(());
            }

            if is_cat_on_expedition(pool, cat_id).await.unwrap_or(false) {
                msg.channel_id
                    .say(
                        &ctx.http,
                        "Ce chat est en promenade. Il pourra déménager après son retour sain et sauf.",
                    )
                    .await
                    .ok();
                return Ok(());
            }

            // Récupérer l'utilisateur cible
            let target_user = match crate::commands::get_user_by_mention(ctx, msg, args[1]).await {
                Ok(u) => u,
                Err(_) => return Ok(()),
            };

            let target_db_user = match get_user_by_discord_id(pool, target_user.id.0).await {
                Ok(Some(u)) => u,
                Ok(None) => {
                    msg.channel_id
                        .say(&ctx.http, "Cet utilisateur n'a pas de profil !")
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

            // Effectuer le transfert
            match transfer_cat(pool, cat_id, user.id, target_db_user.id).await {
                Ok(true) => {
                    let rarity_emoji = get_rarity_emoji(cat.rarity_score);

                    let age_display = format_age(cat.age_months);

                    msg.channel_id.say(&ctx.http, format!(
                        "✅ **Adoption réussie !**\n\n{} **{} {} {} de {}** (#{}) a déménagé chez {} !\n\n🏠 {}",
                        rarity_emoji,
                        cat.name,
                        cat.breed,
                        cat.color,
                        age_display,
                        cat.id,
                        target_user.name,
                        target_user.mention()
                    )).await.ok();
                }
                Ok(false) => {
                    msg.channel_id
                        .say(
                            &ctx.http,
                            "Ce chat ne peut plus être transféré (déjà échangé ou parti en promenade).",
                        )
                        .await
                        .ok();
                }
                Err(_) => {
                    msg.channel_id
                        .say(&ctx.http, "Erreur lors du transfert !")
                        .await
                        .ok();
                }
            }
        }
        Ok(None) => {
            msg.channel_id
                .say(&ctx.http, "Chat introuvable !")
                .await
                .ok();
        }
        Err(_) => {
            msg.channel_id
                .say(&ctx.http, "Erreur avec la base de données")
                .await
                .ok();
        }
    }

    Ok(())
}
