use chrono::{Datelike, Duration, NaiveDate, NaiveDateTime, Timelike, Weekday};
use rand::{thread_rng, Rng};
use serenity::framework::standard::{macros::command, Args, CommandResult};
use serenity::model::channel::Message;
use serenity::model::id::ChannelId;
use serenity::prelude::*;
use sqlx::{MySql, Pool, Row};
use std::env;
use std::sync::atomic::{AtomicBool, Ordering};

use crate::database::{add_cat_memory, get_cat_by_id, get_user_by_discord_id, DatabasePool};
use crate::time::{paris_now, paris_now_naive, paris_today};

const DEFAULT_CAT_CHANNEL_ID: u64 = 1366197497793347685;
const CHALLENGE_DURATION_HOURS: i64 = 3;
const CHALLENGE_COOLDOWN_HOURS: i64 = 6;
const RANDOM_CHALLENGE_CHANCE_PERCENT: i32 = 12;
static RETENTION_SCHEDULER_STARTED: AtomicBool = AtomicBool::new(false);

#[derive(Clone, Copy, Debug)]
pub struct ChallengeTemplate {
    pub key: &'static str,
    pub title: &'static str,
    pub intro: &'static str,
    pub action: &'static str,
    pub success: &'static str,
    pub fallback: &'static str,
}

#[derive(Debug)]
struct ActiveChallenge {
    id: i64,
    channel_id: u64,
    kind: String,
    title: String,
    target: i32,
    progress: i32,
    ends_at: NaiveDateTime,
}

#[derive(Debug)]
struct ActiveExpedition {
    cat_id: i32,
    duration_kind: String,
    started_at: NaiveDateTime,
    ends_at: NaiveDateTime,
}

#[derive(Clone, Copy)]
struct Decoration {
    key: &'static str,
    name: &'static str,
    description: &'static str,
}

const DECORATIONS: &[Decoration] = &[
    Decoration {
        key: "coussin_bleu",
        name: "Coussin bleu",
        description: "Un coussin moelleux près de la fenêtre.",
    },
    Decoration {
        key: "sac_voyage",
        name: "Sac de voyage",
        description: "Un sac ouvert devenu un couchage très apprécié.",
    },
    Decoration {
        key: "fanion_solidaire",
        name: "Fanion solidaire",
        description: "Le souvenir d'un défi réussi ensemble.",
    },
    Decoration {
        key: "guirlande_solidaire",
        name: "Guirlande solidaire",
        description: "Une guirlande gagnée en prenant part à la vie du serveur.",
    },
    Decoration {
        key: "cadre_amical",
        name: "Cadre amical",
        description: "Une photo des maisons visitées cette semaine.",
    },
    Decoration {
        key: "plante_sure",
        name: "Plante non toxique",
        description: "Un peu de verdure choisie pour rester sans danger.",
    },
    Decoration {
        key: "couverture_etoilee",
        name: "Couverture étoilée",
        description: "Une couverture rapportée d'une promenade nocturne.",
    },
    Decoration {
        key: "arbre_chat",
        name: "Arbre à chat",
        description: "Un poste d'observation confortable.",
    },
    Decoration {
        key: "lanterne_douce",
        name: "Lanterne douce",
        description: "Une lumière calme pour les retours d'expédition.",
    },
];

const EXPEDITION_DECORATION_KEYS: &[&str] = &[
    "plante_sure",
    "couverture_etoilee",
    "arbre_chat",
    "lanterne_douce",
];

pub fn week_start(date: NaiveDate) -> NaiveDate {
    date - Duration::days(date.weekday().num_days_from_monday() as i64)
}

pub fn challenge_template_for_roll(roll: usize) -> ChallengeTemplate {
    const TEMPLATES: &[ChallengeTemplate] = &[
        ChallengeTemplate {
            key: "chien_turbulent",
            title: "Le chien turbulent",
            intro: "Un chien très enthousiaste s'approche du refuge. Les chats peuvent unir leur assurance pour lui faire comprendre qu'il vaut mieux garder ses distances.",
            action: "ajoute son courage à la défense paisible du refuge",
            success: "Les chats restent groupés et parfaitement calmes. Impressionné, le chien comprend le message et repart avec son humain.",
            fallback: "Les chats rentrent tranquillement à l'intérieur pendant que les bénévoles ferment le portail. Le chien repart ensuite avec son humain. Tout le monde est en sécurité.",
        },
        ChallengeTemplate {
            key: "carton_geant",
            title: "Le carton géant",
            intro: "Un immense carton bloque le passage. Les chats vont devoir l'explorer ensemble pour retrouver l'entrée de la maison.",
            action: "inspecte le carton et indique un passage aux autres",
            success: "À force d'exploration, les chats trouvent le passage parfait et transforment le carton en salle de jeu collective.",
            fallback: "Le carton résiste encore un peu. Les chats choisissent un autre passage confortable et les bénévoles le déplaceront plus tard.",
        },
        ChallengeTemplate {
            key: "orage_solidaire",
            title: "L'orage solidaire",
            intro: "Un orage approche. La communauté prépare des coins douillets pour que tous les chats puissent se reposer au calme.",
            action: "aide à préparer un abri chaleureux",
            success: "Chaque chat trouve une couverture et une place rassurante avant les premières gouttes.",
            fallback: "Les bénévoles terminent les derniers abris. Les chats sont déjà installés au sec et observent la pluie derrière les fenêtres.",
        },
        ChallengeTemplate {
            key: "pelote_geante",
            title: "La pelote géante",
            intro: "Une pelote démesurée s'est déroulée dans toute la maison. Il faut coopérer pour libérer le passage sans perdre le fil.",
            action: "suit un fil et aide à démêler la pelote",
            success: "Les chats coordonnent leurs découvertes et la pelote devient un magnifique parcours de jeu.",
            fallback: "La pelote reste un peu emmêlée, mais un chemin confortable est dégagé et chacun peut circuler tranquillement.",
        },
        ChallengeTemplate {
            key: "pique_nique",
            title: "Le pique-nique du refuge",
            intro: "Le refuge prépare un pique-nique calme. Les chats peuvent aider à choisir les meilleurs coussins et coins ombragés.",
            action: "repère un endroit accueillant pour le pique-nique",
            success: "Grâce à toutes les propositions, le pique-nique est installé dans un endroit parfait et chacun profite du moment.",
            fallback: "Le pique-nique est déplacé à l'intérieur, dans une pièce calme et confortable. La rencontre reste un joli moment partagé.",
        },
    ];
    TEMPLATES[roll % TEMPLATES.len()]
}

