use rand::{thread_rng, Rng};
use serenity::framework::standard::{macros::command, Args, CommandResult};
use serenity::model::channel::ReactionType;
use serenity::model::prelude::*;
use serenity::prelude::*;
use chrono::{Datelike, NaiveDate, Timelike, Weekday};
use std::collections::{HashMap, HashSet};
use std::sync::Arc;
use tokio::sync::Mutex;

use crate::database::{
    add_collected_cat, add_daily_cat, get_cat_by_id, get_cat_memories,
    get_cat_event_count_last_7_days, get_cat_server_stats, get_daily_cat_count,
    get_daily_cat_count_today, get_refuge_cats, get_user_by_discord_id, get_user_cat_counts,
    get_user_cats, give_refuge_cat_to_user, has_daily_cat_today, move_cat_to_refuge, new_user,
    record_cat_event_start, set_cat_nickname, set_favorite_cat, transfer_cat, CollectedCat,
    DatabasePool,
};
use crate::time::{paris_now, paris_today};

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

const CAT_EVENT_DEFAULT_DURATION_SECS: u64 = 3 * 60 * 60;
const CAT_EVENT_MAX_PER_WEEK: i64 = 7;
const CAT_EVENT_BASE_CHANCE_PERCENT: i32 = 12;
const SOYER_USER_ID: u64 = 530757472336478230;

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

#[derive(Clone)]
pub struct CatBreed {
    pub name: &'static str,
    pub rarity_bonus: i32,
}

#[derive(Clone)]
pub struct CatColor {
    pub name: &'static str,
    pub rarity_bonus: i32,
}

#[derive(Clone)]
pub struct Cat {
    pub breed: CatBreed,
    pub color: CatColor,
    pub age_months: i32,
    pub name: String,
    pub rarity_score: i32,
}

impl Cat {
    pub fn calculate_rarity(&self) -> i32 {
        // Système de rareté revu pour avoir vraiment des scores bas
        let breed_bonus = self.breed.rarity_bonus;
        let color_bonus = self.color.rarity_bonus;

        // Bonus d'âge plus modéré
        let age_bonus = match self.age_months {
            1..=2 => 2,   // Très jeune
            3..=6 => 1,   // Jeune
            7..=12 => 0,  // Normal
            13..=24 => 0, // Adulte
            _ => -1,      // Vieux (malus)
        };

        // Bonus de nom réduit
        let name_bonus = match self.name.len() {
            1..=5 => 0,
            6..=8 => 1,
            _ => 2,
        };

        // Bonus voyelle réduit
        let vowel_bonus = if "AEIOUaeiou".contains(self.name.chars().next().unwrap_or('x')) {
            1
        } else {
            0
        };

        // Score de base très bas pour forcer la rareté
        let base_score = 1;

        // Calcul avec plafonnement plus strict
        let raw_score =
            base_score + breed_bonus + color_bonus + age_bonus + name_bonus + vowel_bonus;

        // Système de plafonnement plus modéré
        let capped_score = match raw_score {
            0..=10 => raw_score,                  // Scores bas : pas de changement
            11..=14 => 10 + (raw_score - 10) / 2, // Scores moyens : légère réduction
            15..=18 => 12 + (raw_score - 14) / 3, // Scores élevés : réduction modérée
            _ => 14 + (raw_score - 18) / 4,       // Scores très élevés : réduction forte
        };

        capped_score.max(1).min(20)
    }

    pub fn generate_random() -> Self {
        let mut rng = thread_rng();

        let breeds = get_cat_breeds();
        let colors = get_cat_colors();
        let names = get_cat_names();

        // Sélection pondérée pour les races (plus rarity_bonus est élevé, plus c'est rare)
        let breed = select_weighted_breed(&breeds, &mut rng);

        // Sélection pondérée pour les couleurs
        let color = select_weighted_color(&colors, &mut rng);

        // Nom aléatoire normal
        let name = names[rng.gen_range(0..names.len())].to_string();
        let age_months = rng.gen_range(1..=60); // Jusqu'à 5 ans

        let mut cat = Cat {
            breed,
            color,
            age_months,
            name,
            rarity_score: 0,
        };

        cat.rarity_score = cat.calculate_rarity();
        cat
    }

    pub fn format_description(&self) -> String {
        let age_display = if self.age_months <= 12 {
            format!("{} mois", self.age_months)
        } else {
            let years = self.age_months / 12;
            let months = self.age_months % 12;
            if months == 0 {
                format!("{} an{}", years, if years > 1 { "s" } else { "" })
            } else {
                format!(
                    "{} an{} et {} mois",
                    years,
                    if years > 1 { "s" } else { "" },
                    months
                )
            }
        };

        format!(
            "{} {} {} de {}",
            self.name, self.breed.name, self.color.name, age_display
        )
    }
}

pub fn get_cat_breeds() -> Vec<CatBreed> {
    vec![
        CatBreed {
            name: "Chat de gouttière",
            rarity_bonus: 0,
        },
        CatBreed {
            name: "European Shorthair",
            rarity_bonus: 1,
        },
        CatBreed {
            name: "British Shorthair",
            rarity_bonus: 2,
        },
        CatBreed {
            name: "Chartreux",
            rarity_bonus: 2,
        },
        CatBreed {
            name: "Siamois",
            rarity_bonus: 3,
        },
        CatBreed {
            name: "Ragdoll",
            rarity_bonus: 4,
        },
        CatBreed {
            name: "Birman",
            rarity_bonus: 4,
        },
        CatBreed {
            name: "Persan",
            rarity_bonus: 4,
        },
        CatBreed {
            name: "Norvégien",
            rarity_bonus: 5,
        },
        CatBreed {
            name: "Maine Coon",
            rarity_bonus: 10,
        },
        CatBreed {
            name: "Scottish Fold",
            rarity_bonus: 5,
        },
        CatBreed {
            name: "Abyssin",
            rarity_bonus: 3,
        },
        CatBreed {
            name: "Bengal",
            rarity_bonus: 6,
        },
        CatBreed {
            name: "Sibérien",
            rarity_bonus: 10,
        },
        CatBreed {
            name: "Sphinx",
            rarity_bonus: 5,
        },
        CatBreed {
            name: "Oriental",
            rarity_bonus: 3,
        },
        CatBreed {
            name: "Savannah",
            rarity_bonus: 9,
        },
        CatBreed {
            name: "Toyger",
            rarity_bonus: 7,
        },
        CatBreed {
            name: "Cornish Rex",
            rarity_bonus: 2,
        },
        CatBreed {
            name: "Devon Rex",
            rarity_bonus: 2,
        },
        CatBreed {
            name: "Balinais",
            rarity_bonus: 4,
        },
        CatBreed {
            name: "Exotic Shorthair",
            rarity_bonus: 3,
        },
        CatBreed {
            name: "Turc de Van",
            rarity_bonus: 4,
        },
        CatBreed {
            name: "Angora Turc",
            rarity_bonus: 4,
        },
        CatBreed {
            name: "Singapura",
            rarity_bonus: 6,
        },
        CatBreed {
            name: "Korat",
            rarity_bonus: 4,
        },
        CatBreed {
            name: "Manx",
            rarity_bonus: 2,
        },
        CatBreed {
            name: "Burmese",
            rarity_bonus: 3,
        },
        CatBreed {
            name: "American Curl",
            rarity_bonus: 4,
        },
        CatBreed {
            name: "Peterbald",
            rarity_bonus: 5,
        },
        CatBreed {
            name: "Lykoi",
            rarity_bonus: 1,
        },
    ]
}

