//! Thèmes saisonniers des événements et calcul des dates de fêtes.

use chrono::{Datelike, NaiveDate, Weekday};

use crate::time::paris_today;

use super::*;

pub(super) fn cat_event_theme_for_date(date: NaiveDate) -> Option<CatEventTheme> {
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

pub(super) fn cat_event_theme_by_key(key: &str) -> Option<CatEventTheme> {
    let current_year = paris_today().year();

    for year in (current_year - 2)..=(current_year + 2) {
        let mut date = NaiveDate::from_ymd_opt(year, 1, 1)?;
        while date.year() == year {
            if let Some(theme) = cat_event_theme_for_date(date) {
                if theme.key == key {
                    return Some(theme);
                }
            }
            date += chrono::Duration::days(1);
        }
    }

    None
}

pub(super) fn cat_event_label(key: &str) -> &'static str {
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

pub(super) fn easter_sunday(year: i32) -> NaiveDate {
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

pub(super) fn french_mothers_day(year: i32) -> NaiveDate {
    let last_sunday_may = last_weekday_of_month(year, 5, Weekday::Sun);
    let pentecost = easter_sunday(year) + chrono::Duration::days(49);

    if last_sunday_may == pentecost {
        nth_weekday_of_month(year, 6, Weekday::Sun, 1)
    } else {
        last_sunday_may
    }
}

pub(super) fn nth_weekday_of_month(year: i32, month: u32, weekday: Weekday, nth: u32) -> NaiveDate {
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

pub(super) fn last_weekday_of_month(year: i32, month: u32, weekday: Weekday) -> NaiveDate {
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

#[cfg(test)]
mod tests {
    use super::*;

    fn date(year: i32, month: u32, day: u32) -> NaiveDate {
        NaiveDate::from_ymd_opt(year, month, day).unwrap()
    }

    #[test]
    fn paques_tombe_aux_dates_connues() {
        assert_eq!(easter_sunday(2024), date(2024, 3, 31));
        assert_eq!(easter_sunday(2025), date(2025, 4, 20));
        assert_eq!(easter_sunday(2026), date(2026, 4, 5));
    }

    #[test]
    fn fete_des_meres_est_le_dernier_dimanche_de_mai() {
        assert_eq!(french_mothers_day(2025), date(2025, 5, 25));
        assert_eq!(french_mothers_day(2026), date(2026, 5, 31));
    }

    #[test]
    fn fete_des_meres_est_decalee_quand_elle_tombe_a_la_pentecote() {
        // En 2004, la Pentecôte tombait le 30 mai.
        assert_eq!(french_mothers_day(2004), date(2004, 6, 6));
    }

    #[test]
    fn nieme_et_dernier_jour_de_la_semaine_du_mois() {
        assert_eq!(nth_weekday_of_month(2026, 6, Weekday::Sun, 3), date(2026, 6, 21));
        assert_eq!(last_weekday_of_month(2026, 12, Weekday::Thu), date(2026, 12, 31));
    }

    #[test]
    fn theme_de_noel_actif_autour_du_25_decembre() {
        assert_eq!(cat_event_theme_for_date(date(2026, 12, 25)).map(|t| t.key), Some("noel"));
    }
}