pub fn challenge_resolution(template: &ChallengeTemplate, progress: i32, target: i32) -> String {
    if progress >= target {
        format!("✅ **{} terminé !**\n{}", template.title, template.success)
    } else {
        format!(
            "🏠 **{} est terminé.**\n{}",
            template.title, template.fallback
        )
    }
}

pub fn expedition_hours(kind: &str) -> Option<i64> {
    match kind.to_lowercase().as_str() {
        "courte" => Some(2),
        "moyenne" => Some(6),
        "longue" => Some(12),
        _ => None,
    }
}

pub fn split_discord_messages(header: &str, entries: &[String], limit: usize) -> Vec<String> {
    let mut messages = Vec::new();
    let mut current = header.to_string();

    for entry in entries {
        let separator = if current.is_empty() { "" } else { "\n" };
        if current.chars().count() + separator.chars().count() + entry.chars().count() > limit
            && !current.is_empty()
        {
            messages.push(current);
            current = format!("{} (suite)\n{}", header, entry);
        } else {
            current.push_str(separator);
            current.push_str(entry);
        }
    }

    if !current.is_empty() {
        messages.push(current);
    }
    messages
}

pub fn safe_discord_text(value: &str) -> String {
    value
        .replace('\\', "")
        .replace('@', "@\u{200b}")
        .replace('*', "\\*")
        .replace('_', "\\_")
        .replace('~', "\\~")
        .replace('`', "ˋ")
        .replace('|', "\\|")
}

fn configured_cat_channel_id() -> u64 {
    env::var("CAT_CHANNEL_ID")
        .ok()
        .and_then(|value| value.parse::<u64>().ok())
        .unwrap_or(DEFAULT_CAT_CHANNEL_ID)
}

pub fn challenge_roll_bucket(now: NaiveDateTime) -> i64 {
    now.and_utc().timestamp() / (15 * 60)
}

pub async fn ensure_retention_schema(pool: &Pool<MySql>) -> Result<(), sqlx::Error> {
    let statements = [
        "CREATE TABLE IF NOT EXISTS cat_activity (
            id bigint(20) NOT NULL AUTO_INCREMENT,
            user_id int(11) NOT NULL,
            activity_type varchar(40) NOT NULL,
            created_at datetime NOT NULL,
            PRIMARY KEY (id),
            KEY idx_cat_activity_user_date (user_id, created_at),
            KEY idx_cat_activity_type_date (activity_type, created_at)
        ) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_unicode_ci",
        "CREATE TABLE IF NOT EXISTS cat_expeditions (
            id bigint(20) NOT NULL AUTO_INCREMENT,
            cat_id int(11) NOT NULL,
            user_id int(11) NOT NULL,
            duration_kind varchar(20) NOT NULL,
            started_at datetime NOT NULL,
            ends_at datetime NOT NULL,
            completed_at datetime DEFAULT NULL,
            reward_description varchar(255) DEFAULT NULL,
            PRIMARY KEY (id),
            KEY idx_cat_expeditions_user (user_id, completed_at),
            KEY idx_cat_expeditions_cat (cat_id, completed_at)
        ) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_unicode_ci",
        "CREATE TABLE IF NOT EXISTS cat_user_decorations (
            user_id int(11) NOT NULL,
            decoration_key varchar(60) NOT NULL,
            equipped tinyint(1) NOT NULL DEFAULT 0,
            unlocked_at datetime NOT NULL,
            PRIMARY KEY (user_id, decoration_key),
            KEY idx_cat_decor_equipped (user_id, equipped)
        ) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_unicode_ci",
        "CREATE TABLE IF NOT EXISTS cat_weekly_mission_claims (
            user_id int(11) NOT NULL,
            week_start date NOT NULL,
            mission_key varchar(60) NOT NULL,
            decoration_key varchar(60) NOT NULL,
            claimed_at datetime NOT NULL,
            PRIMARY KEY (user_id, week_start, mission_key)
        ) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_unicode_ci",
        "CREATE TABLE IF NOT EXISTS cat_community_challenges (
            id bigint(20) NOT NULL AUTO_INCREMENT,
            channel_id bigint(20) UNSIGNED NOT NULL,
            challenge_kind varchar(60) NOT NULL,
            title varchar(120) NOT NULL,
            target int(11) NOT NULL,
            progress int(11) NOT NULL DEFAULT 0,
            started_at datetime NOT NULL,
            ends_at datetime NOT NULL,
            status varchar(20) NOT NULL DEFAULT 'active',
            resolution text DEFAULT NULL,
            PRIMARY KEY (id),
            KEY idx_cat_challenge_status (status, ends_at),
            KEY idx_cat_challenge_started (started_at)
        ) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_unicode_ci",
        "CREATE TABLE IF NOT EXISTS cat_community_participants (
            challenge_id bigint(20) NOT NULL,
            user_id int(11) NOT NULL,
            cat_id int(11) NOT NULL,
            contribution int(11) NOT NULL,
            joined_at datetime NOT NULL,
            PRIMARY KEY (challenge_id, user_id),
            KEY idx_cat_challenge_participants (challenge_id)
        ) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_unicode_ci",
        "CREATE TABLE IF NOT EXISTS cat_scheduled_runs (
            run_key varchar(100) NOT NULL,
            ran_at datetime NOT NULL,
            PRIMARY KEY (run_key)
        ) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_unicode_ci",
    ];

    for statement in statements {
        sqlx::query(statement).execute(pool).await?;
    }
    Ok(())
}

