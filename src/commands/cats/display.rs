//! Mise en forme des chats pour l'affichage dans Discord.

use serenity::model::channel::ReactionType;
use serenity::model::prelude::*;
use serenity::prelude::*;
use chrono::Datelike;

use crate::database::CollectedCat;
use crate::time::paris_today;


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

pub(super) fn format_age(age_months: i32) -> String {
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

pub(super) fn cat_display_name(cat: &CollectedCat) -> String {
    match &cat.nickname {
        Some(nickname) if !nickname.trim().is_empty() => {
            format!("{} \"{}\"", cat.name, nickname.trim())
        }
        _ => cat.name.clone(),
    }
}

pub(super) fn format_cat_identity(cat: &CollectedCat) -> String {
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

pub(super) fn format_cat_scene(cat: &CollectedCat) -> String {
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

pub(super) fn format_mycats_page(
    owner_name: &str,
    cats: &[CollectedCat],
    requested_page: usize,
) -> (String, usize, usize) {
    let per_page = 10usize;
    let total_pages = cats.len().div_ceil(per_page).max(1);
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

pub(super) async fn handle_mycats_reactions(
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

pub(super) async fn reaction_has_user(
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

pub(super) fn daily_house_scene(cat: &CollectedCat) -> &'static str {
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
