//! Événements de chats : chats sauvages, journées d'adoption et leur résolution.

use rand::{thread_rng, Rng};
use serenity::framework::standard::{macros::command, Args, CommandResult};
use serenity::model::prelude::*;
use serenity::prelude::*;
use chrono::{Datelike, NaiveDateTime, Timelike, Weekday};
use std::collections::{HashMap, HashSet};
use std::sync::Arc;
use tokio::sync::Mutex;

use crate::database::{
    add_cat_event_participant, add_collected_cat, delete_active_cat_event,
    get_active_cat_events, get_cat_by_id, get_cat_event_count_last_7_days, get_refuge_cats,
    get_user_by_discord_id, give_refuge_cat_to_user, new_user, record_cat_event_start,
    save_active_cat_event,
    DatabasePool,
};
use crate::time::{paris_now, paris_now_naive, paris_today};
use crate::retention::has_active_community_challenge;

use super::*;

pub struct CatEventContainer;

impl TypeMapKey for CatEventContainer {
    type Value = Arc<Mutex<HashMap<u64, CatEvent>>>;
}

#[derive(Clone, Copy)]
pub enum CatEventKind {
    Wild,
    Adoption,
}

#[derive(Clone)]
pub struct CatEvent {
    pub kind: CatEventKind,
    pub participants: HashSet<UserId>,
    pub theme: Option<CatEventTheme>,
}

pub(super) const CAT_EVENT_DEFAULT_DURATION_SECS: u64 = 3 * 60 * 60;
pub(super) const CAT_EVENT_MAX_PER_WEEK: i64 = 7;
pub(super) const CAT_EVENT_BASE_CHANCE_PERCENT: i32 = 12;
pub(super) const SOYER_USER_ID: u64 = 530757472336478230;
pub(super) const CAT_EVENT_ROLE_MENTION: &str = "<@&1500871713381159125>";

#[derive(Clone, Copy)]
pub struct CatEventTheme {
    pub key: &'static str,
    pub intro: &'static str,
    pub memory: &'static str,
    pub names: &'static [&'static str],
    pub breeds: &'static [&'static str],
    pub colors: &'static [&'static str],
    pub rarity_bonus: i32,
}


#[command]
#[description = "Liste les 10 prochains événements chats"]
pub async fn catevents(ctx: &Context, msg: &Message) -> CommandResult {
    let today = paris_today();
    let mut events = Vec::new();

    for offset in 0..730 {
        let date = today + chrono::Duration::days(offset);
        if let Some(theme) = cat_event_theme_for_date(date) {
            let when = if offset == 0 {
                "Aujourd'hui".to_string()
            } else {
                date.format("%d/%m/%Y").to_string()
            };
            events.push(format!(
                "• **{}** — {} (`+{}` rareté)",
                when,
                cat_event_label(theme.key),
                theme.rarity_bonus
            ));

            if events.len() == 10 {
                break;
            }
        }
    }

    let response = if events.is_empty() {
        "Aucun événement chat trouvé dans les deux prochaines années.".to_string()
    } else {
        format!("📅 **10 prochains événements chats**\n{}", events.join("\n"))
    };

    msg.channel_id.say(&ctx.http, response).await.ok();
    Ok(())
}

#[command]
#[description = "Participe à l'approche d'un chat sauvage"]
pub async fn caliner(ctx: &Context, msg: &Message) -> CommandResult {
    join_cat_event(ctx, msg, CatEventKind::Wild, "Aucun chat sauvage ne rôde ici pour le moment.").await?;
    Ok(())
}

#[command]
#[description = "Participe à une adoption depuis le refuge"]
pub async fn adopter(ctx: &Context, msg: &Message) -> CommandResult {
    join_cat_event(ctx, msg, CatEventKind::Adoption, "Aucune adoption du refuge n'est ouverte ici pour le moment.").await?;
    Ok(())
}

