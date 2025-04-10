use serenity::framework::standard::{macros::command, CommandResult};
use serenity::model::prelude::*;
use serenity::prelude::*;
use tracing::error;

use crate::database::{DatabasePool, new_token, get_user_by_discord_id};

#[command]
#[description = "Affiche la liste des commandes disponibles"]
pub async fn help(ctx: &Context, msg: &Message) -> CommandResult {
    let commands_basic = "`help`, `serverinfo`, `link`, `jeux`";
    let commands_profile = "`np`, `score`";
    let commands_games = "`juste`, `bj`, `usd`";
    let commands_spam = "`rp`, `rpt`";
    let commands_valorant = "`skin`, `rank`";

    if let Err(why) = msg.channel_id.send_message(&ctx.http, |m| {
        m.embed(|e| {
            e.title("Aide du bot")
             .description("Voici toutes les commandes disponibles :")
             .color((65, 105, 225)) // Couleur bleu royal
             .field("Commandes basiques", commands_basic, false)
             .field("Commandes de profil", commands_profile, false)
             .field("Commandes de jeux", commands_games, false)
             .field("Commandes de spam", commands_spam, false)
             .field("Commandes Valorant", commands_valorant, false)
             .footer(|f| f.text("Utilisez ^^help <commande> pour plus d'informations (bientôt disponible)"))
             .timestamp(chrono::Utc::now().to_rfc3339())
        })
    }).await {
        error!("Erreur lors de l'envoi du message d'aide: {:?}", why);
    }

    Ok(())
}

#[command]
#[description = "Affiche les infos du serveur"]
pub async fn serverinfo(ctx: &Context, msg: &Message) -> CommandResult {
    if let Some(guild) = msg.guild(&ctx.cache) {
        let channel_count = guild.channels.len();
        let voice_channel_count = guild.voice_states.len();
        let member_count = guild.member_count;
        let server_name = &guild.name;
        
        let response = format!(
            "Le serveur **{}** contient *{}* personnes !\nCe serveur possède {} salons écrit et {} salon vocaux.",
            server_name, member_count, channel_count, voice_channel_count
        );
        
        if let Err(why) = msg.channel_id.say(&ctx.http, &response).await {
            error!("Erreur lors de l'envoi du message: {:?}", why);
        }
    } else {
        if let Err(why) = msg.channel_id.say(&ctx.http, "Cette commande ne fonctionne que sur un serveur.").await {
            error!("Erreur lors de l'envoi du message: {:?}", why);
        }
    }
    
    Ok(())
}

#[command]
#[description = "Jeux disponibles sur le bot"]
pub async fn jeux(ctx: &Context, msg: &Message) -> CommandResult {
    if let Err(why) = msg.channel_id.say(&ctx.http, "https://discord.gg/jszvZm36r8").await {
        error!("Erreur lors de l'envoi du message: {:?}", why);
    }
    
    Ok(())
}

#[command]
#[description = "Affiche les liens utiles du serveur"]
pub async fn link(ctx: &Context, msg: &Message) -> CommandResult {
    let data = ctx.data.read().await;
    let pool = data.get::<DatabasePool>().expect("Erreur lors de l'obtention du pool de base de données");
    
    match get_user_by_discord_id(&pool, msg.author.id.0).await {
        Ok(Some(user)) => {
            match new_token(&pool, user.id).await {
                Ok(token) => {
                    let url = format!("https://www.soyerbot.fr/token/{}", token);
                    if let Err(why) = msg.author.direct_message(&ctx.http, |m| {
                        m.content(format!("Voici votre lien : {}", url))
                    }).await {
                        error!("Erreur lors de l'envoi du message privé: {:?}", why);
                        
                        if let Err(why) = msg.channel_id.say(&ctx.http, "Impossible de vous envoyer un message privé. Vérifiez que vous autorisez les messages privés sur ce serveur.").await {
                            error!("Erreur lors de l'envoi du message sur le canal: {:?}", why);
                        }
                    }
                },
                Err(e) => {
                    error!("Erreur lors de la création du token: {:?}", e);
                    if let Err(why) = msg.channel_id.say(&ctx.http, "Une erreur est survenue lors de la génération du lien.").await {
                        error!("Erreur lors de l'envoi du message: {:?}", why);
                    }
                }
            }
        },
        Ok(None) => {
            if let Err(why) = msg.channel_id.say(&ctx.http, "Vous devez d'abord créer un profil avec la commande ^^NP").await {
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