pub async fn record_activity(
    pool: &Pool<MySql>,
    user_id: i64,
    activity_type: &str,
) -> Result<(), sqlx::Error> {
    sqlx::query("INSERT INTO cat_activity (user_id, activity_type, created_at) VALUES (?, ?, ?)")
        .bind(user_id as i32)
        .bind(activity_type)
        .bind(paris_now_naive())
        .execute(pool)
        .await?;
    Ok(())
}

async fn unlock_decoration(
    pool: &Pool<MySql>,
    user_id: i64,
    decoration_key: &str,
) -> Result<bool, sqlx::Error> {
    let result = sqlx::query(
        "INSERT IGNORE INTO cat_user_decorations (user_id, decoration_key, equipped, unlocked_at) VALUES (?, ?, 0, ?)",
    )
    .bind(user_id as i32)
    .bind(decoration_key)
    .bind(paris_now_naive())
    .execute(pool)
    .await?;
    Ok(result.rows_affected() > 0)
}

pub async fn get_equipped_decorations(
    pool: &Pool<MySql>,
    user_id: i64,
) -> Result<Vec<String>, sqlx::Error> {
    let rows = sqlx::query(
        "SELECT decoration_key FROM cat_user_decorations WHERE user_id = ? AND equipped = 1 ORDER BY unlocked_at ASC",
    )
    .bind(user_id as i32)
    .fetch_all(pool)
    .await?;
    Ok(rows
        .into_iter()
        .filter_map(|row| {
            let key: String = row.get("decoration_key");
            DECORATIONS
                .iter()
                .find(|decoration| decoration.key == key)
                .map(|decoration| decoration.name.to_string())
        })
        .collect())
}

pub async fn is_cat_on_expedition(pool: &Pool<MySql>, cat_id: i32) -> Result<bool, sqlx::Error> {
    let row = sqlx::query(
        "SELECT 1 FROM cat_expeditions WHERE cat_id = ? AND completed_at IS NULL LIMIT 1",
    )
    .bind(cat_id)
    .fetch_optional(pool)
    .await?;
    Ok(row.is_some())
}

async fn active_expeditions_for_user(
    pool: &Pool<MySql>,
    user_id: i64,
) -> Result<Vec<ActiveExpedition>, sqlx::Error> {
    let rows = sqlx::query(
        "SELECT id, cat_id, duration_kind, started_at, ends_at FROM cat_expeditions WHERE user_id = ? AND completed_at IS NULL ORDER BY ends_at ASC",
    )
    .bind(user_id as i32)
    .fetch_all(pool)
    .await?;
    Ok(rows
        .into_iter()
        .map(|row| ActiveExpedition {
            cat_id: row.get("cat_id"),
            duration_kind: row.get("duration_kind"),
            started_at: row.get("started_at"),
            ends_at: row.get("ends_at"),
        })
        .collect())
}

fn decoration_by_key(key: &str) -> Option<Decoration> {
    DECORATIONS
        .iter()
        .find(|decoration| decoration.key == key)
        .copied()
}

#[command]
#[description = "Affiche les missions chats de la semaine"]
pub async fn missions(ctx: &Context, msg: &Message) -> CommandResult {
    let pool = {
        let data = ctx.data.read().await;
        data.get::<DatabasePool>()
            .expect("Impossible d'obtenir le pool")
            .clone()
    };
    let user = match get_user_by_discord_id(&pool, msg.author.id.0).await? {
        Some(user) => user,
        None => {
            msg.channel_id
                .say(&ctx.http, "Crée d'abord ton profil avec `^^np`.")
                .await?;
            return Ok(());
        }
    };
    let start = week_start(paris_today());
    let start_at = start.and_hms_opt(0, 0, 0).unwrap();
    let rows = sqlx::query(
        "SELECT activity_type, COUNT(*) AS total, COUNT(DISTINCT DATE(created_at)) AS distinct_days
         FROM cat_activity WHERE user_id = ? AND created_at >= ? GROUP BY activity_type",
    )
    .bind(user.id as i32)
    .bind(start_at)
    .fetch_all(&*pool)
    .await?;

    let mut daily_days = 0i64;
    let mut expeditions = 0i64;
    let mut challenges = 0i64;
    let mut visits = 0i64;
    for row in rows {
        let activity: String = row.get("activity_type");
        let total: i64 = row.get("total");
        let distinct_days: i64 = row.get("distinct_days");
        match activity.as_str() {
            "daily_cat" => daily_days = distinct_days,
            "expedition" => expeditions = total,
            "challenge" => challenges = total,
            "visit" => visits = total,
            _ => {}
        }
    }

    let mission_data = [
        (
            "regularite",
            "Récupérer un Daily Cat pendant 3 jours différents",
            daily_days,
            3,
            "coussin_bleu",
        ),
        (
            "exploration",
            "Lancer une expédition",
            expeditions,
            1,
            "sac_voyage",
        ),
        (
            "entraide",
            "Participer à un défi communautaire",
            challenges,
            1,
            "guirlande_solidaire",
        ),
        (
            "visite",
            "Visiter la maison d'un autre membre",
            visits,
            1,
            "cadre_amical",
        ),
    ];
    let mut newly_unlocked = Vec::new();
    let mut lines = Vec::new();
    for (key, label, progress, target, reward) in mission_data {
        let completed = progress >= target;
        if completed {
            let claim = sqlx::query(
                "INSERT IGNORE INTO cat_weekly_mission_claims (user_id, week_start, mission_key, decoration_key, claimed_at) VALUES (?, ?, ?, ?, ?)",
            )
            .bind(user.id as i32)
            .bind(start)
            .bind(key)
            .bind(reward)
            .bind(paris_now_naive())
            .execute(&*pool)
            .await?;
            if claim.rows_affected() > 0 && unlock_decoration(&pool, user.id, reward).await? {
                if let Some(decoration) = decoration_by_key(reward) {
                    newly_unlocked.push(decoration.name);
                }
            }
        }
        lines.push(format!(
            "{} **{}** — {}/{}",
            if completed { "✅" } else { "▫️" },
            label,
            progress.min(target),
            target
        ));
    }
    let mut response = format!(
        "📋 **Missions de la semaine**\n{}\n\nLes missions reviennent chaque lundi, sans série à perdre.",
        lines.join("\n")
    );
    if !newly_unlocked.is_empty() {
        response += &format!(
            "\n\n🎁 Nouvelle{} décoration{} : **{}**",
            if newly_unlocked.len() > 1 { "s" } else { "" },
            if newly_unlocked.len() > 1 { "s" } else { "" },
            newly_unlocked.join(", ")
        );
    }
    msg.channel_id.say(&ctx.http, response).await?;
    Ok(())
}