#[command]
#[description = "Commande secrète de contrôle des événements chats"]
pub async fn catcontrol(ctx: &Context, msg: &Message, mut args: Args) -> CommandResult {
    if msg.author.id.0 != SOYER_USER_ID {
        return Ok(());
    }

    let action = args.single::<String>().unwrap_or_default().to_lowercase();
    match action.as_str() {
        "stop" | "arret" | "arrêt" => {
            if take_cat_event(ctx, msg.channel_id).await.is_some() {
                msg.channel_id
                    .say(&ctx.http, "Événement chat arrêté dans ce salon.")
                    .await
                    .ok();
            } else {
                msg.channel_id
                    .say(&ctx.http, "Aucun événement chat actif dans ce salon.")
                    .await
                    .ok();
            }
        }
        "wild" | "sauvage" | "caliner" => {
            if start_cat_event(
                ctx,
                msg.channel_id,
                CatEventKind::Wild,
                cat_event_theme_for_date(paris_today()),
            )
            .await
            {
                msg.channel_id
                    .say(&ctx.http, "Événement chat sauvage lancé manuellement.")
                    .await
                    .ok();
            } else {
                msg.channel_id
                    .say(&ctx.http, "Un événement chat est déjà actif dans ce salon.")
                    .await
                    .ok();
            }
        }
        "adoption" | "adopter" | "refuge" => {
            let data = ctx.data.read().await;
            let pool = data
                .get::<DatabasePool>()
                .expect("Impossible d'obtenir le pool")
                .clone();
            drop(data);

            match get_refuge_cats(&pool, 1).await {
                Ok(cats) if cats.is_empty() => {
                    msg.channel_id
                        .say(&ctx.http, "Impossible: le refuge est vide.")
                        .await
                        .ok();
                }
                Ok(_) => {
                    if start_cat_event(
                        ctx,
                        msg.channel_id,
                        CatEventKind::Adoption,
                        cat_event_theme_for_date(paris_today()),
                    )
                    .await
                    {
                        msg.channel_id
                            .say(&ctx.http, "Événement adoption lancé manuellement.")
                            .await
                            .ok();
                    } else {
                        msg.channel_id
                            .say(&ctx.http, "Un événement chat est déjà actif dans ce salon.")
                            .await
                            .ok();
                    }
                }
                Err(_) => {
                    msg.channel_id
                        .say(&ctx.http, "Impossible de vérifier le refuge.")
                        .await
                        .ok();
                }
            }
        }
        _ => {
            msg.channel_id
                .say(
                    &ctx.http,
                    "Usage secret: `^^catcontrol wild`, `^^catcontrol adoption`, `^^catcontrol stop`.",
                )
                .await
                .ok();
        }
    }

    Ok(())
}

pub(super) async fn join_cat_event(
    ctx: &Context,
    msg: &Message,
    kind: CatEventKind,
    no_event_message: &str,
) -> CommandResult {
    let pool = {
        let data = ctx.data.read().await;
        data.get::<DatabasePool>().cloned()
    };
    let events = {
        let data = ctx.data.read().await;
        data.get::<CatEventContainer>()
            .expect("Impossible d'obtenir les événements de chats")
            .clone()
    };

    let mut events = events.lock().await;
    let channel_key = msg.channel_id.0;

    if let Some(event) = events.get_mut(&channel_key) {
        let same_kind = matches!(
            (&event.kind, &kind),
            (CatEventKind::Wild, CatEventKind::Wild)
                | (CatEventKind::Adoption, CatEventKind::Adoption)
        );

        if !same_kind {
            msg.channel_id
                .say(
                    &ctx.http,
                    "Un autre type d'événement chat est en cours dans ce salon.",
                )
                .await
                .ok();
            return Ok(());
        }

        let inserted = event.participants.insert(msg.author.id);
        let event_kind = event.kind;
        drop(events);

        if inserted {
            if let Some(pool) = pool {
                add_cat_event_participant(&pool, channel_key, msg.author.id.0)
                    .await
                    .ok();
            }
        }

        let action = match event_kind {
            CatEventKind::Wild => "tente de rassurer le chat sauvage",
            CatEventKind::Adoption => "visite le refuge avec douceur",
        };
        let suffix = if inserted {
            "."
        } else {
            " à nouveau, mais il est déjà inscrit."
        };
        msg.channel_id
            .say(&ctx.http, format!("{} {}{}", msg.author.name, action, suffix))
            .await
            .ok();
        return Ok(());
    }

    msg.channel_id.say(&ctx.http, no_event_message).await.ok();
    Ok(())
}

