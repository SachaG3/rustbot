use serenity::async_trait;
use serenity::model::channel::Message;
use serenity::model::gateway::Ready;
use serenity::model::event::{ResumedEvent, MessageUpdateEvent};
use serenity::model::guild::{Guild, Member, UnavailableGuild};
use serenity::model::id::{GuildId, ChannelId, MessageId, RoleId};
use serenity::model::guild::Role;
use serenity::model::user::User;
use serenity::prelude::*;
use tracing::{info, error};
use sqlx::{MySql, Pool};

use crate::database::{DatabasePool, add_log, get_user_by_discord_id, new_user, get_guild, add_guild, add_user_to_guild, new_message, new_message_delete, new_message_edit};

pub struct Handler;

#[async_trait]
impl EventHandler for Handler {
    async fn ready(&self, ctx: Context, ready: Ready) {
        info!("Bot connecté en tant que {}!", ready.user.name);
        
        let pool = {
            let data = ctx.data.read().await;
            data.get::<DatabasePool>().expect("Erreur lors de l'obtention du pool de base de données").clone()
        };
        
        if let Err(e) = add_log(&pool, "Bot Status", "Le bot est connecté et prêt.").await {
            error!("Erreur lors de l'ajout d'un log: {:?}", e);
        }
    }
    
    async fn resume(&self, ctx: Context, _: ResumedEvent) {
        let pool = {
            let data = ctx.data.read().await;
            data.get::<DatabasePool>().expect("Erreur lors de l'obtention du pool de base de données").clone()
        };
        
        if let Err(e) = add_log(&pool, "Bot Status", "Le bot a été reconnecté au serveur Discord.").await {
            error!("Erreur lors de l'ajout d'un log: {:?}", e);
        }
    }
    
    async fn guild_create(&self, ctx: Context, guild: Guild, is_new: bool) {
        if !is_new {
            return;
        }
        
        let pool = {
            let data = ctx.data.read().await;
            data.get::<DatabasePool>().expect("Erreur lors de l'obtention du pool de base de données").clone()
        };
        
        for member in guild.members.values() {
            handle_new_member(&pool, member, guild.id.0).await;
        }
        
        if let Err(e) = add_log(&pool, "Bot Status", &format!("Le bot a rejoint le serveur : {}", guild.name)).await {
            error!("Erreur lors de l'ajout d'un log: {:?}", e);
        }
    }
    
    async fn guild_delete(&self, ctx: Context, incomplete: UnavailableGuild, _full: Option<Guild>) {
        let pool = {
            let data = ctx.data.read().await;
            data.get::<DatabasePool>().expect("Erreur lors de l'obtention du pool de base de données").clone()
        };
        
        if let Err(e) = add_log(&pool, "Bot Status", &format!("Le bot a quitté le serveur : ID {}", incomplete.id)).await {
            error!("Erreur lors de l'ajout d'un log: {:?}", e);
        }
    }
    
    async fn guild_member_addition(&self, ctx: Context, new_member: Member) {
        let pool = {
            let data = ctx.data.read().await;
            data.get::<DatabasePool>().expect("Erreur lors de l'obtention du pool de base de données").clone()
        };
        
        handle_new_member(&pool, &new_member, new_member.guild_id.0).await;
    }
    
