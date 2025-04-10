use serenity::framework::standard::{macros::command, CommandResult};
use serenity::model::prelude::*;
use serenity::prelude::*;
use tracing::error;

use crate::database::{DatabasePool, get_user_by_discord_id, new_user, update_user_score};
use crate::utils::generate_score_image;

#[command]
#[description = "Créer un nouveau profil"]
async fn np(ctx: &Context, msg: &Message) -> CommandResult {
    let data = ctx.data.read().await;
    let pool = data.get::<DatabasePool>().expect("Erreur lors de l'obtention du pool de base de données");
    
    match get_user_by_discord_id(&pool, msg.author.id.0).await {
        Ok(Some(_)) => {
            if let Err(why) = msg.channel_id.say(&ctx.http, "Un profil existe déjà pour cet utilisateur.").await {
                error!("Erreur lors de l'envoi du message: {:?}", why);
            }
        },
        Ok(None) => {
            match new_user(&pool, msg.author.id.0, &msg.author.name).await {
                Ok(_) => {
                    if let Err(why) = msg.channel_id.say(&ctx.http, "Profil créé avec succès!").await {
                        error!("Erreur lors de l'envoi du message: {:?}", why);
                    }
                },
                Err(e) => {
                    error!("Erreur lors de la création du profil: {:?}", e);
                    if let Err(why) = msg.channel_id.say(&ctx.http, "Une erreur est survenue lors de la création du profil.").await {
                        error!("Erreur lors de l'envoi du message: {:?}", why);
                    }
                }
            }
        },
        Err(e) => {
            error!("Erreur lors de la recherche de l'utilisateur: {:?}", e);
            if let Err(why) = msg.channel_id.say(&ctx.http, "Une erreur est survenue lors de la recherche de votre profil.").await {
                error!("Erreur lors de l'envoi du message: {:?}", why);
            }
        }
    }
    
    Ok(())
}

#[command]
#[description = "Affiche votre score actuel"]
async fn score(ctx: &Context, msg: &Message) -> CommandResult {
    let data = ctx.data.read().await;
    let pool = data.get::<DatabasePool>().expect("Erreur lors de l'obtention du pool de base de données");
    
    match get_user_by_discord_id(&pool, msg.author.id.0).await {
        Ok(Some(user)) => {
            let score = user.score;
            
            // Obtenez l'URL de l'avatar
            let avatar_url = msg.author.avatar_url().unwrap_or_else(|| msg.author.default_avatar_url());
            
            // Générez l'image avec le score
            match generate_score_image(&avatar_url, score).await {
                Some(image_data) => {
                    // Envoyez l'image au canal avec un message plus élégant
                    if let Err(why) = msg.channel_id.send_message(&ctx.http, |m| {
                        m.embed(|e| {
                            e.title(format!("🏆 Profil de {}", msg.author.name))
                             .color((45, 70, 180)) // Couleur bleue élégante
                             .description(format!("Voici votre carte de score, <@{}>!", msg.author.id.0))
                             .footer(|f| f.text("Gagnez plus de points en jouant à nos jeux !"))
                             .timestamp(chrono::Utc::now().to_rfc3339())
                        })
                        .add_file((&image_data[..], "score.png"))
                    }).await {
                        error!("Erreur lors de l'envoi de l'image: {:?}", why);
                        
                        // Fallback en cas d'erreur d'envoi d'image
                        if let Err(why) = msg.channel_id.say(&ctx.http, format!("Votre score est de {} points.", score)).await {
                            error!("Erreur lors de l'envoi du message de fallback: {:?}", why);
                        }
                    }
                },
                None => {
                    // Fallback si l'image ne peut pas être générée
                    if let Err(why) = msg.channel_id.say(&ctx.http, format!("Votre score est de {} points.", score)).await {
                        error!("Erreur lors de l'envoi du message: {:?}", why);
                    }
                }
            }
        },
        Ok(None) => {
            if let Err(why) = msg.channel_id.say(&ctx.http, "Vous n'avez pas encore de profil. Utilisez la commande ^^NP pour en créer un.").await {
                error!("Erreur lors de l'envoi du message: {:?}", why);
            }
        },
        Err(e) => {
            error!("Erreur lors de la recherche de l'utilisateur: {:?}", e);
            if let Err(why) = msg.channel_id.say(&ctx.http, "Une erreur est survenue lors de la recherche de votre profil.").await {
                error!("Erreur lors de l'envoi du message: {:?}", why);
            }
        }
    }
    
    Ok(())
}

// Fonction utilitaire pour mettre à jour le score d'un utilisateur
pub async fn update_profile_score(ctx: &Context, msg: &Message, points: i32) -> CommandResult {
    let data = ctx.data.read().await;
    let pool = data.get::<DatabasePool>().expect("Erreur lors de l'obtention du pool de base de données");
    
    match get_user_by_discord_id(&pool, msg.author.id.0).await {
        Ok(Some(user)) => {
            if let Err(e) = update_user_score(&pool, user.id, points).await {
                error!("Erreur lors de la mise à jour du score: {:?}", e);
            }
        },
        Ok(None) => {
            match new_user(&pool, msg.author.id.0, &msg.author.name).await {
                Ok(user_id) => {
                    if let Err(e) = update_user_score(&pool, user_id, points).await {
                        error!("Erreur lors de la mise à jour du score: {:?}", e);
                    }
                },
                Err(e) => error!("Erreur lors de la création de l'utilisateur: {:?}", e),
            }
        },
        Err(e) => error!("Erreur lors de la recherche de l'utilisateur: {:?}", e),
    }
    
    Ok(())
} 