pub(super) async fn take_cat_event(ctx: &Context, channel_id: ChannelId) -> Option<CatEvent> {
    let pool = {
        let data = ctx.data.read().await;
        data.get::<DatabasePool>().cloned()
    };
    let events = {
        let data = ctx.data.read().await;
        data.get::<CatEventContainer>()?.clone()
    };

    let mut events = events.lock().await;
    let event = events.remove(&channel_id.0);
    drop(events);

    if event.is_some() {
        if let Some(pool) = pool {
            delete_active_cat_event(&pool, channel_id.0).await.ok();
        }
    }

    event
}

pub(super) async fn maybe_trigger_cat_event(ctx: &Context, msg: &Message, pool: &sqlx::Pool<sqlx::MySql>) {
    if has_active_community_challenge(pool).await {
        return;
    }
    if channel_has_cat_event(ctx, msg.channel_id).await {
        return;
    }

    let weekly_count = match get_cat_event_count_last_7_days(pool).await {
        Ok(count) => count,
        Err(_) => return,
    };

    if weekly_count >= CAT_EVENT_MAX_PER_WEEK {
        return;
    }

    let today = paris_today();
    let must_force_weekly_event = weekly_count == 0
        && matches!(today.weekday(), Weekday::Fri | Weekday::Sat | Weekday::Sun);
    let should_start = must_force_weekly_event
        || thread_rng().gen_range(0..100) < CAT_EVENT_BASE_CHANCE_PERCENT;

    if !should_start {
        return;
    }

    let theme = cat_event_theme_for_date(today);
    let has_refuge_cats = matches!(get_refuge_cats(pool, 1).await, Ok(cats) if !cats.is_empty());
    let kind = if has_refuge_cats && thread_rng().gen_range(0..100) < 35 {
        CatEventKind::Adoption
    } else {
        CatEventKind::Wild
    };

    if start_cat_event(ctx, msg.channel_id, kind, theme).await {
        record_cat_event_start(pool, msg.channel_id.0, cat_event_kind_key(kind), theme.map(|theme| theme.key)).await.ok();
    }
}

pub(super) async fn channel_has_cat_event(ctx: &Context, channel_id: ChannelId) -> bool {
    let events = {
        let data = ctx.data.read().await;
        match data.get::<CatEventContainer>() {
            Some(events) => events.clone(),
            None => return false,
        }
    };

    let events = events.lock().await;
    events.contains_key(&channel_id.0)
}

pub(super) fn cat_event_kind_key(kind: CatEventKind) -> &'static str {
    match kind {
        CatEventKind::Wild => "wild",
        CatEventKind::Adoption => "adoption",
    }
}

pub(super) fn cat_event_kind_from_key(key: &str) -> Option<CatEventKind> {
    match key {
        "wild" => Some(CatEventKind::Wild),
        "adoption" => Some(CatEventKind::Adoption),
        _ => None,
    }
}

