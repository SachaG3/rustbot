use serenity::prelude::*;
use serenity::model::id::ChannelId;
use serenity::model::channel::Message;
use chrono::{DateTime, Utc, Datelike, Timelike, NaiveDate};
use rand::{thread_rng, Rng};
use sqlx::{Pool, MySql};

use crate::database::{
    DatabasePool, get_user_by_discord_id, new_user,
    add_daily_cat_with_date, has_daily_cat_on_date,
    add_collected_cat_with_date, get_daily_cat_count
};
use crate::commands::cats::{Cat, get_rarity_emoji};

const DISCORD_EPOCH: i64 = 1420070400000;
const MAX_MESSAGES_PER_DAY: usize = 1000;
const CHECKUP_DAYS: i64 = 14;

// Construit un Snowflake boundary pour une date donnée
fn build_snowflake_boundary(date: DateTime<Utc>, is_before: bool) -> u64 {
    let timestamp = date.timestamp_millis();
    let ms_since_epoch = timestamp - DISCORD_EPOCH;

    if ms_since_epoch <= 0 {
        return if is_before { u64::MAX >> 42 } else { 0 };
    }

    let base_snowflake = (ms_since_epoch as u64) << 22;

    if is_before {
        base_snowflake | ((1u64 << 22) - 1)
    } else {
        base_snowflake.saturating_sub(1).max(0)
    }
}

// Récupère les bounds (début et fin) d'une journée
fn get_day_bounds(days_ago: i64) -> (DateTime<Utc>, DateTime<Utc>) {
    let now = Utc::now();
    let start = now
        .date_naive()
        .and_hms_opt(0, 0, 0)
        .unwrap()
        - chrono::Duration::days(days_ago);
    let end = start + chrono::Duration::days(1) - chrono::Duration::seconds(1);

    (
        DateTime::from_naive_utc_and_offset(start, Utc),
        DateTime::from_naive_utc_and_offset(end, Utc),
    )
}

// Récupère la date locale au format YYYY-MM-DD
fn get_local_date(days_ago: i64) -> String {
    let now = Utc::now();
    let date = now.date_naive() - chrono::Duration::days(days_ago);
    date.format("%Y-%m-%d").to_string()
}

