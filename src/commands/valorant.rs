use serenity::framework::standard::{macros::command, CommandResult};
use serenity::model::prelude::*;
use serenity::prelude::*;
use tracing::error;
use rand::Rng;
use sqlx::{MySql, Pool, Row};

use crate::commands::profile::update_profile_score;
use crate::database::DatabasePool;

// Structure pour les skins d'armes Valorant
#[derive(Clone)]
struct WeaponSkin {
    uuid: String,
    display_name: String,
    weapon_type: Option<String>,
    display_icon: Option<String>,
}

#[command]
#[description = "Affiche votre rang Valorant (bientôt disponible)"]
pub async fn rank(ctx: &Context, msg: &Message) -> CommandResult {
    if let Err(why) = msg.channel_id.say(&ctx.http, "La commande rank sera bientôt disponible. Elle vous permettra de consulter le rang d'un joueur Valorant.").await {
        error!("Erreur lors de l'envoi du message: {:?}", why);
    }
    
    Ok(())
}

#[command]
#[description = "Jeu pour deviner le nom d'un skin d'arme Valorant"]
pub async fn skin(ctx: &Context, msg: &Message) -> CommandResult {
    let data = ctx.data.read().await;
    let pool = data.get::<DatabasePool>().expect("Erreur lors de l'obtention du pool de base de données");
    
    // Récupérer un skin aléatoire avec une icône
    match get_random_weapon_skin(pool).await {
        Ok(Some(correct_weapon)) => {
            // Récupérer des skins similaires pour les options
            match get_similar_weapons(pool, &correct_weapon, 3).await {
                Ok(similar_weapons) => {
                    let mut options = similar_weapons;
                    options.push(correct_weapon.clone());
                    
                    // Mélanger les options
                    options.shuffle();
                    
                    // Envoyer l'image du skin
                    if let Some(icon_url) = &correct_weapon.display_icon {
                        if let Err(why) = msg.channel_id.say(&ctx.http, icon_url).await {
                            error!("Erreur lors de l'envoi de l'image: {:?}", why);
                            return Ok(());
                        }
                    } else {
                        if let Err(why) = msg.channel_id.say(&ctx.http, "Impossible d'afficher l'image du skin.").await {
                            error!("Erreur lors de l'envoi du message: {:?}", why);
                            return Ok(());
                        }
                        return Ok(());
                    }
                    
                    // Préparer le texte du sondage
                    let poll_text = options.iter().enumerate()
                        .map(|(idx, weapon)| format!("{}: {}", idx + 1, weapon.display_name))
                        .collect::<Vec<String>>()
                        .join("\n");
                    
                    // Envoyer les options
                    let poll_message = match msg.channel_id.say(&ctx.http, format!("{}\nQuelle est le nom de cette arme ?", poll_text)).await {
                        Ok(msg) => msg,
                        Err(why) => {
                            error!("Erreur lors de l'envoi du message: {:?}", why);
                            return Ok(());
                        }
                    };
                    
                    // Ajouter les réactions
                    let emojis = ["1️⃣", "2️⃣", "3️⃣", "4️⃣"];
                    for emoji in &emojis {
                        if let Err(why) = poll_message.react(&ctx.http, ReactionType::Unicode(emoji.to_string())).await {
                            error!("Erreur lors de l'ajout de la réaction: {:?}", why);
                        }
                    }
                    
                    // Trouver l'index de la bonne réponse
                    let correct_index = options.iter().position(|weapon| weapon.uuid == correct_weapon.uuid).unwrap_or(0);
                    
                    // Attendre une réaction avec un timeout de 120 secondes
                    let mut user_choice = None;
                    
                    // Boucle pour vérifier toutes les réactions possibles
                    let start_time = std::time::Instant::now();
                    let timeout_duration = std::time::Duration::from_secs(120);
                    
                    while start_time.elapsed() < timeout_duration && user_choice.is_none() {
                        // Vérifier les réactions toutes les secondes
                        tokio::time::sleep(std::time::Duration::from_secs(1)).await;
                        
                        for (idx, emoji) in emojis.iter().enumerate() {
                            if let Ok(reactions) = poll_message.reaction_users(&ctx.http, ReactionType::Unicode(emoji.to_string()), None, None).await {
                                if reactions.iter().any(|u| u.id == msg.author.id) {
                                    user_choice = Some(idx);
                                    break;
                                }
                            }
                        }
                    }
                    
                    match user_choice {
                        Some(choice_idx) => {
                            // Vérifier si c'est la bonne réponse
                            if choice_idx == correct_index {
                                if let Err(why) = msg.channel_id.say(&ctx.http, "Félicitations ! Vous avez bien deviné !").await {
                                    error!("Erreur lors de l'envoi du message: {:?}", why);
                                }
                                update_profile_score(ctx, msg, 2).await?;
                            } else {
                                if let Err(why) = msg.channel_id.say(&ctx.http, 
                                    format!("Désolé, ce n'était pas la bonne réponse. La bonne réponse était {}.", 
                                    correct_weapon.display_name)).await {
                                    error!("Erreur lors de l'envoi du message: {:?}", why);
                                }
                            }
                        },
                        None => {
                            if let Err(why) = msg.channel_id.say(&ctx.http, "Désolé, le temps est écoulé. Essayez encore !").await {
                                error!("Erreur lors de l'envoi du message: {:?}", why);
                            }
                        }
                    }
                },
                Err(e) => {
                    error!("Erreur lors de la récupération des armes similaires: {:?}", e);
                    if let Err(why) = msg.channel_id.say(&ctx.http, "Une erreur est survenue lors de la récupération des armes. Veuillez réessayer plus tard.").await {
                        error!("Erreur lors de l'envoi du message: {:?}", why);
                    }
                }
            }
        },
        Ok(None) => {
            if let Err(why) = msg.channel_id.say(&ctx.http, "Aucun skin d'arme avec image n'a été trouvé. Veuillez réessayer plus tard.").await {
                error!("Erreur lors de l'envoi du message: {:?}", why);
            }
        },
        Err(e) => {
            error!("Erreur lors de la récupération d'un skin aléatoire: {:?}", e);
            if let Err(why) = msg.channel_id.say(&ctx.http, "Une erreur est survenue lors de la récupération des skins. Veuillez réessayer plus tard.").await {
                error!("Erreur lors de l'envoi du message: {:?}", why);
            }
        }
    }
    
    Ok(())
}