#[command]
#[description = "Envoie un chat en promenade : ^^expedition <id> <courte|moyenne|longue>"]
pub async fn expedition(ctx: &Context, msg: &Message, mut args: Args) -> CommandResult {
    let cat_id = match args.single::<i32>() {
        Ok(id) => id,
        Err(_) => {
            msg.channel_id
                .say(
                    &ctx.http,
                    "Usage : `^^expedition <id_chat> <courte|moyenne|longue>`",
                )
                .await?;
            return Ok(());
        }
    };
    let kind = args.single::<String>().unwrap_or_default().to_lowercase();
    let hours = match expedition_hours(&kind) {
        Some(hours) => hours,
        None => {
            msg.channel_id
                .say(
                    &ctx.http,
                    "Choisis une promenade `courte`, `moyenne` ou `longue`.",
                )
                .await?;
            return Ok(());
        }
    };
    let pool = {
        let data = ctx.data.read().await;
        data.get::<DatabasePool>().expect("Pool absent").clone()
    };
    let user = match get_user_by_discord_id(&pool, msg.author.id.0).await? {
        Some(user) => user,
        None => {
            msg.channel_id
                .say(&ctx.http, "Crée d'abord ton profil avec `^^np`.")
                .await?;
            return Ok(());
        }
    };
    let cat = match get_cat_by_id(&pool, cat_id).await? {
        Some(cat) if cat.user_id == user.id as i32 && cat.location == "home" => cat,
        _ => {
            msg.channel_id
                .say(&ctx.http, "Ce chat ne vit pas chez toi.")
                .await?;
            return Ok(());
        }
    };
    if is_cat_on_expedition(&pool, cat_id).await? {
        msg.channel_id
            .say(
                &ctx.http,
                "Ce chat est déjà en promenade et reviendra toujours sain et sauf.",
            )
            .await?;
        return Ok(());
    }
    let now = paris_now_naive();
    let ends_at = now + Duration::hours(hours);
    sqlx::query(
        "INSERT INTO cat_expeditions (cat_id, user_id, duration_kind, started_at, ends_at) VALUES (?, ?, ?, ?, ?)",
    )
    .bind(cat_id)
    .bind(user.id as i32)
    .bind(&kind)
    .bind(now)
    .bind(ends_at)
    .execute(&*pool)
    .await?;
    record_activity(&pool, user.id, "expedition").await?;
    msg.channel_id
        .say(
            &ctx.http,
            format!(
                "🎒 **{}** part pour une promenade {}. Retour prévu vers **{}**. Il restera en sécurité pendant toute l'exploration.",
                cat.name,
                kind,
                ends_at.format("%d/%m à %H:%M")
            ),
        )
        .await?;
    Ok(())
}

#[command]
#[description = "Affiche les chats actuellement en expédition"]
pub async fn expeditions(ctx: &Context, msg: &Message) -> CommandResult {
    let pool = {
        let data = ctx.data.read().await;
        data.get::<DatabasePool>().expect("Pool absent").clone()
    };
    let user = match get_user_by_discord_id(&pool, msg.author.id.0).await? {
        Some(user) => user,
        None => {
            msg.channel_id
                .say(&ctx.http, "Aucune expédition en cours.")
                .await?;
            return Ok(());
        }
    };
    let expeditions = active_expeditions_for_user(&pool, user.id).await?;
    if expeditions.is_empty() {
        msg.channel_id
            .say(&ctx.http, "Aucune expédition en cours.")
            .await?;
        return Ok(());
    }
    let now = paris_now_naive();
    let lines: Vec<String> = expeditions
        .into_iter()
        .map(|expedition| {
            let state = if expedition.ends_at <= now {
                format!("retour disponible avec `^^retour {}`", expedition.cat_id)
            } else {
                format!("retour vers {}", expedition.ends_at.format("%d/%m %H:%M"))
            };
            format!(
                "• Chat #{} — promenade {} — {} (départ {})",
                expedition.cat_id,
                expedition.duration_kind,
                state,
                expedition.started_at.format("%d/%m %H:%M")
            )
        })
        .collect();
    msg.channel_id
        .say(
            &ctx.http,
            format!("🎒 **Expéditions**\n{}", lines.join("\n")),
        )
        .await?;
    Ok(())
}