pub fn get_cat_colors() -> Vec<CatColor> {
    vec![
        CatColor {
            name: "noir",
            rarity_bonus: 0,
        },
        CatColor {
            name: "blanc",
            rarity_bonus: 1,
        },
        CatColor {
            name: "gris",
            rarity_bonus: 0,
        },
        CatColor {
            name: "bleu",
            rarity_bonus: 1,
        },
        CatColor {
            name: "roux",
            rarity_bonus: 2,
        },
        CatColor {
            name: "crème",
            rarity_bonus: 1,
        },
        CatColor {
            name: "chocolat",
            rarity_bonus: 2,
        },
        CatColor {
            name: "lilas",
            rarity_bonus: 3,
        },
        CatColor {
            name: "cannelle",
            rarity_bonus: 2,
        },
        CatColor {
            name: "fauve",
            rarity_bonus: 3,
        },
        CatColor {
            name: "tigré",
            rarity_bonus: 1,
        },
        CatColor {
            name: "marbré",
            rarity_bonus: 2,
        },
        CatColor {
            name: "moucheté",
            rarity_bonus: 2,
        },
        CatColor {
            name: "ticked",
            rarity_bonus: 3,
        },
        CatColor {
            name: "écaille de tortue",
            rarity_bonus: 3,
        },
        CatColor {
            name: "calico",
            rarity_bonus: 4,
        },
        CatColor {
            name: "dilute calico",
            rarity_bonus: 4,
        },
        CatColor {
            name: "torbie",
            rarity_bonus: 4,
        },
        CatColor {
            name: "bicolore noir et blanc",
            rarity_bonus: 1,
        },
        CatColor {
            name: "bicolore roux et blanc",
            rarity_bonus: 2,
        },
        CatColor {
            name: "bicolore bleu et blanc",
            rarity_bonus: 2,
        },
        CatColor {
            name: "bicolore crème et blanc",
            rarity_bonus: 2,
        },
        CatColor {
            name: "smoke",
            rarity_bonus: 3,
        },
        CatColor {
            name: "silver",
            rarity_bonus: 3,
        },
        CatColor {
            name: "golden",
            rarity_bonus: 4,
        },
        CatColor {
            name: "colourpoint",
            rarity_bonus: 4,
        },
        CatColor {
            name: "seal point",
            rarity_bonus: 4,
        },
        CatColor {
            name: "blue point",
            rarity_bonus: 4,
        },
        CatColor {
            name: "chocolate point",
            rarity_bonus: 5,
        },
        CatColor {
            name: "lilac point",
            rarity_bonus: 5,
        },
        CatColor {
            name: "red point",
            rarity_bonus: 4,
        },
        CatColor {
            name: "cream point",
            rarity_bonus: 4,
        },
        CatColor {
            name: "sepia",
            rarity_bonus: 3,
        },
        CatColor {
            name: "mink",
            rarity_bonus: 3,
        },
    ]
}

pub fn get_cat_names() -> Vec<&'static str> {
    vec![
        // Noms courts (bonus 0)
        "Max",
        "Leo",
        "Mia",
        "Sox",
        "Rex",
        "Zoe",
        "Ace",
        "Rio",
        // Noms moyens (bonus 1)
        "Luna",
        "Felix",
        "Bella",
        "Oscar",
        "Milo",
        "Chloe",
        "Zeus",
        "Nala",
        "Tiger",
        "Smoky",
        "Pearl",
        "Storm",
        // Noms longs (bonus 2)
        "Whiskers",
        "Shadow",
        "Princess",
        "Midnight",
        "Snowball",
        "Pumpkin",
        "Biscuit",
        "Caramel",
        "Thunder",
        "Duchess",
        "Reyna",
        "Aslan",
        // Noms très longs (bonus 3)
        "Buttercup",
        "Cinnamon",
        "Marshmallow",
        "Thunderbolt",
        "Strawberry",
        "Firecracker",
        "Blueberry",
        "Chocolate",
        // Noms avec voyelles (bonus +1)
        "Oliver",
        "Emma",
        "Oreo",
        "Angel",
        "Echo",
        "Iris",
        "Amber",
        "Opal",
        "Uma",
        "Ivy",
        "Aria",
        "Aspen",
        "Oakley",
        "Ember",
    ]
}

// Sélection pondérée pour les races (plus rarity_bonus est élevé, plus c'est rare)
pub fn select_weighted_breed(breeds: &[CatBreed], rng: &mut impl Rng) -> CatBreed {
    // Calculer les poids inversés : plus rarity_bonus est élevé, plus le poids est faible
    let weights: Vec<f32> = breeds
        .iter()
        .map(|breed| {
            // Poids inversé avec pénalité modérée pour les races rares
            let base_weight = 1000.0;
            let penalty = match breed.rarity_bonus {
                0 => 0.0,
                1..=2 => breed.rarity_bonus as f32 * 30.0,
                3..=5 => breed.rarity_bonus as f32 * 80.0,
                6..=8 => breed.rarity_bonus as f32 * 120.0,
                _ => breed.rarity_bonus as f32 * 160.0, // Races légendaires rares mais pas impossibles
            };
            (base_weight - penalty).max(5.0) // Minimum raisonnable
        })
        .collect();

    // Sélection pondérée
    let total_weight: f32 = weights.iter().sum();
    let mut random_weight = rng.gen::<f32>() * total_weight;

    for (i, weight) in weights.iter().enumerate() {
        random_weight -= weight;
        if random_weight <= 0.0 {
            return breeds[i].clone();
        }
    }

    // Fallback (ne devrait jamais arriver)
    breeds[0].clone()
}

// Sélection pondérée pour les couleurs
pub fn select_weighted_color(colors: &[CatColor], rng: &mut impl Rng) -> CatColor {
    // Calculer les poids inversés avec pénalité exponentielle pour les couleurs rares
    let weights: Vec<f32> = colors
        .iter()
        .map(|color| {
            let base_weight = 1000.0;
            let penalty = match color.rarity_bonus {
                0 => 0.0,
                1..=2 => color.rarity_bonus as f32 * 50.0,
                3..=4 => color.rarity_bonus as f32 * 100.0,
                _ => color.rarity_bonus as f32 * 150.0, // Couleurs légendaires rares mais possibles
            };
            (base_weight - penalty).max(8.0)
        })
        .collect();

    // Sélection pondérée
    let total_weight: f32 = weights.iter().sum();
    let mut random_weight = rng.gen::<f32>() * total_weight;

    for (i, weight) in weights.iter().enumerate() {
        random_weight -= weight;
        if random_weight <= 0.0 {
            return colors[i].clone();
        }
    }

    // Fallback
    colors[0].clone()
}