pub(super) async fn start_cat_event(
    ctx: &Context,
    channel_id: ChannelId,
    kind: CatEventKind,
    theme: Option<CatEventTheme>,
) -> bool {
    let (duration_secs, duration_label) = cat_event_duration();
    let ends_at = paris_now_naive() + chrono::Duration::seconds(duration_secs as i64);
    let pool = {
        let data = ctx.data.read().await;
        data.get::<DatabasePool>().cloned()
    };
    let events = {
        let data = ctx.data.read().await;
        match data.get::<CatEventContainer>() {
            Some(events) => events.clone(),
            None => return false,
        }
    };

    {
        let mut events = events.lock().await;
        if events.contains_key(&channel_id.0) {
            return false;
        }

        events.insert(
            channel_id.0,
            CatEvent {
                kind,
                participants: HashSet::new(),
                theme,
            },
        );
    }

    if let Some(pool) = pool {
        let event_kind = cat_event_kind_key(kind);
        if save_active_cat_event(
            &pool,
            channel_id.0,
            event_kind,
            theme.map(|theme| theme.key),
            ends_at,
        )
        .await
        .is_err()
        {
            take_cat_event(ctx, channel_id).await;
            return false;
        }
    }

    match kind {
        CatEventKind::Wild => {
            let intro = match theme {
                Some(theme) => format!("{}\nUn chat sauvage spécial rôde près du serveur...", theme.intro),
                None => "Un chat sauvage rôde près du serveur...".to_string(),
            };
            channel_id.say(&ctx.http, format!(
                "{}\n{}\nUtilisez `^^caliner` {} pour tenter de gagner sa confiance.",
                CAT_EVENT_ROLE_MENTION,
                intro,
                duration_label
            )).await.ok();
            spawn_wild_cat_resolution(ctx.clone(), channel_id, duration_secs).await;
        }
        CatEventKind::Adoption => {
            let intro = match theme {
                Some(theme) => format!("{}\nUne journée d'adoption spéciale commence au refuge.", theme.intro),
                None => "Une journée d'adoption commence au refuge.".to_string(),
            };
            channel_id.say(&ctx.http, format!(
                "{}\n{}\nUtilisez `^^adopter` {}. Un résident du refuge choisira une maison.",
                CAT_EVENT_ROLE_MENTION,
                intro,
                duration_label
            )).await.ok();
            spawn_adoption_resolution(ctx.clone(), channel_id, duration_secs).await;
        }
    }

    true
}

pub async fn restore_cat_events(ctx: &Context) {
    let pool = {
        let data = ctx.data.read().await;
        match data.get::<DatabasePool>() {
            Some(pool) => pool.clone(),
            None => return,
        }
    };
    let stored_events = match get_active_cat_events(&pool).await {
        Ok(events) => events,
        Err(_) => return,
    };
    let events = {
        let data = ctx.data.read().await;
        match data.get::<CatEventContainer>() {
            Some(events) => events.clone(),
            None => return,
        }
    };

    for stored in stored_events {
        let kind = match cat_event_kind_from_key(&stored.event_kind) {
            Some(kind) => kind,
            None => {
                delete_active_cat_event(&pool, stored.channel_id).await.ok();
                continue;
            }
        };
        let duration_secs = remaining_cat_event_seconds(stored.ends_at);
        let channel_id = ChannelId(stored.channel_id);
        let theme = stored
            .theme
            .as_deref()
            .and_then(cat_event_theme_by_key);
        let participants = stored.participants.into_iter().map(UserId).collect();

        {
            let mut events = events.lock().await;
            if events.contains_key(&stored.channel_id) {
                continue;
            }

            events.insert(
                stored.channel_id,
                CatEvent {
                    kind,
                    participants,
                    theme,
                },
            );
        }

        spawn_cat_event_resolution(ctx.clone(), channel_id, kind, duration_secs).await;
    }
}

pub(super) fn remaining_cat_event_seconds(ends_at: NaiveDateTime) -> u64 {
    ends_at
        .signed_duration_since(paris_now_naive())
        .num_seconds()
        .max(0) as u64
}

pub(super) async fn spawn_cat_event_resolution(ctx: Context, channel_id: ChannelId, kind: CatEventKind, duration_secs: u64) {
    match kind {
        CatEventKind::Wild => spawn_wild_cat_resolution(ctx, channel_id, duration_secs).await,
        CatEventKind::Adoption => spawn_adoption_resolution(ctx, channel_id, duration_secs).await,
    }
}

