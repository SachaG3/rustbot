mod commands;
mod database;
mod events;
mod utils;

use std::env;
use std::collections::HashSet;
use serenity::model::id::UserId;
use serenity::client::bridge::gateway::ShardManager;
use serenity::framework::StandardFramework;
use serenity::framework::standard::{
    macros::{group, hook},
    CommandResult, DispatchError,
};
use serenity::model::channel::Message;
use serenity::prelude::*;
use tokio::sync::Mutex;
use std::sync::Arc;

use commands::{
    basic::*,
    profile::*,
    games::*,
    spam::*,
    valorant::*,
};

use dotenv::dotenv;
use sqlx::mysql::MySqlPoolOptions;
use tracing::{error, info};

use crate::database::DatabasePool;
use crate::events::Handler;

pub struct ShardManagerContainer;

impl TypeMapKey for ShardManagerContainer {
    type Value = Arc<Mutex<ShardManager>>;
}

#[group]
#[commands(help, serverinfo, link, jeux, np, score, juste, usd, bj, rp, rpt, skin, rank)]
struct General;

#[hook]
async fn before(_: &Context, _msg: &Message, command_name: &str) -> bool {
    info!("Commande reçue: {}", command_name);
    true
}

#[hook]
async fn after(_: &Context, _: &Message, command_name: &str, command_result: CommandResult) {
    match command_result {
        Ok(()) => info!("Commande traitée: {}", command_name),
        Err(why) => error!("Erreur lors de l'exécution de la commande {}: {:?}", command_name, why),
    }
}

#[hook]
async fn unknown_command(_: &Context, _: &Message, unknown_command_name: &str) {
    info!("Commande inconnue: {}", unknown_command_name);
}

#[hook]
async fn dispatch_error(ctx: &Context, msg: &Message, error: DispatchError, _: &str) {
    if let DispatchError::Ratelimited(info) = error {
        if info.is_first_try {
            let _ = msg
                .channel_id
                .say(&ctx.http, &format!("Essayez à nouveau dans {} secondes.", info.as_secs()))
                .await;
        }
    }
}

#[tokio::main]
async fn main() {
    // Initialiser la journalisation
    tracing_subscriber::fmt::init();
    
    // Charger les variables d'environnement
    dotenv().ok();
    
    // Récupérer le token Discord depuis les variables d'environnement
    let token = env::var("TOKEN_DISCORD").expect("Token Discord non trouvé");
    
    // Connexion à la base de données
    let database_url = format!(
        "mysql://{}:{}@{}:{}/{}",
        env::var("DB_USER").expect("DB_USER non trouvé"),
        env::var("DB_PASSWORD").expect("DB_PASSWORD non trouvé"),
        env::var("DB_HOST").expect("DB_HOST non trouvé"),
        env::var("DB_PORT").unwrap_or_else(|_| "3306".to_string()),
        env::var("DB_DATABASE").expect("DB_DATABASE non trouvé")
    );
    
    let pool = MySqlPoolOptions::new()
        .max_connections(5)
        .connect(&database_url)
        .await
        .expect("Erreur lors de la connexion à la base de données");
    
    // Configurer le framework de commandes
    let framework = StandardFramework::new()
        .configure(|c| {
            c.prefix("^^")
             .ignore_bots(true)
             .allow_dm(true)
             .with_whitespace(true)
             .case_insensitivity(true)
             .owners(get_owners())
        })
        .before(before)
        .after(after)
        .unrecognised_command(unknown_command)
        .on_dispatch_error(dispatch_error)
        .group(&GENERAL_GROUP);
    
    // Initialiser le client Discord
    let mut client = Client::builder(&token, GatewayIntents::all())
        .event_handler(Handler)
        .framework(framework)
        .await
        .expect("Erreur lors de la création du client");
    
    // Stocker le gestionnaire de shards pour une utilisation ultérieure
    {
        let mut data = client.data.write().await;
        data.insert::<ShardManagerContainer>(client.shard_manager.clone());
        data.insert::<DatabasePool>(Arc::new(pool));
    }
    
    // Démarrer le client
    info!("Démarrage du bot...");
    if let Err(why) = client.start().await {
        error!("Erreur lors du démarrage du client: {:?}", why);
    }
}

fn get_owners() -> HashSet<UserId> {
    let mut owners = HashSet::new();
    
    if let Ok(owner_id) = env::var("OWNER_ID") {
        if let Ok(id) = owner_id.parse::<u64>() {
            owners.insert(UserId(id));
        }
    }
    
    owners
} 