#[command]
#[description = "Gagne un Daily Cat (avec parfois un chat secret bonus !)"]
pub async fn cat(ctx: &Context, msg: &Message) -> CommandResult {
    let data = ctx.data.read().await;
    let pool = data
        .get::<DatabasePool>()
        .expect("Impossible d'obtenir le pool");

    let user = match get_user_by_discord_id(&pool, msg.author.id.0).await {
        Ok(Some(u)) => u,
        Ok(None) => {
            let user_id = new_user(&pool, msg.author.id.0, &msg.author.name).await?;
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

    if has_daily_cat_today(&pool, user.id).await.unwrap_or(false) {
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
    if add_daily_cat(&pool, user.id).await.is_err() {
        msg.channel_id
            .say(&ctx.http, "Impossible d'ajouter le Cat")
            .await
            .ok();
        return Ok(());
    }

    let total = match get_daily_cat_count(&pool, user.id).await {
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
            &pool,
            user.id,
            &secret_cat.name,
            secret_cat.breed.name,
            secret_cat.color.name,
            secret_cat.age_months,
            secret_cat.rarity_score,
        )
        .await
        {
            Ok(cat_id) => {
                let rarity_emoji = get_rarity_emoji(secret_cat.rarity_score);
                let temperament = match get_cat_by_id(&pool, cat_id).await {
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
    if matches!(get_daily_cat_count_today(&pool).await, Ok(1)) {
        maybe_trigger_cat_event(ctx, msg, &pool).await;
    }

    Ok(())
}

pub fn get_rarity_badge(score: i32) -> &'static str {
    match score {
        0..=5 => "",                    // Commun - pas de badge
        6..=8 => "✨ Spécial",          // Peu commun
        9..=11 => "🌟 Remarquable",     // Rare
        12..=14 => "💎 Exceptionnel",   // Épique
        15..=17 => "👑 Extraordinaire", // Légendaire
        18..=20 => "🔥 Mythique",       // Ultra-légendaire
        _ => "",
    }
}

pub fn get_rarity_emoji(score: i32) -> &'static str {
    match score {
        0..=5 => "🐾",   // Commun
        6..=10 => "✨",  // Peu commun
        11..=15 => "🌟", // Rare
        16..=18 => "💎", // Épique
        19..=20 => "👑", // Légendaire
        _ => "🐾",
    }
}

fn format_age(age_months: i32) -> String {
    if age_months <= 12 {
        format!("{} mois", age_months)
    } else {
        let years = age_months / 12;
        let months = age_months % 12;
        if months == 0 {
            format!("{} an{}", years, if years > 1 { "s" } else { "" })
        } else {
            format!(
                "{} an{} et {} mois",
                years,
                if years > 1 { "s" } else { "" },
                months
            )
        }
    }
}

fn cat_display_name(cat: &CollectedCat) -> String {
    match &cat.nickname {
        Some(nickname) if !nickname.trim().is_empty() => {
            format!("{} \"{}\"", cat.name, nickname.trim())
        }
        _ => cat.name.clone(),
    }
}

fn format_cat_identity(cat: &CollectedCat) -> String {
    let rarity_emoji = get_rarity_emoji(cat.rarity_score);
    let rarity_badge = get_rarity_badge(cat.rarity_score);
    let badge = if rarity_badge.is_empty() {
        String::new()
    } else {
        format!(" • {}", rarity_badge)
    };

    format!(
        "{} **{} {} {} de {}**{} (#{})",
        rarity_emoji,
        cat_display_name(cat),
        cat.breed,
        cat.color,
        format_age(cat.age_months),
        badge,
        cat.id
    )
}

fn format_cat_scene(cat: &CollectedCat) -> String {
    let mood = cat
        .mood
        .as_deref()
        .filter(|value| {
            let value = value.trim();
            !value.is_empty() && value != "observe la piece en silence"
        })
        .unwrap_or_else(|| daily_house_scene(cat));

    format!("{} {}", cat_display_name(cat), mood)
}

fn format_mycats_page(
    owner_name: &str,
    cats: &[CollectedCat],
    requested_page: usize,
) -> (String, usize, usize) {
    let per_page = 10usize;
    let total_pages = ((cats.len() + per_page - 1) / per_page).max(1);
    let page = requested_page.clamp(1, total_pages);
    let start = (page - 1) * per_page;
    let end = (start + per_page).min(cats.len());
    let mut response = format!(
        "🏠 **Les chats qui vivent chez {}** 🏠\nPage {}/{} • {} résidents\n\n",
        owner_name,
        page,
        total_pages,
        cats.len()
    );

    for (index, cat) in cats[start..end].iter().enumerate() {
        let rarity_emoji = get_rarity_emoji(cat.rarity_score);
        let rarity_badge = get_rarity_badge(cat.rarity_score);
        let badge_display = if !rarity_badge.is_empty() {
            format!(" • {}", rarity_badge)
        } else {
            String::new()
        };

        response += &format!(
            "{}. {} **{} {} {} de {}**{} — ID: `{}`\n",
            start + index + 1,
            rarity_emoji,
            cat_display_name(cat),
            cat.breed,
            cat.color,
            format_age(cat.age_months),
            badge_display,
            cat.id
        );
    }

    if total_pages > 1 {
        response += "\n";
        if page > 1 {
            response += &format!("Page précédente: `^^mycats {}`\n", page - 1);
        }
        if page < total_pages {
            response += &format!("Page suivante: `^^mycats {}`\n", page + 1);
        }
        response += "Réagis avec ◀️ / ▶️ pour changer de page.\n";
    }

    response += &format!("\n🏠 **Total: {} chats vivent chez toi**", cats.len());
    (response, page, total_pages)
}

async fn handle_mycats_reactions(
    ctx: &Context,
    author_id: UserId,
    mut page_message: Message,
    owner_name: String,
    cats: Vec<CollectedCat>,
    start_page: usize,
    total_pages: usize,
) {
    let previous = ReactionType::Unicode("◀️".to_string());
    let next = ReactionType::Unicode("▶️".to_string());
    page_message.react(&ctx.http, previous.clone()).await.ok();
    page_message.react(&ctx.http, next.clone()).await.ok();

    let mut current_page = start_page;
    let started_at = std::time::Instant::now();
    let timeout = std::time::Duration::from_secs(120);

    while started_at.elapsed() < timeout {
        tokio::time::sleep(std::time::Duration::from_millis(500)).await;

        let mut requested_page = current_page;

        if current_page > 1 && reaction_has_user(ctx, &page_message, &previous, author_id).await {
            requested_page -= 1;
        } else if current_page < total_pages && reaction_has_user(ctx, &page_message, &next, author_id).await {
            requested_page += 1;
        }

        if requested_page == current_page {
            continue;
        }

        current_page = requested_page;
        let (content, _, _) = format_mycats_page(&owner_name, &cats, current_page);
        page_message
            .edit(&ctx.http, |message| message.content(content))
            .await
            .ok();
        page_message
            .delete_reaction_emoji(&ctx.http, previous.clone())
            .await
            .ok();
        page_message
            .delete_reaction_emoji(&ctx.http, next.clone())
            .await
            .ok();
        page_message.react(&ctx.http, previous.clone()).await.ok();
        page_message.react(&ctx.http, next.clone()).await.ok();
    }
}

async fn reaction_has_user(
    ctx: &Context,
    message: &Message,
    reaction: &ReactionType,
    user_id: UserId,
) -> bool {
    match message
        .reaction_users(&ctx.http, reaction.clone(), None, None)
        .await
    {
        Ok(users) => users.iter().any(|user| user.id == user_id),
        Err(_) => false,
    }
}

fn daily_house_scene(cat: &CollectedCat) -> &'static str {
    let scenes = [
        "dort en boule sur le canapé",
        "surveille la fenêtre comme si une mission lui avait été confiée",
        "s'est installé dans une boîte beaucoup trop petite",
        "tapote une poussière invisible sous le meuble",
        "fait semblant de ne pas entendre son nom",
        "attend devant une gamelle pourtant pleine",
        "patrouille lentement dans le couloir",
        "s'étire au milieu du passage",
        "s'est approprié le meilleur coussin",
        "inspecte un sac posé par terre",
        "regarde fixement un coin vide de la pièce",
        "dort sur du linge propre",
        "essaie d'ouvrir une porte fermée",
        "suit quelqu'un de pièce en pièce",
        "se cache sous une chaise mais laisse dépasser sa queue",
        "renifle une tasse avec beaucoup de sérieux",
        "fait tomber un petit objet puis quitte la pièce",
        "se roule sur le tapis",
        "réclame des câlins puis change d'avis",
        "observe la pluie contre la vitre",
        "se pose exactement là où il gêne le plus",
        "attend qu'une boîte soit disponible",
        "s'endort à moitié assis",
        "gratte doucement près d'une porte",
        "chasse une ombre au sol",
        "surveille les nouveaux résidents",
        "fait sa toilette avec une concentration totale",
        "s'assoit sur un vêtement noir",
        "marche sur la table avec une fausse discrétion",
        "réclame l'attention sans faire de bruit",
        "se cache derrière un rideau",
        "a trouvé un rayon de soleil stratégique",
        "fixe sa gamelle comme si elle allait se remplir seule",
        "fait la sieste près du refuge à couvertures",
        "joue avec une chaussette abandonnée",
        "vient saluer puis repart aussitôt",
        "s'installe près de la personne la plus occupée",
        "fait tomber un coussin pour mieux dormir",
        "observe les autres chats d'un air très officiel",
        "s'endort contre un mur chaud",
        "se poste devant la fenêtre comme un gardien",
        "renverse une petite couverture pour en faire un nid",
        "semble préparer une bêtise",
        "vient poser une patte sur le bord du bureau",
        "fait une course soudaine dans le couloir",
        "se couche pile au centre de la pièce",
        "inspecte le dessous du canapé",
        "dort avec une patte sur les yeux",
        "attend poliment une place libre",
        "fait semblant d'être affamé",
        "s'approche pour écouter la conversation",
        "se frotte contre un meuble comme s'il lui appartenait",
        "observe une mouche avec intensité",
        "a choisi une couverture et refuse de la partager",
        "s'installe dans le passage puis juge tout le monde",
        "grimpe sur une chaise pour mieux superviser la maison",
        "fait un petit bruit pour demander quelque chose",
        "se cache dans un endroit évident",
        "regarde dehors comme s'il attendait quelqu'un",
        "ronronne discrètement près du canapé",
    ];
    let day_seed = paris_today().num_days_from_ce() as usize;
    let cat_seed = cat.id as usize + cat.age_months as usize + cat.rarity_score as usize;
    scenes[(cat_seed + day_seed) % scenes.len()]
}

#[command]
#[description = "Affiche ta collection de chats"]
pub async fn mycats(ctx: &Context, msg: &Message, mut args: Args) -> CommandResult {
    let data = ctx.data.read().await;
    let pool = data
        .get::<DatabasePool>()
        .expect("Impossible d'obtenir le pool");
    let requested_page = args.single::<usize>().unwrap_or(1).max(1);

    let user = match get_user_by_discord_id(&pool, msg.author.id.0).await {
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

    match get_user_cats(&pool, user.id).await {
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

    match get_user_by_discord_id(&pool, msg.author.id.0).await {
        Ok(Some(user)) => match get_daily_cat_count(&pool, user.id).await {
            Ok(total) => {
                let counts = get_user_cat_counts(&pool, user.id).await.ok();
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

    let user = match get_user_by_discord_id(&pool, msg.author.id.0).await {
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
    match get_cat_by_id(&pool, cat_id).await {
        Ok(Some(cat)) => {
            if cat.user_id != user.id as i32 {
                msg.channel_id
                    .say(&ctx.http, "Ce chat ne t'appartient pas !")
                    .await
                    .ok();
                return Ok(());
            }

            // Récupérer l'utilisateur cible
            let target_user = match crate::commands::get_user_by_mention(ctx, msg, args[1]).await {
                Ok(u) => u,
                Err(_) => return Ok(()),
            };

            let target_db_user = match get_user_by_discord_id(&pool, target_user.id.0).await {
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
            match transfer_cat(&pool, cat_id, target_db_user.id).await {
                Ok(_) => {
                    let rarity_emoji = get_rarity_emoji(cat.rarity_score);

                    let age_display = if cat.age_months <= 12 {
                        format!("{} mois", cat.age_months)
                    } else {
                        let years = cat.age_months / 12;
                        let months = cat.age_months % 12;
                        if months == 0 {
                            format!("{} an{}", years, if years > 1 { "s" } else { "" })
                        } else {
                            format!(
                                "{} an{} et {} mois",
                                years,
                                if years > 1 { "s" } else { "" },
                                months
                            )
                        }
                    };

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

async fn show_house(ctx: &Context, msg: &Message, is_visit: bool) -> CommandResult {
    let data = ctx.data.read().await;
    let pool = data
        .get::<DatabasePool>()
        .expect("Impossible d'obtenir le pool");

    let target = msg
        .mentions
        .first()
        .cloned()
        .unwrap_or_else(|| msg.author.clone());
    let user = match get_user_by_discord_id(&pool, target.id.0).await {
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

    let daily_total = get_daily_cat_count(&pool, user.id).await.unwrap_or(0);
    let counts = get_user_cat_counts(&pool, user.id)
        .await
        .unwrap_or(crate::database::CatCounts {
            home: 0,
            refuge: 0,
            total: 0,
        });
    let cats = get_user_cats(&pool, user.id).await.unwrap_or_default();

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

    match get_refuge_cats(&pool, 12).await {
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
    let user = match get_user_by_discord_id(&pool, msg.author.id.0).await {
        Ok(Some(user)) => user,
        _ => {
            msg.channel_id
                .say(&ctx.http, "Tu n'as pas encore de profil.")
                .await
                .ok();
            return Ok(());
        }
    };

    let cat = match get_cat_by_id(&pool, cat_id).await {
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

    if move_cat_to_refuge(&pool, cat_id, user.id).await.is_err() {
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
    let user = match get_user_by_discord_id(&pool, msg.author.id.0).await {
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

    if set_cat_nickname(&pool, cat_id, user.id, new_nickname)
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
    let user = match get_user_by_discord_id(&pool, msg.author.id.0).await {
        Ok(Some(user)) => user,
        _ => {
            msg.channel_id
                .say(&ctx.http, "Tu n'as pas encore de profil.")
                .await
                .ok();
            return Ok(());
        }
    };

    match get_cat_by_id(&pool, cat_id).await {
        Ok(Some(cat)) if cat.user_id == user.id as i32 && cat.location == "home" => {
            set_favorite_cat(&pool, user.id, cat_id).await?;
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

    let cat = match get_cat_by_id(&pool, cat_id).await {
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

    if let Ok(memories) = get_cat_memories(&pool, cat.id, 4).await {
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

    match get_cat_server_stats(&pool).await {
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

async fn join_cat_event(
    ctx: &Context,
    msg: &Message,
    kind: CatEventKind,
    no_event_message: &str,
) -> CommandResult {
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
        let action = match event.kind {
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

async fn take_cat_event(ctx: &Context, channel_id: ChannelId) -> Option<CatEvent> {
    let events = {
        let data = ctx.data.read().await;
        data.get::<CatEventContainer>()?.clone()
    };

    let mut events = events.lock().await;
    events.remove(&channel_id.0)
}

async fn maybe_trigger_cat_event(ctx: &Context, msg: &Message, pool: &sqlx::Pool<sqlx::MySql>) {
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
        let event_kind = match kind {
            CatEventKind::Wild => "wild",
            CatEventKind::Adoption => "adoption",
        };
        record_cat_event_start(pool, msg.channel_id.0, event_kind, theme.map(|theme| theme.key)).await.ok();
    }
}

async fn channel_has_cat_event(ctx: &Context, channel_id: ChannelId) -> bool {
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

async fn start_cat_event(
    ctx: &Context,
    channel_id: ChannelId,
    kind: CatEventKind,
    theme: Option<CatEventTheme>,
) -> bool {
    let (duration_secs, duration_label) = cat_event_duration();
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
                kind: kind.clone(),
                participants: HashSet::new(),
                theme,
            },
        );
    }

    match kind {
        CatEventKind::Wild => {
            let intro = match theme {
                Some(theme) => format!("{}\nUn chat sauvage spécial rôde près du serveur...", theme.intro),
                None => "Un chat sauvage rôde près du serveur...".to_string(),
            };
            channel_id.say(&ctx.http, format!(
                "{}\nUtilisez `^^caliner` {} pour tenter de gagner sa confiance.",
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
                "{}\nUtilisez `^^adopter` {}. Un résident du refuge choisira une maison.",
                intro,
                duration_label
            )).await.ok();
            spawn_adoption_resolution(ctx.clone(), channel_id, duration_secs).await;
        }
    }

    true
}

fn cat_event_duration() -> (u64, &'static str) {
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

fn cat_event_theme_for_date(date: NaiveDate) -> Option<CatEventTheme> {
    let month = date.month();
    let day = date.day();
    let easter = easter_sunday(date.year());
    let mothers_day = french_mothers_day(date.year());
    let fathers_day = nth_weekday_of_month(date.year(), 6, Weekday::Sun, 3);
    let grandmothers_day = nth_weekday_of_month(date.year(), 3, Weekday::Sun, 1);
    let grandfathers_day = nth_weekday_of_month(date.year(), 10, Weekday::Sun, 1);
    let neighbours_day = last_weekday_of_month(date.year(), 5, Weekday::Fri);

    if month == 12 && (20..=26).contains(&day) {
        Some(CatEventTheme {
            key: "noel",
            intro: "🎄 Ambiance de Noël: les chats semblent attirés par les guirlandes.",
            memory: "A rejoint sa maison pendant un événement de Noël.",
            names: &["Noel", "Neige", "Grelot", "Sapin", "Flocon", "Holly"],
            breeds: &["Norvégien", "Sibérien", "Maine Coon", "Ragdoll"],
            colors: &["blanc", "silver", "golden", "colourpoint"],
            rarity_bonus: 3,
        })
    } else if month == 10 && (25..=31).contains(&day) {
        Some(CatEventTheme {
            key: "halloween",
            intro: "🎃 Ambiance d'Halloween: un chat mystérieux se faufile dans l'ombre.",
            memory: "A rejoint sa maison pendant un événement d'Halloween.",
            names: &["Ombre", "Citrouille", "Minuit", "Spectre", "Rune", "Salem"],
            breeds: &["Lykoi", "Sphinx", "Peterbald", "Oriental"],
            colors: &["noir", "smoke", "écaille de tortue", "chocolat"],
            rarity_bonus: 3,
        })
    } else if date >= easter - chrono::Duration::days(2) && date <= easter + chrono::Duration::days(2) {
        Some(CatEventTheme {
            key: "paques",
            intro: "🐣 Ambiance de Pâques: un chat curieux suit une piste de petites surprises.",
            memory: "A rejoint sa maison pendant un événement de Pâques.",
            names: &["Cacao", "Praline", "Lapinou", "Muguet", "Panier", "Choco"],
            breeds: &["British Shorthair", "Ragdoll", "Birman", "Exotic Shorthair"],
            colors: &["chocolat", "crème", "lilas", "golden"],
            rarity_bonus: 2,
        })
    } else if date == grandmothers_day {
        Some(CatEventTheme {
            key: "fete_grand_meres",
            intro: "🌷 Fête des grands-mères: un chat cherche un foyer plein d'histoires et de douceur.",
            memory: "A rejoint sa maison pendant la Fête des grands-mères.",
            names: &["Mamie", "Madeleine", "Biscotte", "Suzette", "Tisane", "Rose"],
            breeds: &["Persan", "British Shorthair", "Chartreux", "Ragdoll"],
            colors: &["crème", "gris", "lilas", "blanc"],
            rarity_bonus: 2,
        })
    } else if date == grandfathers_day {
        Some(CatEventTheme {
            key: "fete_grand_peres",
            intro: "🧶 Fête des grands-pères: un chat tranquille inspecte les fauteuils du refuge.",
            memory: "A rejoint sa maison pendant la Fête des grands-pères.",
            names: &["Papi", "Gaston", "Marcel", "Brioche", "Canne", "Moka"],
            breeds: &["Chartreux", "European Shorthair", "Korat", "British Shorthair"],
            colors: &["gris", "bleu", "tigré", "chocolat"],
            rarity_bonus: 2,
        })
    } else if date == mothers_day {
        Some(CatEventTheme {
            key: "fete_meres",
            intro: "💐 Fête des mères: un chat délicat arrive avec une humeur très câline.",
            memory: "A rejoint sa maison pendant la Fête des mères.",
            names: &["Maman", "Rose", "Douce", "Fleur", "Coton", "Cherie"],
            breeds: &["Ragdoll", "Birman", "Persan", "British Shorthair"],
            colors: &["crème", "blanc", "calico", "dilute calico"],
            rarity_bonus: 3,
        })
    } else if date == fathers_day {
        Some(CatEventTheme {
            key: "fete_peres",
            intro: "🧰 Fête des pères: un chat solide prétend savoir réparer une étagère.",
            memory: "A rejoint sa maison pendant la Fête des pères.",
            names: &["Papa", "Bricole", "Atlas", "Moustache", "Galet", "Cargo"],
            breeds: &["Maine Coon", "Norvégien", "Chartreux", "Sibérien"],
            colors: &["gris", "bleu", "tigré", "bicolore noir et blanc"],
            rarity_bonus: 3,
        })
    } else if month == 5 && day == 15 {
        Some(CatEventTheme {
            key: "familles",
            intro: "🏡 Journée des familles: un chat cherche une maison où tout le monde a sa place.",
            memory: "A rejoint sa maison pendant la Journée des familles.",
            names: &["Foyer", "Nid", "Tribu", "Coussin", "Maison", "Lien"],
            breeds: &["European Shorthair", "Ragdoll", "Birman", "Chat de gouttière"],
            colors: &["bicolore noir et blanc", "calico", "tigré", "crème"],
            rarity_bonus: 2,
        })
    } else if month == 5 && day == 1 {
        Some(CatEventTheme {
            key: "fete_travail",
            intro: "✊ Fête du Travail: même les chats réclament une pause digne.",
            memory: "A rejoint sa maison pendant la Fête du Travail.",
            names: &["Repos", "Pause", "Manif", "Solidarite", "Muguet", "Camarade"],
            breeds: &["Chat de gouttière", "European Shorthair", "Chartreux", "Manx"],
            colors: &["blanc", "roux", "tigré", "bicolore noir et blanc"],
            rarity_bonus: 1,
        })
    } else if month == 3 && day == 8 {
        Some(CatEventTheme {
            key: "droits_femmes",
            intro: "💜 Journée des droits des femmes: le refuge accueille tout le monde avec respect.",
            memory: "A rejoint sa maison pendant la Journée des droits des femmes.",
            names: &["Olympe", "Simone", "Rosa", "Ada", "Frida", "Gisele"],
            breeds: &["Abyssin", "Siamois", "Oriental", "Bengal"],
            colors: &["lilas", "fauve", "calico", "torbie"],
            rarity_bonus: 2,
        })
    } else if month == 6 && day == 21 {
        Some(CatEventTheme {
            key: "musique",
            intro: "🎵 Fête de la musique: un chat semble suivre le rythme.",
            memory: "A rejoint sa maison pendant la Fête de la musique.",
            names: &["Tempo", "Jazz", "Melodie", "Solo", "Disco", "Riff"],
            breeds: &["Oriental", "Balinais", "Cornish Rex", "Devon Rex"],
            colors: &["ticked", "silver", "blue point", "red point"],
            rarity_bonus: 2,
        })
    } else if month == 6 && [1, 15, 28].contains(&day) {
        Some(CatEventTheme {
            key: "fiertes",
            intro: "🏳️‍🌈 Fiertés: le refuge ouvre grand ses portes.",
            memory: "A rejoint sa maison pendant un événement des fiertés.",
            names: &["Pride", "Iris", "Arc", "Nova", "Libre", "Pixel"],
            breeds: &["Bengal", "Toyger", "Savannah", "Singapura"],
            colors: &["calico", "torbie", "golden", "colourpoint"],
            rarity_bonus: 3,
        })
    } else if month == 1 && day == 1 {
        Some(CatEventTheme {
            key: "nouvel_an",
            intro: "✨ Nouvel An: un nouveau départ attire les chats errants.",
            memory: "A rejoint sa maison pendant le Nouvel An.",
            names: &["Aurore", "Minuit", "Nova", "Voeu", "Etincelle", "Janus"],
            breeds: &["Sibérien", "Maine Coon", "Savannah", "Korat"],
            colors: &["silver", "golden", "blanc", "smoke"],
            rarity_bonus: 3,
        })
    } else if month == 2 && day == 14 {
        Some(CatEventTheme {
            key: "saint_valentin",
            intro: "💌 Saint-Valentin: un chat cherche une maison pleine d'affection.",
            memory: "A rejoint sa maison pendant la Saint-Valentin.",
            names: &["Amour", "Coeur", "Cupidon", "Rose", "Velours", "Bisou"],
            breeds: &["Ragdoll", "Persan", "Birman", "British Shorthair"],
            colors: &["roux", "crème", "red point", "cream point"],
            rarity_bonus: 2,
        })
    } else if month == 2 && day == 4 {
        Some(CatEventTheme {
            key: "cancer",
            intro: "🎗️ Journée contre le cancer: le refuge se fait plus doux et plus patient.",
            memory: "A rejoint sa maison pendant la Journée contre le cancer.",
            names: &["Ruban", "Courage", "Espoir", "Soin", "Lumiere", "Veille"],
            breeds: &["Ragdoll", "Birman", "British Shorthair", "Chartreux"],
            colors: &["blanc", "crème", "silver", "lilas"],
            rarity_bonus: 2,
        })
    } else if month == 3 && day == 22 {
        Some(CatEventTheme {
            key: "eau",
            intro: "💧 Journée mondiale de l'eau: un chat suit le bruit d'une fontaine.",
            memory: "A rejoint sa maison pendant la Journée mondiale de l'eau.",
            names: &["Onde", "Goutte", "Ruisseau", "Pluie", "Source", "Brume"],
            breeds: &["Turc de Van", "Sibérien", "Korat", "Bengal"],
            colors: &["bleu", "blue point", "silver", "gris"],
            rarity_bonus: 2,
        })
    } else if month == 3 && day == 21 {
        Some(CatEventTheme {
            key: "forets",
            intro: "🌲 Journée des forêts: un chat revient avec l'air d'avoir exploré les sous-bois.",
            memory: "A rejoint sa maison pendant la Journée internationale des forêts.",
            names: &["Sylve", "Cedre", "Mousse", "Racine", "Ecorce", "Fougère"],
            breeds: &["Norvégien", "Maine Coon", "Sibérien", "Bengal"],
            colors: &["tigré", "fauve", "golden", "marbré"],
            rarity_bonus: 2,
        })
    } else if month == 4 && day == 7 {
        Some(CatEventTheme {
            key: "sante",
            intro: "🩺 Journée mondiale de la santé: un chat calme vient rappeler de souffler un peu.",
            memory: "A rejoint sa maison pendant la Journée mondiale de la santé.",
            names: &["Sante", "Calme", "Remede", "Repos", "Pulse", "Baume"],
            breeds: &["Ragdoll", "British Shorthair", "Birman", "Chartreux"],
            colors: &["blanc", "crème", "gris", "colourpoint"],
            rarity_bonus: 2,
        })
    } else if month == 9 && (1..=7).contains(&day) {
        Some(CatEventTheme {
            key: "rentree",
            intro: "🎒 Rentrée: un chat inspecte les cartables et les nouveaux projets.",
            memory: "A rejoint sa maison pendant la rentrée.",
            names: &["Cartable", "Craie", "Page", "Plume", "Bureau", "Recre"],
            breeds: &["European Shorthair", "Chartreux", "Siamois", "Korat"],
            colors: &["gris", "bleu", "tigré", "moucheté"],
            rarity_bonus: 1,
        })
    } else if month == 9 && day == 21 {
        Some(CatEventTheme {
            key: "paix",
            intro: "🕊️ Journée de la paix: un chat avance sans bruit et choisit le calme.",
            memory: "A rejoint sa maison pendant la Journée internationale de la paix.",
            names: &["Paix", "Colombe", "Silence", "Havre", "Accord", "Doux"],
            breeds: &["Birman", "Ragdoll", "Korat", "Persan"],
            colors: &["blanc", "crème", "silver", "blue point"],
            rarity_bonus: 2,
        })
    } else if month == 9 && day == 29 {
        Some(CatEventTheme {
            key: "coeur",
            intro: "❤️ Journée du cœur: un chat cherche une maison pleine d'attention.",
            memory: "A rejoint sa maison pendant la Journée mondiale du cœur.",
            names: &["Coeur", "Pulse", "Tempo", "Rouge", "Vivant", "Battement"],
            breeds: &["Ragdoll", "Siamois", "Birman", "British Shorthair"],
            colors: &["roux", "red point", "crème", "calico"],
            rarity_bonus: 2,
        })
    } else if month == 11 && day == 1 {
        Some(CatEventTheme {
            key: "toussaint",
            intro: "🕯️ Toussaint: le refuge est particulièrement calme aujourd'hui.",
            memory: "A rejoint sa maison pendant la Toussaint.",
            names: &["Brume", "Cierge", "Memoire", "Sauge", "Silence", "Veille"],
            breeds: &["Chartreux", "Korat", "British Shorthair", "Lykoi"],
            colors: &["gris", "smoke", "bleu", "noir"],
            rarity_bonus: 2,
        })
    } else if month == 10 && day == 10 {
        Some(CatEventTheme {
            key: "sante_mentale",
            intro: "🧠 Journée de la santé mentale: un chat tranquille vient prendre de la place sans pression.",
            memory: "A rejoint sa maison pendant la Journée de la santé mentale.",
            names: &["Pause", "Nuage", "Respire", "Ancre", "Doux", "Havre"],
            breeds: &["Ragdoll", "Chartreux", "British Shorthair", "Persan"],
            colors: &["gris", "crème", "bleu", "lilas"],
            rarity_bonus: 2,
        })
    } else if month == 10 && day == 16 {
        Some(CatEventTheme {
            key: "alimentation",
            intro: "🥣 Journée de l'alimentation: un chat inspecte les gamelles avec sérieux.",
            memory: "A rejoint sa maison pendant la Journée mondiale de l'alimentation.",
            names: &["Biscuit", "Soupe", "Miette", "Cacao", "Noisette", "Bol"],
            breeds: &["British Shorthair", "Burmese", "European Shorthair", "Exotic Shorthair"],
            colors: &["chocolat", "crème", "cannelle", "roux"],
            rarity_bonus: 1,
        })
    } else if month == 12 && day == 3 {
        Some(CatEventTheme {
            key: "handicap",
            intro: "♿ Journée du handicap: le refuge rappelle que chaque maison peut s'adapter.",
            memory: "A rejoint sa maison pendant la Journée internationale des personnes handicapées.",
            names: &["Acces", "Rampe", "Patience", "Force", "Egal", "Soutien"],
            breeds: &["Manx", "European Shorthair", "Chartreux", "Ragdoll"],
            colors: &["gris", "blanc", "tigré", "bicolore bleu et blanc"],
            rarity_bonus: 2,
        })
    } else if month == 12 && day == 1 {
        Some(CatEventTheme {
            key: "sida",
            intro: "🔴 Journée de lutte contre le sida: un chat arrive avec un ruban de solidarité.",
            memory: "A rejoint sa maison pendant la Journée de lutte contre le sida.",
            names: &["Ruban", "Solidarite", "Rouge", "Memoire", "Soutien", "Vie"],
            breeds: &["Siamois", "Oriental", "Burmese", "Birman"],
            colors: &["roux", "red point", "bicolore roux et blanc", "crème"],
            rarity_bonus: 2,
        })
    } else if month == 4 && day == 1 {
        Some(CatEventTheme {
            key: "poisson_avril",
            intro: "🐟 Poisson d'avril: un chat fait semblant de ne rien préparer.",
            memory: "A rejoint sa maison pendant le Poisson d'avril.",
            names: &["Farce", "Sardine", "Blague", "Malice", "Surprise", "Pixel"],
            breeds: &["Devon Rex", "Cornish Rex", "Manx", "Burmese"],
            colors: &["moucheté", "marbré", "tigré", "bicolore bleu et blanc"],
            rarity_bonus: 1,
        })
    } else if month == 4 && day == 22 {
        Some(CatEventTheme {
            key: "jour_terre",
            intro: "🌍 Jour de la Terre: un chat revient d'une promenade entre les feuilles.",
            memory: "A rejoint sa maison pendant le Jour de la Terre.",
            names: &["Gaia", "Mousse", "Feuille", "Ronce", "Terra", "Lichen"],
            breeds: &["Norvégien", "Maine Coon", "Sibérien", "Bengal"],
            colors: &["tigré", "golden", "fauve", "marbré"],
            rarity_bonus: 2,
        })
    } else if month == 5 && day == 22 {
        Some(CatEventTheme {
            key: "biodiversite",
            intro: "🌿 Journée de la biodiversité: un chat rare observe les petites vies autour de lui.",
            memory: "A rejoint sa maison pendant la Journée de la biodiversité.",
            names: &["Faune", "Flore", "Lichen", "Prairie", "Abeille", "Ronce"],
            breeds: &["Bengal", "Savannah", "Toyger", "Norvégien"],
            colors: &["tigré", "marbré", "golden", "moucheté"],
            rarity_bonus: 3,
        })
    } else if month == 5 && day == 17 {
        Some(CatEventTheme {
            key: "idahot",
            intro: "🌈 Journée contre les LGBTphobies: le refuge rappelle que chaque foyer doit être sûr.",
            memory: "A rejoint sa maison pendant la Journée contre les LGBTphobies.",
            names: &["Safe", "Libre", "Fierte", "Echo", "Iris", "Nova"],
            breeds: &["Singapura", "Toyger", "Bengal", "Oriental"],
            colors: &["calico", "dilute calico", "torbie", "colourpoint"],
            rarity_bonus: 3,
        })
    } else if month == 6 && day == 5 {
        Some(CatEventTheme {
            key: "environnement",
            intro: "🌱 Journée de l'environnement: un chat arrive avec une feuille coincée dans les moustaches.",
            memory: "A rejoint sa maison pendant la Journée mondiale de l'environnement.",
            names: &["Verte", "Feuille", "Gaia", "Compost", "Pousse", "Ortie"],
            breeds: &["Norvégien", "Sibérien", "Maine Coon", "Chat de gouttière"],
            colors: &["tigré", "fauve", "golden", "gris"],
            rarity_bonus: 2,
        })
    } else if month == 6 && day == 8 {
        Some(CatEventTheme {
            key: "oceans",
            intro: "🌊 Journée des océans: un chat semble revenir du bord de l'eau.",
            memory: "A rejoint sa maison pendant la Journée mondiale des océans.",
            names: &["Ecume", "Corail", "Vague", "Nacre", "Marin", "Algue"],
            breeds: &["Turc de Van", "Korat", "Sibérien", "Balinais"],
            colors: &["bleu", "blue point", "silver", "seal point"],
            rarity_bonus: 2,
        })
    } else if month == 6 && day == 14 {
        Some(CatEventTheme {
            key: "don_sang",
            intro: "🩸 Journée du don du sang: un chat courageux vient saluer les gestes utiles.",
            memory: "A rejoint sa maison pendant la Journée mondiale du don du sang.",
            names: &["Don", "Rouge", "Veine", "Courage", "Pulse", "Merci"],
            breeds: &["Siamois", "Burmese", "European Shorthair", "Chartreux"],
            colors: &["roux", "red point", "crème", "bicolore roux et blanc"],
            rarity_bonus: 2,
        })
    } else if date == neighbours_day {
        Some(CatEventTheme {
            key: "voisins",
            intro: "🤝 Fête des voisins: un chat passe de porte en porte comme s'il connaissait tout le monde.",
            memory: "A rejoint sa maison pendant la Fête des voisins.",
            names: &["Palier", "Bonjour", "Partage", "Cour", "Cloche", "Apéro"],
            breeds: &["Chat de gouttière", "European Shorthair", "Manx", "Burmese"],
            colors: &["tigré", "bicolore roux et blanc", "gris", "roux"],
            rarity_bonus: 1,
        })
    } else if month == 7 && day == 14 {
        Some(CatEventTheme {
            key: "fete_nationale",
            intro: "🇫🇷 Fête nationale: un chat observe les lumières avec beaucoup de sérieux.",
            memory: "A rejoint sa maison pendant la Fête nationale.",
            names: &["Bastille", "Bleuet", "Marianne", "Lumiere", "Bal", "Ruban"],
            breeds: &["Chartreux", "European Shorthair", "Birman", "Persan"],
            colors: &["bleu", "blanc", "roux", "bicolore bleu et blanc"],
            rarity_bonus: 2,
        })
    } else if month == 8 && day == 8 {
        Some(CatEventTheme {
            key: "jour_chat",
            intro: "🐱 Journée internationale du chat: les résidents ont clairement pris le pouvoir.",
            memory: "A rejoint sa maison pendant la Journée internationale du chat.",
            names: &["Majeste", "Ronron", "Moustache", "Pacha", "Velours", "Patte"],
            breeds: &["Maine Coon", "Siamois", "Ragdoll", "Bengal"],
            colors: &["golden", "silver", "colourpoint", "calico"],
            rarity_bonus: 4,
        })
    } else if month == 8 && day == 19 {
        Some(CatEventTheme {
            key: "humanitaire",
            intro: "🧡 Journée humanitaire: un chat prudent accepte enfin de s'approcher.",
            memory: "A rejoint sa maison pendant la Journée humanitaire mondiale.",
            names: &["Secours", "Abri", "Lien", "Veille", "Soin", "Main"],
            breeds: &["Chat de gouttière", "European Shorthair", "Birman", "Chartreux"],
            colors: &["blanc", "tigré", "roux", "bicolore noir et blanc"],
            rarity_bonus: 2,
        })
    } else if month == 9 && day == 10 {
        Some(CatEventTheme {
            key: "prevention_suicide",
            intro: "💛 Journée de prévention du suicide: un chat vient rappeler que personne ne devrait rester seul.",
            memory: "A rejoint sa maison pendant la Journée de prévention du suicide.",
            names: &["Ancre", "Lueur", "Ecoute", "Présence", "Soutien", "Demain"],
            breeds: &["Ragdoll", "Birman", "Chartreux", "British Shorthair"],
            colors: &["golden", "crème", "blanc", "silver"],
            rarity_bonus: 2,
        })
    } else if month == 7 && day == 30 {
        Some(CatEventTheme {
            key: "amitie",
            intro: "🫶 Journée de l'amitié: un chat sociable cherche quelqu'un à suivre partout.",
            memory: "A rejoint sa maison pendant la Journée de l'amitié.",
            names: &["Copain", "Amie", "Buddy", "Lien", "Tandem", "Soleil"],
            breeds: &["Ragdoll", "Siamois", "Birman", "Burmese"],
            colors: &["colourpoint", "roux", "crème", "golden"],
            rarity_bonus: 2,
        })
    } else if month == 11 && (20..=30).contains(&day) {
        Some(CatEventTheme {
            key: "solidarite",
            intro: "🤝 Semaine de solidarité: un chat du dehors cherche un foyer patient.",
            memory: "A rejoint sa maison pendant une semaine de solidarité.",
            names: &["Abri", "Ami", "Espoir", "Main", "Lien", "Partage"],
            breeds: &["Chat de gouttière", "European Shorthair", "Chartreux", "Manx"],
            colors: &["noir", "gris", "tigré", "bicolore noir et blanc"],
            rarity_bonus: 1,
        })
    } else {
        None
    }
}

fn cat_event_label(key: &str) -> &'static str {
    match key {
        "noel" => "Noël",
        "halloween" => "Halloween",
        "paques" => "Pâques",
        "fete_grand_meres" => "Fête des grands-mères",
        "fete_grand_peres" => "Fête des grands-pères",
        "fete_meres" => "Fête des mères",
        "fete_peres" => "Fête des pères",
        "familles" => "Journée des familles",
        "fete_travail" => "Fête du Travail",
        "droits_femmes" => "Journée des droits des femmes",
        "musique" => "Fête de la musique",
        "fiertes" => "Fiertés",
        "nouvel_an" => "Nouvel An",
        "saint_valentin" => "Saint-Valentin",
        "cancer" => "Journée contre le cancer",
        "eau" => "Journée mondiale de l'eau",
        "forets" => "Journée des forêts",
        "sante" => "Journée mondiale de la santé",
        "rentree" => "Rentrée",
        "paix" => "Journée de la paix",
        "coeur" => "Journée du cœur",
        "toussaint" => "Toussaint",
        "sante_mentale" => "Journée de la santé mentale",
        "alimentation" => "Journée de l'alimentation",
        "handicap" => "Journée du handicap",
        "sida" => "Journée de lutte contre le sida",
        "poisson_avril" => "Poisson d'avril",
        "jour_terre" => "Jour de la Terre",
        "biodiversite" => "Journée de la biodiversité",
        "idahot" => "Journée contre les LGBTphobies",
        "environnement" => "Journée de l'environnement",
        "oceans" => "Journée des océans",
        "don_sang" => "Journée du don du sang",
        "voisins" => "Fête des voisins",
        "fete_nationale" => "Fête nationale",
        "jour_chat" => "Journée internationale du chat",
        "humanitaire" => "Journée humanitaire",
        "prevention_suicide" => "Journée de prévention du suicide",
        "amitie" => "Journée de l'amitié",
        "solidarite" => "Semaine de solidarité",
        _ => "Événement spécial",
    }
}

fn easter_sunday(year: i32) -> NaiveDate {
    let a = year % 19;
    let b = year / 100;
    let c = year % 100;
    let d = b / 4;
    let e = b % 4;
    let f = (b + 8) / 25;
    let g = (b - f + 1) / 3;
    let h = (19 * a + b - d - g + 15) % 30;
    let i = c / 4;
    let k = c % 4;
    let l = (32 + 2 * e + 2 * i - h - k) % 7;
    let m = (a + 11 * h + 22 * l) / 451;
    let month = (h + l - 7 * m + 114) / 31;
    let day = ((h + l - 7 * m + 114) % 31) + 1;
    NaiveDate::from_ymd_opt(year, month as u32, day as u32).expect("date de Pâques valide")
}

fn french_mothers_day(year: i32) -> NaiveDate {
    let last_sunday_may = last_weekday_of_month(year, 5, Weekday::Sun);
    let pentecost = easter_sunday(year) + chrono::Duration::days(49);

    if last_sunday_may == pentecost {
        nth_weekday_of_month(year, 6, Weekday::Sun, 1)
    } else {
        last_sunday_may
    }
}

fn nth_weekday_of_month(year: i32, month: u32, weekday: Weekday, nth: u32) -> NaiveDate {
    let mut date = NaiveDate::from_ymd_opt(year, month, 1).expect("mois valide");
    let mut seen = 0;

    loop {
        if date.weekday() == weekday {
            seen += 1;
            if seen == nth {
                return date;
            }
        }

        date += chrono::Duration::days(1);
    }
}

fn last_weekday_of_month(year: i32, month: u32, weekday: Weekday) -> NaiveDate {
    let next_month = if month == 12 {
        NaiveDate::from_ymd_opt(year + 1, 1, 1).expect("mois valide")
    } else {
        NaiveDate::from_ymd_opt(year, month + 1, 1).expect("mois valide")
    };
    let mut date = next_month - chrono::Duration::days(1);

    while date.weekday() != weekday {
        date -= chrono::Duration::days(1);
    }

    date
}

fn generate_event_cat(theme: Option<CatEventTheme>) -> Cat {
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

async fn spawn_wild_cat_resolution(ctx: Context, channel_id: ChannelId, duration_secs: u64) {
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

async fn spawn_adoption_resolution(ctx: Context, channel_id: ChannelId, duration_secs: u64) {
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

        if give_refuge_cat_to_user(&pool, chosen_cat.id, user.id)
            .await
            .is_err()
        {
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