    async fn message(&self, ctx: Context, msg: Message) {
        if msg.author.bot {
            return;
        }
        
        let pool = {
            let data = ctx.data.read().await;
            data.get::<DatabasePool>().expect("Erreur lors de l'obtention du pool de base de données").clone()
        };
        
        let guild_id = msg.guild_id.map_or(0, |id| id.0);
        let user_id = msg.author.id.0;
        
        match get_user_by_discord_id(&pool, user_id).await {
            Ok(Some(user)) => {
                if let Err(e) = new_message(&pool, user.id, &msg.content, guild_id).await {
                    error!("Erreur lors de l'enregistrement du message: {:?}", e);
                }
            },
            Ok(None) => {
                match new_user(&pool, user_id, &msg.author.name).await {
                    Ok(user_id) => {
                        if let Err(e) = new_message(&pool, user_id, &msg.content, guild_id).await {
                            error!("Erreur lors de l'enregistrement du message: {:?}", e);
                        }
                    },
                    Err(e) => error!("Erreur lors de la création de l'utilisateur: {:?}", e),
                }
            },
            Err(e) => error!("Erreur lors de la recherche de l'utilisateur: {:?}", e),
        }
        
        if let Some(guild_id) = msg.guild_id {
            if let Ok(Some(user)) = get_user_by_discord_id(&pool, user_id).await {
                if let Ok(None) = get_guild(&pool, guild_id.0).await {
                    if let Ok(guild) = guild_id.to_partial_guild(&ctx.http).await {
                        if let Err(e) = add_guild(&pool, guild_id.0, &guild.name).await {
                            error!("Erreur lors de l'ajout de la guilde: {:?}", e);
                        }
                    }
                }
                
                if let Err(e) = add_user_to_guild(&pool, user.id, guild_id.0).await {
                    error!("Erreur lors de l'ajout de l'utilisateur à la guilde: {:?}", e);
                }
            }
        }
    }
    
    async fn message_delete(&self, ctx: Context, channel_id: ChannelId, deleted_message_id: MessageId, guild_id: Option<GuildId>) {
        let pool = {
            let data = ctx.data.read().await;
            data.get::<DatabasePool>().expect("Erreur lors de l'obtention du pool de base de données").clone()
        };
        
        if let Some(msg) = ctx.cache.message(channel_id, deleted_message_id) {
            if msg.author.bot {
                return;
            }
            
            let guild_id = guild_id.map_or(0, |id| id.0);
            
            if let Ok(Some(user)) = get_user_by_discord_id(&pool, msg.author.id.0).await {
                if let Err(e) = new_message_delete(&pool, user.id, &msg.content, guild_id).await {
                    error!("Erreur lors de l'enregistrement du message supprimé: {:?}", e);
                }
            }
        }
    }
    
    async fn message_update(&self, ctx: Context, old_if_available: Option<Message>, new: Option<Message>, _event: MessageUpdateEvent) {
        if let Some(new_msg) = new {
            if new_msg.author.bot {
                return;
            }
            
            if let Some(old) = old_if_available {
                if old.content == new_msg.content {
                    return;
                }
                
                let pool = {
                    let data = ctx.data.read().await;
                    data.get::<DatabasePool>().expect("Erreur lors de l'obtention du pool de base de données").clone()
                };
                
                let guild_id = new_msg.guild_id.map_or(0, |id| id.0);
                
                if let Ok(Some(user)) = get_user_by_discord_id(&pool, new_msg.author.id.0).await {
                    if let Err(e) = new_message_edit(&pool, user.id, &old.content, &new_msg.content, guild_id).await {
                        error!("Erreur lors de l'enregistrement du message modifié: {:?}", e);
                    }
                }
            }
        }
    }
    
    async fn guild_role_create(&self, ctx: Context, new: Role) {
        let pool = {
            let data = ctx.data.read().await;
            data.get::<DatabasePool>().expect("Erreur lors de l'obtention du pool de base de données").clone()
        };
        
        if let Err(e) = add_log(&pool, "Role Status", &format!("Un rôle a été créé: {}", new.name)).await {
            error!("Erreur lors de l'ajout d'un log: {:?}", e);
        }
    }
    
    async fn guild_role_delete(&self, ctx: Context, _guild_id: GuildId, removed_role_id: RoleId, removed_role_data_if_available: Option<Role>) {
        let pool = {
            let data = ctx.data.read().await;
            data.get::<DatabasePool>().expect("Erreur lors de l'obtention du pool de base de données").clone()
        };
        
        let role_name = removed_role_data_if_available
            .map(|r| r.name)
            .unwrap_or_else(|| format!("ID: {}", removed_role_id));
        
        if let Err(e) = add_log(&pool, "Role Status", &format!("Un rôle a été supprimé: {}", role_name)).await {
            error!("Erreur lors de l'ajout d'un log: {:?}", e);
        }
    }
    