pub(super) fn cat_event_duration() -> (u64, &'static str) {
    let now = paris_now();
    let hour = now.hour();
    let elapsed_in_hour = now.minute() as u64 * 60 + now.second() as u64;

    if hour >= 23 {
        let seconds_until_midnight = (24 - hour) as u64 * 60 * 60 - elapsed_in_hour;
        return (
            seconds_until_midnight + 12 * 60 * 60,
            "jusqu'à midi",
        );
    }

    if hour < 9 {
        return (
            (12 - hour) as u64 * 60 * 60 - elapsed_in_hour,
            "jusqu'à midi",
        );
    }

    (
        CAT_EVENT_DEFAULT_DURATION_SECS,
        "pendant 3 heures",
    )
}


pub(super) fn generate_event_cat(theme: Option<CatEventTheme>) -> Cat {
    let Some(theme) = theme else {
        return Cat::generate_random();
    };

    let mut rng = thread_rng();
    let name = theme.names[rng.gen_range(0..theme.names.len())].to_string();
    let breed = theme.breeds[rng.gen_range(0..theme.breeds.len())];
    let color = theme.colors[rng.gen_range(0..theme.colors.len())];
    let age_months = rng.gen_range(1..=60);
    let breed_bonus = get_cat_breeds()
        .into_iter()
        .find(|item| item.name == breed)
        .map(|item| item.rarity_bonus)
        .unwrap_or(0);
    let color_bonus = get_cat_colors()
        .into_iter()
        .find(|item| item.name == color)
        .map(|item| item.rarity_bonus)
        .unwrap_or(0);

    let mut cat = Cat {
        breed: CatBreed {
            name: breed,
            rarity_bonus: breed_bonus,
        },
        color: CatColor {
            name: color,
            rarity_bonus: color_bonus,
        },
        age_months,
        name,
        rarity_score: 0,
    };
    cat.rarity_score = (cat.calculate_rarity() + theme.rarity_bonus).clamp(1, 20);
    cat
}

pub(super) async fn spawn_wild_cat_resolution(ctx: Context, channel_id: ChannelId, duration_secs: u64) {
    tokio::spawn(async move {
        tokio::time::sleep(tokio::time::Duration::from_secs(duration_secs)).await;

        let event = match take_cat_event(&ctx, channel_id).await {
            Some(event) => event,
            None => return,
        };
        let theme_line = event
            .theme
            .map(|theme| format!("{}\n", theme.intro))
            .unwrap_or_default();
        let event_theme = event.theme;

        let participants: Vec<UserId> = event.participants.into_iter().collect();
        if participants.is_empty() {
            channel_id
                .say(
                    &ctx.http,
                    format!("{}Le chat sauvage a observe le serveur de loin, puis il est reparti.", theme_line),
                )
                .await
                .ok();
            return;
        }

        let winner_id = participants[thread_rng().gen_range(0..participants.len())];
        let data = ctx.data.read().await;
        let pool = match data.get::<DatabasePool>() {
            Some(pool) => pool.clone(),
            None => return,
        };
        drop(data);

        let discord_user = match winner_id.to_user(&ctx.http).await {
            Ok(user) => user,
            Err(_) => return,
        };

        let user = match get_user_by_discord_id(&pool, winner_id.0).await {
            Ok(Some(user)) => user,
            Ok(None) => match new_user(&pool, winner_id.0, &discord_user.name).await {
                Ok(id) => crate::database::User {
                    id,
                    id_utilisateur: winner_id.0.to_string(),
                    pseudo: discord_user.name.clone(),
                    score: 0,
                },
                Err(_) => return,
            },
            Err(_) => return,
        };

        let wild_cat = generate_event_cat(event_theme);
        match add_collected_cat(
            &pool,
            user.id,
            &wild_cat.name,
            wild_cat.breed.name,
            wild_cat.color.name,
            wild_cat.age_months,
            wild_cat.rarity_score,
        )
        .await
        {
            Ok(cat_id) => {
                if let Some(theme) = event_theme {
                    crate::database::add_cat_memory(
                        &pool,
                        cat_id,
                        Some(user.id),
                        "event",
                        theme.memory,
                    )
                    .await
                    .ok();
                }
                let saved_cat = get_cat_by_id(&pool, cat_id).await.ok().flatten();
                let description =
                    saved_cat
                        .as_ref()
                        .map(format_cat_identity)
                        .unwrap_or_else(|| {
                            format!(
                                "{} **{}** (#{})",
                                get_rarity_emoji(wild_cat.rarity_score),
                                wild_cat.format_description(),
                                cat_id
                            )
                        });

                channel_id.say(&ctx.http, format!(
                    "{}Le chat sauvage s'approche lentement...\nIl a choisi <@{}> !\n{} rejoint sa maison.",
                    theme_line,
                    winner_id.0,
                    description
                )).await.ok();
            }
            Err(_) => {
                channel_id
                    .say(
                        &ctx.http,
                        "Le chat sauvage voulait rester, mais une erreur est survenue.",
                    )
                    .await
                    .ok();
            }
        }
    });
}