#[command]
#[description = "Accueille un chat revenu d'expédition : ^^retour <id_chat>"]
pub async fn retour(ctx: &Context, msg: &Message, mut args: Args) -> CommandResult {
    let cat_id = match args.single::<i32>() {
        Ok(id) => id,
        Err(_) => {
            msg.channel_id
                .say(&ctx.http, "Usage : `^^retour <id_chat>`")
                .await?;
            return Ok(());
        }
    };
    let pool = {
        let data = ctx.data.read().await;
        data.get::<DatabasePool>().expect("Pool absent").clone()
    };
    let user = match get_user_by_discord_id(&pool, msg.author.id.0).await? {
        Some(user) => user,
        None => return Ok(()),
    };
    let row = sqlx::query(
        "SELECT id, ends_at FROM cat_expeditions WHERE cat_id = ? AND user_id = ? AND completed_at IS NULL ORDER BY id DESC LIMIT 1",
    )
    .bind(cat_id)
    .bind(user.id as i32)
    .fetch_optional(&*pool)
    .await?;
    let row = match row {
        Some(row) => row,
        None => {
            msg.channel_id
                .say(&ctx.http, "Aucune expédition active pour ce chat.")
                .await?;
            return Ok(());
        }
    };
    let expedition_id: i64 = row.get("id");
    let ends_at: NaiveDateTime = row.get("ends_at");
    let now = paris_now_naive();
    if ends_at > now {
        let minutes = ends_at.signed_duration_since(now).num_minutes().max(1);
        msg.channel_id
            .say(
                &ctx.http,
                format!(
                    "Le chat profite encore de sa promenade. Retour dans environ **{} minutes**.",
                    minutes
                ),
            )
            .await?;
        return Ok(());
    }
    let decoration_key =
        EXPEDITION_DECORATION_KEYS[(expedition_id as usize) % EXPEDITION_DECORATION_KEYS.len()];
    let decoration = decoration_by_key(decoration_key).expect("Décoration d'expédition inconnue");
    let reward = format!("{} — {}", decoration.name, decoration.description);
    let result = sqlx::query(
        "UPDATE cat_expeditions SET completed_at = ?, reward_description = ? WHERE id = ? AND completed_at IS NULL",
    )
    .bind(now)
    .bind(&reward)
    .bind(expedition_id)
    .execute(&*pool)
    .await?;
    if result.rows_affected() == 0 {
        msg.channel_id
            .say(&ctx.http, "Ce retour a déjà été accueilli.")
            .await?;
        return Ok(());
    }
    unlock_decoration(&pool, user.id, decoration.key).await?;
    let cat = get_cat_by_id(&pool, cat_id)
        .await?
        .ok_or("Chat introuvable")?;
    add_cat_memory(
        &pool,
        cat_id,
        Some(user.id),
        "expedition",
        &format!(
            "{} est revenu d'une promenade avec {}.",
            cat.name, decoration.name
        ),
    )
    .await
    .ok();
    msg.channel_id
        .say(
            &ctx.http,
            format!(
                "🏡 **{} est rentré sain et sauf !**\nIl rapporte **{}**.\n{}",
                cat.name, decoration.name, decoration.description
            ),
        )
        .await?;
    Ok(())
}

#[command]
#[description = "Affiche les décorations débloquées pour la maison"]
pub async fn decorations(ctx: &Context, msg: &Message) -> CommandResult {
    let pool = {
        let data = ctx.data.read().await;
        data.get::<DatabasePool>().expect("Pool absent").clone()
    };
    let user = match get_user_by_discord_id(&pool, msg.author.id.0).await? {
        Some(user) => user,
        None => return Ok(()),
    };
    let rows = sqlx::query(
        "SELECT decoration_key, equipped FROM cat_user_decorations WHERE user_id = ? ORDER BY unlocked_at ASC",
    )
    .bind(user.id as i32)
    .fetch_all(&*pool)
    .await?;
    if rows.is_empty() {
        msg.channel_id.say(&ctx.http, "Tu n'as pas encore de décoration. Les missions, expéditions et défis en débloquent.").await?;
        return Ok(());
    }
    let lines: Vec<String> = rows
        .into_iter()
        .filter_map(|row| {
            let key: String = row.get("decoration_key");
            let equipped: i8 = row.get("equipped");
            decoration_by_key(&key).map(|decoration| {
                format!(
                    "{} `{}` — **{}** : {}",
                    if equipped != 0 { "✅" } else { "▫️" },
                    key,
                    decoration.name,
                    decoration.description
                )
            })
        })
        .collect();
    msg.channel_id
        .say(
            &ctx.http,
            format!("🪴 **Décorations**\n{}", lines.join("\n")),
        )
        .await?;
    Ok(())
}

#[command]
#[description = "Installe une décoration : ^^decorer <clé>"]
pub async fn decorer(ctx: &Context, msg: &Message, mut args: Args) -> CommandResult {
    set_decoration_state(ctx, msg, args.single::<String>().unwrap_or_default(), true).await
}

#[command]
#[description = "Range une décoration : ^^ranger <clé>"]
pub async fn ranger(ctx: &Context, msg: &Message, mut args: Args) -> CommandResult {
    set_decoration_state(ctx, msg, args.single::<String>().unwrap_or_default(), false).await
}

async fn set_decoration_state(
    ctx: &Context,
    msg: &Message,
    key: String,
    equipped: bool,
) -> CommandResult {
    if key.is_empty() {
        msg.channel_id
            .say(&ctx.http, "Indique la clé visible dans `^^decorations`.")
            .await?;
        return Ok(());
    }
    let pool = {
        let data = ctx.data.read().await;
        data.get::<DatabasePool>().expect("Pool absent").clone()
    };
    let user = match get_user_by_discord_id(&pool, msg.author.id.0).await? {
        Some(user) => user,
        None => return Ok(()),
    };
    if equipped {
        let row = sqlx::query(
            "SELECT COUNT(*) AS total FROM cat_user_decorations WHERE user_id = ? AND equipped = 1",
        )
        .bind(user.id as i32)
        .fetch_one(&*pool)
        .await?;
        let total: i64 = row.get("total");
        if total >= 3 {
            msg.channel_id
                .say(
                    &ctx.http,
                    "Ta maison présente déjà trois décorations. Range-en une avant.",
                )
                .await?;
            return Ok(());
        }
    }
    let result = sqlx::query(
        "UPDATE cat_user_decorations SET equipped = ? WHERE user_id = ? AND decoration_key = ?",
    )
    .bind(if equipped { 1i8 } else { 0i8 })
    .bind(user.id as i32)
    .bind(&key)
    .execute(&*pool)
    .await?;
    if result.rows_affected() == 0 {
        msg.channel_id
            .say(&ctx.http, "Cette décoration n'est pas encore débloquée.")
            .await?;
        return Ok(());
    }
    let decoration_name = decoration_by_key(&key)
        .map(|decoration| decoration.name)
        .unwrap_or("Décoration");
    msg.channel_id
        .say(
            &ctx.http,
            if equipped {
                format!(
                    "🏠 **{}** est maintenant installé dans la maison.",
                    decoration_name
                )
            } else {
                format!("📦 **{}** a été rangé avec soin.", decoration_name)
            },
        )
        .await?;
    Ok(())
}