    async fn guild_role_update(&self, ctx: Context, _old_data_if_available: Option<Role>, new: Role) {
        let pool = {
            let data = ctx.data.read().await;
            data.get::<DatabasePool>().expect("Erreur lors de l'obtention du pool de base de données").clone()
        };
        
        if let Err(e) = add_log(&pool, "Role Status", &format!("Un rôle a été mis à jour: {}", new.name)).await {
            error!("Erreur lors de l'ajout d'un log: {:?}", e);
        }
    }
    
    async fn guild_ban_addition(&self, ctx: Context, guild_id: GuildId, banned_user: User) {
        let pool = {
            let data = ctx.data.read().await;
            data.get::<DatabasePool>().expect("Erreur lors de l'obtention du pool de base de données").clone()
        };
        
        let guild_name = guild_id.to_partial_guild(&ctx.http).await
            .map(|g| g.name)
            .unwrap_or_else(|_| format!("ID: {}", guild_id));
        
        if let Err(e) = add_log(&pool, "Ban Status", &format!("{} a été banni du serveur: {}", banned_user.name, guild_name)).await {
            error!("Erreur lors de l'ajout d'un log: {:?}", e);
        }
    }
    
    async fn guild_ban_removal(&self, ctx: Context, guild_id: GuildId, unbanned_user: User) {
        let pool = {
            let data = ctx.data.read().await;
            data.get::<DatabasePool>().expect("Erreur lors de l'obtention du pool de base de données").clone()
        };
        
        let guild_name = guild_id.to_partial_guild(&ctx.http).await
            .map(|g| g.name)
            .unwrap_or_else(|_| format!("ID: {}", guild_id));
        
        if let Err(e) = add_log(&pool, "Ban Status", &format!("{} a été réintégré sur le serveur: {}", unbanned_user.name, guild_name)).await {
            error!("Erreur lors de l'ajout d'un log: {:?}", e);
        }
    }
}

async fn handle_new_member(pool: &Pool<MySql>, member: &Member, guild_id: u64) {
    let user_id = member.user.id.0;
    
    match get_user_by_discord_id(pool, user_id).await {
        Ok(Some(user)) => {
            if let Err(e) = add_log(pool, "Ajout utilisateur", &format!("L'utilisateur {} existe déjà dans la base de données.", user_id)).await {
                error!("Erreur lors de l'ajout d'un log: {:?}", e);
            }
            
            if let Ok(None) = get_guild(pool, guild_id).await {
                if let Err(e) = add_guild(pool, guild_id, &format!("ID: {}", guild_id)).await {
                    error!("Erreur lors de l'ajout de la guilde: {:?}", e);
                }
            }
            
            if let Err(e) = add_user_to_guild(pool, user.id, guild_id).await {
                error!("Erreur lors de l'ajout de l'utilisateur à la guilde: {:?}", e);
            }
        },
        Ok(None) => {
            if let Err(e) = add_log(pool, "Ajout utilisateur", &format!("Ajout d'un nouvel utilisateur: {}", user_id)).await {
                error!("Erreur lors de l'ajout d'un log: {:?}", e);
            }
            
            match new_user(pool, user_id, &member.user.name).await {
                Ok(user_id) => {
                    if let Ok(None) = get_guild(pool, guild_id).await {
                        if let Err(e) = add_guild(pool, guild_id, &format!("ID: {}", guild_id)).await {
                            error!("Erreur lors de l'ajout de la guilde: {:?}", e);
                        }
                    }
                    
                    if let Err(e) = add_user_to_guild(pool, user_id, guild_id).await {
                        error!("Erreur lors de l'ajout de l'utilisateur à la guilde: {:?}", e);
                    }
                },
                Err(e) => error!("Erreur lors de la création de l'utilisateur: {:?}", e),
            }
        },
        Err(e) => error!("Erreur lors de la recherche de l'utilisateur: {:?}", e),
    }
} 