pub(super) async fn spawn_adoption_resolution(ctx: Context, channel_id: ChannelId, duration_secs: u64) {
    tokio::spawn(async move {
        tokio::time::sleep(tokio::time::Duration::from_secs(duration_secs)).await;

        let event = match take_cat_event(&ctx, channel_id).await {
            Some(event) => event,
            None => return,
        };
        let theme_line = event
            .theme
            .map(|theme| format!("{}\n", theme.intro))
            .unwrap_or_default();

        let participants: Vec<UserId> = event.participants.into_iter().collect();
        if participants.is_empty() {
            channel_id.say(&ctx.http, format!("{}La journée d'adoption se termine dans le calme. Aucun résident n'a choisi de partir.", theme_line)).await.ok();
            return;
        }

        let data = ctx.data.read().await;
        let pool = match data.get::<DatabasePool>() {
            Some(pool) => pool.clone(),
            None => return,
        };
        drop(data);

        let refuge_cats = match get_refuge_cats(&pool, 100).await {
            Ok(cats) if !cats.is_empty() => cats,
            _ => {
                channel_id
                    .say(&ctx.http, "Le refuge est vide au moment de l'adoption.")
                    .await
                    .ok();
                return;
            }
        };

        let winner_id = participants[thread_rng().gen_range(0..participants.len())];
        let chosen_cat = refuge_cats[thread_rng().gen_range(0..refuge_cats.len())].clone();
        let discord_user = match winner_id.to_user(&ctx.http).await {
            Ok(user) => user,
            Err(_) => return,
        };

        let user = match get_user_by_discord_id(&pool, winner_id.0).await {
            Ok(Some(user)) => user,
            Ok(None) => match new_user(&pool, winner_id.0, &discord_user.name).await {
                Ok(id) => crate::database::User {
                    id,
                    id_utilisateur: winner_id.0.to_string(),
                    pseudo: discord_user.name.clone(),
                    score: 0,
                },
                Err(_) => return,
            },
            Err(_) => return,
        };

        if !matches!(
            give_refuge_cat_to_user(&pool, chosen_cat.id, user.id).await,
            Ok(true)
        ) {
            channel_id
                .say(&ctx.http, "L'adoption n'a pas pu etre finalisee.")
                .await
                .ok();
            return;
        }

        channel_id.say(&ctx.http, format!(
            "{}La journée d'adoption se termine...\n{} a choisi <@{}> et quitte le refuge pour rejoindre sa maison.",
            theme_line,
            format_cat_identity(&chosen_cat),
            winner_id.0
        )).await.ok();
    });
}