async fn get_active_challenge(pool: &Pool<MySql>) -> Result<Option<ActiveChallenge>, sqlx::Error> {
    let row = sqlx::query(
        "SELECT id, channel_id, challenge_kind, title, target, progress, ends_at FROM cat_community_challenges WHERE status = 'active' ORDER BY id DESC LIMIT 1",
    )
    .fetch_optional(pool)
    .await?;
    Ok(row.map(|row| ActiveChallenge {
        id: row.get("id"),
        channel_id: row.get("channel_id"),
        kind: row.get("challenge_kind"),
        title: row.get("title"),
        target: row.get("target"),
        progress: row.get("progress"),
        ends_at: row.get("ends_at"),
    }))
}

pub async fn has_active_community_challenge(pool: &Pool<MySql>) -> bool {
    matches!(get_active_challenge(pool).await, Ok(Some(_)))
}

async fn start_random_challenge(
    ctx: &Context,
    pool: &Pool<MySql>,
    channel_id: ChannelId,
    force_roll: bool,
) -> Result<bool, sqlx::Error> {
    if get_active_challenge(pool).await?.is_some() {
        return Ok(false);
    }
    let existing_cat_event = sqlx::query("SELECT 1 FROM cat_active_events LIMIT 1")
        .fetch_optional(pool)
        .await?;
    if existing_cat_event.is_some() {
        return Ok(false);
    }
    if !force_roll {
        let roll_key = format!(
            "community_challenge_roll_{}",
            challenge_roll_bucket(paris_now_naive())
        );
        let roll =
            sqlx::query("INSERT IGNORE INTO cat_scheduled_runs (run_key, ran_at) VALUES (?, ?)")
                .bind(roll_key)
                .bind(paris_now_naive())
                .execute(pool)
                .await?;
        if roll.rows_affected() == 0 {
            return Ok(false);
        }
    }
    let cooldown = paris_now_naive() - Duration::hours(CHALLENGE_COOLDOWN_HOURS);
    let recent =
        sqlx::query("SELECT 1 FROM cat_community_challenges WHERE started_at >= ? LIMIT 1")
            .bind(cooldown)
            .fetch_optional(pool)
            .await?;
    if recent.is_some() {
        return Ok(false);
    }
    if !force_roll && thread_rng().gen_range(0..100) >= RANDOM_CHALLENGE_CHANCE_PERCENT {
        return Ok(false);
    }
    let template = challenge_template_for_roll(thread_rng().gen_range(0..5));
    let active_users_row = sqlx::query(
        "SELECT COUNT(DISTINCT user_id) AS total FROM cat_activity WHERE created_at >= ?",
    )
    .bind(paris_now_naive() - Duration::days(7))
    .fetch_one(pool)
    .await?;
    let active_users: i64 = active_users_row.get("total");
    let target = ((active_users.max(2) * 4).min(40)) as i32;
    let now = paris_now_naive();
    sqlx::query(
        "INSERT INTO cat_community_challenges (channel_id, challenge_kind, title, target, progress, started_at, ends_at, status) VALUES (?, ?, ?, ?, 0, ?, ?, 'active')",
    )
    .bind(channel_id.0)
    .bind(template.key)
    .bind(template.title)
    .bind(target)
    .bind(now)
    .bind(now + Duration::hours(CHALLENGE_DURATION_HOURS))
    .execute(pool)
    .await?;
    channel_id
        .say(
            &ctx.http,
            format!(
                "🤝 **Nouveau défi communautaire : {}**\n{}\n\nUtilisez `^^participer_defi <id_chat>` pendant les trois prochaines heures. Les chats ne risquent rien : ils peuvent toujours rentrer se reposer en sécurité.",
                template.title, template.intro
            ),
        )
        .await
        .ok();
    Ok(true)
}

pub async fn record_daily_cat_and_roll_challenge(
    ctx: &Context,
    msg: &Message,
    pool: &Pool<MySql>,
    user_id: i64,
) {
    record_activity(pool, user_id, "daily_cat").await.ok();
    if msg.guild_id.is_some() {
        start_random_challenge(ctx, pool, msg.channel_id, false)
            .await
            .ok();
    }
}

#[command]
#[description = "Affiche le défi communautaire en cours"]
pub async fn defi(ctx: &Context, msg: &Message) -> CommandResult {
    let pool = {
        let data = ctx.data.read().await;
        data.get::<DatabasePool>().expect("Pool absent").clone()
    };
    match get_active_challenge(&pool).await? {
        Some(challenge) => {
            let template = challenge_template_for_key(&challenge.kind);
            msg.channel_id
                .say(
                    &ctx.http,
                    format!(
                        "🤝 **{}**\n{}\n\nProgression : **{}/{}**\nFin vers **{}**\nCommande : `^^participer_defi <id_chat>`",
                        challenge.title,
                        template.intro,
                        challenge.progress,
                        challenge.target,
                        challenge.ends_at.format("%H:%M")
                    ),
                )
                .await?;
        }
        None => {
            msg.channel_id
                .say(
                    &ctx.http,
                    "Aucun défi communautaire en cours. Ils peuvent apparaître à tout moment.",
                )
                .await?;
        }
    }
    Ok(())
}

fn challenge_template_for_key(key: &str) -> ChallengeTemplate {
    (0..5)
        .map(challenge_template_for_roll)
        .find(|template| template.key == key)
        .unwrap_or_else(|| challenge_template_for_roll(0))
}