// Fonction pour récupérer un skin d'arme aléatoire avec une image
async fn get_random_weapon_skin(pool: &Pool<MySql>) -> Result<Option<WeaponSkin>, sqlx::Error> {
    let rows = sqlx::query("SELECT uuid, displayName, weaponType, displayIcon FROM weapons_skins WHERE displayIcon IS NOT NULL")
        .fetch_all(pool)
        .await?;
    
    if rows.is_empty() {
        return Ok(None);
    }
    
    let mut rng = rand::thread_rng();
    let random_index = rng.gen_range(0..rows.len());
    
    let random_row = &rows[random_index];
    
    Ok(Some(WeaponSkin {
        uuid: random_row.get("uuid"),
        display_name: random_row.get("displayName"),
        weapon_type: random_row.get("weaponType"),
        display_icon: random_row.get("displayIcon"),
    }))
}

// Fonction pour récupérer des skins d'armes similaires
async fn get_similar_weapons(pool: &Pool<MySql>, correct_weapon: &WeaponSkin, num_samples: usize) -> Result<Vec<WeaponSkin>, sqlx::Error> {
    let weapon_type = correct_weapon.weapon_type.clone().unwrap_or_else(|| "Knife".to_string());
    
    let rows = sqlx::query("SELECT uuid, displayName, weaponType, displayIcon FROM weapons_skins 
                           WHERE weaponType = ? AND uuid != ? AND displayIcon IS NOT NULL")
        .bind(&weapon_type)
        .bind(&correct_weapon.uuid)
        .fetch_all(pool)
        .await?;
    
    let mut similar_weapons = Vec::new();
    
    if rows.is_empty() {
        // Si aucune arme similaire n'est trouvée, chercher des armes avec une image
        let fallback_rows = sqlx::query("SELECT uuid, displayName, weaponType, displayIcon FROM weapons_skins 
                                       WHERE uuid != ? AND displayIcon IS NOT NULL LIMIT ?")
            .bind(&correct_weapon.uuid)
            .bind(num_samples as i32)
            .fetch_all(pool)
            .await?;
        
        for row in fallback_rows {
            similar_weapons.push(WeaponSkin {
                uuid: row.get("uuid"),
                display_name: row.get("displayName"),
                weapon_type: row.get("weaponType"),
                display_icon: row.get("displayIcon"),
            });
        }
    } else {
        // Utiliser les armes du même type
        let mut indices: Vec<usize> = (0..rows.len()).collect();
        indices.shuffle();
        
        let sample_count = std::cmp::min(num_samples, rows.len());
        
        for i in 0..sample_count {
            let row = &rows[indices[i]];
            similar_weapons.push(WeaponSkin {
                uuid: row.get("uuid"),
                display_name: row.get("displayName"),
                weapon_type: row.get("weaponType"),
                display_icon: row.get("displayIcon"),
            });
        }
    }
    
    Ok(similar_weapons)
}

// Extension pour mélanger un vecteur
trait VecShuffle {
    fn shuffle(&mut self);
}

impl<T> VecShuffle for Vec<T> {
    fn shuffle(&mut self) {
        let mut rng = rand::thread_rng();
        for i in (1..self.len()).rev() {
            let j = rng.gen_range(0..=i);
            self.swap(i, j);
        }
    }
} 