pub async fn perform_cat_checkup(ctx: &Context) {
    let channel_id = ChannelId(764781222492635169);

    println!("🔍 Début du checkup des cats des {} derniers jours...", CHECKUP_DAYS);

    let data = ctx.data.read().await;
    let pool = match data.get::<DatabasePool>() {
        Some(pool) => pool.clone(),
        None => {
            println!("❌ Impossible d'obtenir le pool de base de données");
            return;
        }
    };

    let mut total_cats_added = 0;
    let mut total_special_cats_added = 0;

    // Parcourir les 14 derniers jours (jour actuel + 13 précédents)
    for day_offset in 0..CHECKUP_DAYS {
        let date_to_check = get_local_date(day_offset);
        println!("📅 Vérification du {}...", date_to_check);

        let (start_of_day, end_of_day) = get_day_bounds(day_offset);
        let after_snowflake = build_snowflake_boundary(start_of_day, false);
        let before_snowflake = build_snowflake_boundary(end_of_day, true);

        // Récupérer les messages du jour
        let messages = match fetch_messages_for_day(
            ctx,
            channel_id,
            after_snowflake,
            before_snowflake,
        ).await {
            Ok(msgs) => msgs,
            Err(e) => {
                println!("  ❌ Erreur lors de la récupération des messages: {}", e);
                continue;
            }
        };

        println!("  📝 {} messages récupérés pour le {}", messages.len(), date_to_check);

        // Filtrer les messages contenant ^^cat
        let cat_messages: Vec<&Message> = messages
            .iter()
            .filter(|msg| {
                msg.content.to_lowercase().contains("^^cat") &&
                !msg.author.bot &&
                msg.timestamp.timestamp() >= start_of_day.timestamp() &&
                msg.timestamp.timestamp() <= end_of_day.timestamp()
            })
            .collect();

        println!("  📝 {} messages ^^cat trouvés pour le {}", cat_messages.len(), date_to_check);

        // Traiter chaque message ^^cat
        for message in cat_messages {
            let user_id = message.author.id.0;

            // Récupérer ou créer l'utilisateur
            let user = match get_user_by_discord_id(&pool, user_id).await {
                Ok(Some(u)) => u,
                Ok(None) => {
                    // Créer un nouvel utilisateur
                    match new_user(&pool, user_id, &message.author.name).await {
                        Ok(new_user_id) => crate::database::User {
                            id: new_user_id,
                            id_utilisateur: user_id.to_string(),
                            pseudo: message.author.name.clone(),
                            score: 0,
                        },
                        Err(e) => {
                            println!("  ❌ Erreur lors de la création de l'utilisateur {}: {}", user_id, e);
                            continue;
                        }
                    }
                },
                Err(e) => {
                    println!("  ❌ Erreur base de données pour {}: {}", user_id, e);
                    continue;
                }
            };

            // Vérifier si l'utilisateur a déjà son daily cat pour cette date
            match has_daily_cat_on_date(&pool, user.id, &date_to_check).await {
                Ok(true) => {
                    println!("  ⏭️  {} a déjà son cat pour le {}", user_id, date_to_check);
                    continue;
                }
                Ok(false) => {
                    // Attribuer le daily cat
                    let timestamp = message.timestamp.format("%Y-%m-%d %H:%M:%S").to_string();

                    match add_daily_cat_with_date(&pool, user.id, &timestamp).await {
                        Ok(_) => {
                            total_cats_added += 1;

                            // Compter le total de cats
                            let total_count = match get_daily_cat_count(&pool, user.id).await {
                                Ok(count) => count,
                                Err(_) => 0,
                            };

                            println!("  ✅ Cat attribué à {} pour le {} (total: {})", user_id, date_to_check, total_count);

                            // Message de rattrapage
                            if let Err(e) = channel_id.say(&ctx.http, format!(
                                "🐱 Cat de rattrapage attribué à <@{}> pour le {}! Tu en as maintenant {}.",
                                user_id, date_to_check, total_count
                            )).await {
                                println!("  ⚠️  Erreur lors de l'envoi du message: {}", e);
                            }

                            // 15% de chance d'obtenir un chat spécial EN PLUS
                            let secret_cat_option = {
                                let mut rng = thread_rng();
                                if rng.gen_range(0..100) < 15 {
                                    Some(Cat::generate_random())
                                } else {
                                    None
                                }
                            };

                            if let Some(secret_cat) = secret_cat_option {
                                match add_collected_cat_with_date(
                                    &pool,
                                    user.id,
                                    &secret_cat.name,
                                    secret_cat.breed.name,
                                    secret_cat.color.name,
                                    secret_cat.age_months,
                                    secret_cat.rarity_score,
                                    &timestamp
                                ).await {
                                    Ok(cat_id) => {
                                        total_special_cats_added += 1;
                                        let rarity_emoji = get_rarity_emoji(secret_cat.rarity_score);

                                        println!("  🏠 Chat spécial attribué à {} (rareté: {})", user_id, secret_cat.rarity_score);

                                        if let Err(e) = channel_id.say(&ctx.http, format!(
                                            "🏠 **Un chat errant a rejoint le foyer de <@{}> !** 🏠\n{} **{}** (#{}) s'est installé chez toi !",
                                            user_id,
                                            rarity_emoji,
                                            secret_cat.format_description(),
                                            cat_id
                                        )).await {
                                            println!("  ⚠️  Erreur lors de l'envoi du message de chat spécial: {}", e);
                                        }
                                    }
                                    Err(e) => {
                                        println!("  ⚠️  Erreur lors de l'ajout du chat spécial: {}", e);
                                    }
                                }
                            }

                            // Délai pour éviter le rate limiting
                            tokio::time::sleep(tokio::time::Duration::from_millis(1000)).await;
                        }
                        Err(e) => {
                            println!("  ❌ Erreur lors de l'attribution du cat à {}: {}", user_id, e);
                        }
                    }
                }
                Err(e) => {
                    println!("  ❌ Erreur lors de la vérification du cat pour {}: {}", user_id, e);
                }
            }
        }

        // Délai entre chaque jour
        tokio::time::sleep(tokio::time::Duration::from_millis(500)).await;
    }

    println!("✅ Checkup terminé !");
    println!("📊 {} daily cats ajoutés", total_cats_added);
    println!("📊 {} chats spéciaux ajoutés", total_special_cats_added);
}

async fn fetch_messages_for_day(
    ctx: &Context,
    channel_id: ChannelId,
    after_snowflake: u64,
    before_snowflake: u64,
) -> Result<Vec<Message>, Box<dyn std::error::Error>> {
    let mut all_messages = Vec::new();
    let mut current_before = before_snowflake;

    while all_messages.len() < MAX_MESSAGES_PER_DAY && current_before > after_snowflake {
        let batch = channel_id
            .messages(&ctx.http, |retriever| {
                retriever
                    .limit(100)
                    .before(current_before)
            })
            .await?;

        if batch.is_empty() {
            break;
        }

        // Filtrer les messages dans la plage temporelle
        let filtered: Vec<Message> = batch
            .into_iter()
            .filter(|msg| msg.id.0 > after_snowflake)
            .collect();

        if filtered.is_empty() {
            break;
        }

        // Trouver le message le plus ancien
        if let Some(oldest) = filtered.iter().min_by_key(|msg| msg.id.0) {
            current_before = oldest.id.0.saturating_sub(1);

            if current_before <= after_snowflake {
                all_messages.extend(filtered);
                break;
            }
        }

        all_messages.extend(filtered);

        // Délai pour éviter le rate limiting
        tokio::time::sleep(tokio::time::Duration::from_millis(200)).await;
    }

    Ok(all_messages)
}