#[command]
#[description = "Participe au défi avec un chat : ^^participer_defi <id_chat>"]
pub async fn participer_defi(ctx: &Context, msg: &Message, mut args: Args) -> CommandResult {
    if msg.guild_id.is_none() {
        msg.channel_id
            .say(
                &ctx.http,
                "Les défis communautaires ont lieu sur le serveur.",
            )
            .await?;
        return Ok(());
    }
    let cat_id = match args.single::<i32>() {
        Ok(id) => id,
        Err(_) => {
            msg.channel_id
                .say(&ctx.http, "Usage : `^^participer_defi <id_chat>`")
                .await?;
            return Ok(());
        }
    };
    let pool = {
        let data = ctx.data.read().await;
        data.get::<DatabasePool>().expect("Pool absent").clone()
    };
    let challenge = match get_active_challenge(&pool).await? {
        Some(challenge) => challenge,
        None => {
            msg.channel_id
                .say(&ctx.http, "Aucun défi communautaire en cours.")
                .await?;
            return Ok(());
        }
    };
    let user = match get_user_by_discord_id(&pool, msg.author.id.0).await? {
        Some(user) => user,
        None => return Ok(()),
    };
    let cat = match get_cat_by_id(&pool, cat_id).await? {
        Some(cat) if cat.user_id == user.id as i32 && cat.location == "home" => cat,
        _ => {
            msg.channel_id
                .say(&ctx.http, "Ce chat ne vit pas chez toi.")
                .await?;
            return Ok(());
        }
    };
    if is_cat_on_expedition(&pool, cat_id).await? {
        msg.channel_id
            .say(
                &ctx.http,
                "Ce chat profite actuellement de sa promenade. Choisis un autre résident.",
            )
            .await?;
        return Ok(());
    }
    let contribution = 2 + (cat.rarity_score / 4).max(0);
    let inserted = sqlx::query(
        "INSERT IGNORE INTO cat_community_participants (challenge_id, user_id, cat_id, contribution, joined_at) VALUES (?, ?, ?, ?, ?)",
    )
    .bind(challenge.id)
    .bind(user.id as i32)
    .bind(cat_id)
    .bind(contribution)
    .bind(paris_now_naive())
    .execute(&*pool)
    .await?;
    if inserted.rows_affected() == 0 {
        msg.channel_id
            .say(
                &ctx.http,
                "Tu participes déjà à ce défi. Ton chat peut maintenant se reposer.",
            )
            .await?;
        return Ok(());
    }
    sqlx::query("UPDATE cat_community_challenges SET progress = progress + ? WHERE id = ? AND status = 'active'")
        .bind(contribution)
        .bind(challenge.id)
        .execute(&*pool)
        .await?;
    record_activity(&pool, user.id, "challenge").await?;
    let template = challenge_template_for_key(&challenge.kind);
    msg.channel_id
        .say(
            &ctx.http,
            format!(
                "🐱 **{}** {} (+{}).",
                cat.name, template.action, contribution
            ),
        )
        .await?;
    resolve_completed_challenges(ctx, &pool).await?;
    Ok(())
}

async fn resolve_completed_challenges(
    ctx: &Context,
    pool: &Pool<MySql>,
) -> Result<(), sqlx::Error> {
    let Some(challenge) = get_active_challenge(pool).await? else {
        return Ok(());
    };
    let now = paris_now_naive();
    if challenge.progress < challenge.target && challenge.ends_at > now {
        return Ok(());
    }
    let success = challenge.progress >= challenge.target;
    let template = challenge_template_for_key(&challenge.kind);
    let resolution = challenge_resolution(&template, challenge.progress, challenge.target);
    let result = sqlx::query(
        "UPDATE cat_community_challenges SET status = ?, resolution = ? WHERE id = ? AND status = 'active'",
    )
    .bind(if success { "success" } else { "closed" })
    .bind(&resolution)
    .bind(challenge.id)
    .execute(pool)
    .await?;
    if result.rows_affected() == 0 {
        return Ok(());
    }
    let participants = sqlx::query(
        "SELECT user_id, cat_id FROM cat_community_participants WHERE challenge_id = ?",
    )
    .bind(challenge.id)
    .fetch_all(pool)
    .await?;
    for participant in participants {
        let user_id: i32 = participant.get("user_id");
        let cat_id: i32 = participant.get("cat_id");
        unlock_decoration(pool, user_id as i64, "fanion_solidaire")
            .await
            .ok();
        add_cat_memory(
            pool,
            cat_id,
            Some(user_id as i64),
            "community",
            &format!("A participé au défi communautaire « {} ».", template.title),
        )
        .await
        .ok();
    }
    ChannelId(challenge.channel_id)
        .say(&ctx.http, resolution)
        .await
        .ok();
    Ok(())
}

async fn weekly_summary(pool: &Pool<MySql>) -> Result<Vec<String>, sqlx::Error> {
    let start = week_start(paris_today()).and_hms_opt(0, 0, 0).unwrap();
    let regular_rows = sqlx::query(
        "SELECT u.id_utilisateur, u.pseudo, COUNT(DISTINCT DATE(d.created_at)) AS regular_days
         FROM daily_cats d JOIN utilisateurs u ON u.id = d.user_id
         WHERE d.created_at >= ?
         GROUP BY u.id, u.id_utilisateur, u.pseudo
         ORDER BY regular_days DESC, u.pseudo ASC LIMIT 10",
    )
    .bind(start)
    .fetch_all(pool)
    .await?;
    let mut entries = vec!["\n**Les personnes les plus régulières sur `^^cat`**".to_string()];
    if regular_rows.is_empty() {
        entries.push("• Aucun Daily Cat enregistré cette semaine.".to_string());
    } else {
        for (index, row) in regular_rows.into_iter().enumerate() {
            let discord_id: String = row.get("id_utilisateur");
            let days: i64 = row.get("regular_days");
            entries.push(format!(
                "{}. <@{}> — **{} jour{} sur 7**",
                index + 1,
                discord_id,
                days,
                if days > 1 { "s" } else { "" }
            ));
        }
    }
    entries.push("\n**Tous les chats apparus cette semaine**".to_string());
    let cat_rows = sqlx::query(
        "SELECT c.id, c.name, c.nickname, c.breed, c.color, c.rarity_score, u.id_utilisateur
         FROM collected_cats c JOIN utilisateurs u ON u.id = c.user_id
         WHERE c.obtained_at >= ? ORDER BY c.obtained_at ASC, c.id ASC",
    )
    .bind(start)
    .fetch_all(pool)
    .await?;
    if cat_rows.is_empty() {
        entries.push("• Aucun nouveau résident nommé cette semaine.".to_string());
    } else {
        for row in cat_rows {
            let name: String = row.try_get("name").unwrap_or_else(|_| "Chat".to_string());
            let nickname: Option<String> = row.try_get("nickname").unwrap_or(None);
            let name = safe_discord_text(&name);
            let display = nickname
                .filter(|value| !value.trim().is_empty())
                .map(|value| format!("{} « {} »", name, safe_discord_text(&value)))
                .unwrap_or(name);
            let breed: String = row.get("breed");
            let color: String = row.get("color");
            let breed = safe_discord_text(&breed);
            let color = safe_discord_text(&color);
            let rarity: i32 = row.get("rarity_score");
            let discord_id: String = row.get("id_utilisateur");
            let id: i32 = row.get("id");
            entries.push(format!(
                "• **{}** (#{}) — {} {}, rareté {}/20 — <@{}>",
                display, id, breed, color, rarity, discord_id
            ));
        }
    }
    Ok(split_discord_messages(
        "📅 **Résumé humain de la semaine**",
        &entries,
        1900,
    ))
}

#[command]
#[description = "Affiche le résumé actuel de la semaine"]
pub async fn hebdo(ctx: &Context, msg: &Message) -> CommandResult {
    let pool = {
        let data = ctx.data.read().await;
        data.get::<DatabasePool>().expect("Pool absent").clone()
    };
    for message in weekly_summary(&pool).await? {
        msg.channel_id.say(&ctx.http, message).await?;
    }
    Ok(())
}

async fn maybe_send_scheduled_summary(
    ctx: &Context,
    pool: &Pool<MySql>,
    channel_id: ChannelId,
) -> Result<(), sqlx::Error> {
    let now = paris_now();
    if now.weekday() != Weekday::Sun || now.hour() < 20 {
        return Ok(());
    }
    let run_key = format!("weekly_summary_{}", week_start(paris_today()));
    let result =
        sqlx::query("INSERT IGNORE INTO cat_scheduled_runs (run_key, ran_at) VALUES (?, ?)")
            .bind(run_key)
            .bind(paris_now_naive())
            .execute(pool)
            .await?;
    if result.rows_affected() == 0 {
        return Ok(());
    }
    for message in weekly_summary(pool).await? {
        channel_id.say(&ctx.http, message).await.ok();
    }
    Ok(())
}

pub fn start_retention_scheduler(ctx: Context) {
    if RETENTION_SCHEDULER_STARTED.swap(true, Ordering::SeqCst) {
        return;
    }
    tokio::spawn(async move {
        let channel_id = ChannelId(configured_cat_channel_id());
        loop {
            let pool = {
                let data = ctx.data.read().await;
                data.get::<DatabasePool>().cloned()
            };
            if let Some(pool) = pool {
                resolve_completed_challenges(&ctx, &pool).await.ok();
                maybe_send_scheduled_summary(&ctx, &pool, channel_id)
                    .await
                    .ok();
                start_random_challenge(&ctx, &pool, channel_id, false)
                    .await
                    .ok();
            }
            tokio::time::sleep(std::time::Duration::from_secs(15 * 60)).await;
        }
    });
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn semaine_commence_le_lundi() {
        let sunday = NaiveDate::from_ymd_opt(2026, 9, 20).unwrap();
        assert_eq!(
            week_start(sunday),
            NaiveDate::from_ymd_opt(2026, 9, 14).unwrap()
        );
        let monday = NaiveDate::from_ymd_opt(2026, 9, 21).unwrap();
        assert_eq!(week_start(monday), monday);
    }

    #[test]
    fn les_defis_aleatoires_ne_sont_pas_uniquement_des_boss() {
        assert_eq!(challenge_template_for_roll(0).key, "chien_turbulent");
        assert_eq!(challenge_template_for_roll(1).key, "carton_geant");
        assert_eq!(challenge_template_for_roll(2).key, "orage_solidaire");
        assert_eq!(challenge_template_for_roll(3).key, "pelote_geante");
        assert_eq!(challenge_template_for_roll(4).key, "pique_nique");
    }

    #[test]
    fn un_seul_tirage_est_possible_par_quart_d_heure() {
        let first = NaiveDate::from_ymd_opt(2026, 9, 20)
            .unwrap()
            .and_hms_opt(12, 1, 0)
            .unwrap();
        let same_bucket = NaiveDate::from_ymd_opt(2026, 9, 20)
            .unwrap()
            .and_hms_opt(12, 14, 59)
            .unwrap();
        let next_bucket = NaiveDate::from_ymd_opt(2026, 9, 20)
            .unwrap()
            .and_hms_opt(12, 15, 0)
            .unwrap();
        assert_eq!(
            challenge_roll_bucket(first),
            challenge_roll_bucket(same_bucket)
        );
        assert_ne!(
            challenge_roll_bucket(first),
            challenge_roll_bucket(next_bucket)
        );
    }

    #[test]
    fn aucune_issue_de_defi_ne_fait_de_mal_aux_chats() {
        for roll in 0..5 {
            let template = challenge_template_for_roll(roll);
            let success = challenge_resolution(&template, 20, 20);
            let retreat = challenge_resolution(&template, 5, 20);
            for text in [success, retreat] {
                let lower = text.to_lowercase();
                for forbidden in ["bless", "mort", "attaque", "dégât", "maltrait", "perdu"] {
                    assert!(!lower.contains(forbidden), "texte interdit: {text}");
                }
            }
        }
    }

    #[test]
    fn durees_expedition_connues() {
        assert_eq!(expedition_hours("courte"), Some(2));
        assert_eq!(expedition_hours("moyenne"), Some(6));
        assert_eq!(expedition_hours("longue"), Some(12));
        assert_eq!(expedition_hours("dangereuse"), None);
    }

    #[test]
    fn les_longues_listes_sont_decoupees_pour_discord() {
        let header = "Résumé";
        let entries: Vec<String> = (0..120)
            .map(|index| format!("• Chat numéro {index} avec une description chaleureuse"))
            .collect();
        let messages = split_discord_messages(header, &entries, 500);
        assert!(messages.len() > 1);
        assert!(messages
            .iter()
            .all(|message| message.chars().count() <= 500));
        for entry in entries {
            assert!(messages.iter().any(|message| message.contains(&entry)));
        }
    }

    #[test]
    fn les_surnoms_ne_peuvent_pas_declencher_de_mention_dans_le_resume() {
        let safe = safe_discord_text("@everyone **Minou**");
        assert!(!safe.contains("@everyone"));
        assert!(!safe.contains("**Minou**"));
        assert!(safe.contains("Minou"));
    }
}
