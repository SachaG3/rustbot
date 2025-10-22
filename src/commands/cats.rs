use serenity::framework::standard::{macros::command, CommandResult};
use serenity::model::prelude::*;
use serenity::prelude::*;
use rand::{thread_rng, Rng};

use crate::database::{DatabasePool, get_user_by_discord_id, new_user, add_daily_cat, get_daily_cat_count, has_daily_cat_today, debug_daily_cats, add_collected_cat, get_user_cats, get_cat_by_id, transfer_cat, get_user_cat_count};

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
        let vowel_bonus = if "AEIOUaeiou".contains(self.name.chars().next().unwrap_or('x')) { 1 } else { 0 };
        
        // Score de base très bas pour forcer la rareté
        let base_score = 1;
        
        // Calcul avec plafonnement plus strict
        let raw_score = base_score + breed_bonus + color_bonus + age_bonus + name_bonus + vowel_bonus;
        
        // Système de plafonnement plus modéré
        let capped_score = match raw_score {
            0..=10 => raw_score,     // Scores bas : pas de changement
            11..=14 => 10 + (raw_score - 10) / 2,  // Scores moyens : légère réduction
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
                format!("{} an{} et {} mois", years, if years > 1 { "s" } else { "" }, months)
            }
        };
        
        format!("{} {} {} de {}", 
                self.name,
                self.breed.name, 
                self.color.name, 
                age_display)
    }
}

fn get_cat_breeds() -> Vec<CatBreed> {
    vec![
        CatBreed { name: "Chat de gouttière", rarity_bonus: 0 },
        CatBreed { name: "European Shorthair", rarity_bonus: 1 },
        CatBreed { name: "British Shorthair", rarity_bonus: 2 },
        CatBreed { name: "Chartreux", rarity_bonus: 2 },
        CatBreed { name: "Siamois", rarity_bonus: 3 },
        CatBreed { name: "Ragdoll", rarity_bonus: 4 },
        CatBreed { name: "Birman", rarity_bonus: 4 },
        CatBreed { name: "Persan", rarity_bonus: 4 },
        CatBreed { name: "Norvégien", rarity_bonus: 5 },
        CatBreed { name: "Maine Coon", rarity_bonus: 10 },
        CatBreed { name: "Scottish Fold", rarity_bonus: 5 },
        CatBreed { name: "Abyssin", rarity_bonus: 3 },
        CatBreed { name: "Bengal", rarity_bonus: 6 },
        CatBreed { name: "Sibérien", rarity_bonus: 10 },
        CatBreed { name: "Sphinx", rarity_bonus: 5 },
        CatBreed { name: "Oriental", rarity_bonus: 3 },
        CatBreed { name: "Savannah", rarity_bonus: 9 },
        CatBreed { name: "Toyger", rarity_bonus: 7 },
        CatBreed { name: "Cornish Rex", rarity_bonus: 2 },
        CatBreed { name: "Devon Rex", rarity_bonus: 2 },
        CatBreed { name: "Balinais", rarity_bonus: 4 },
        CatBreed { name: "Exotic Shorthair", rarity_bonus: 3 },
        CatBreed { name: "Turc de Van", rarity_bonus: 4 },
        CatBreed { name: "Angora Turc", rarity_bonus: 4 },
        CatBreed { name: "Singapura", rarity_bonus: 6 },
        CatBreed { name: "Korat", rarity_bonus: 4 },
        CatBreed { name: "Manx", rarity_bonus: 2 },
        CatBreed { name: "Burmese", rarity_bonus: 3 },
        CatBreed { name: "American Curl", rarity_bonus: 4 },
        CatBreed { name: "Peterbald", rarity_bonus: 5 },
        CatBreed { name: "Lykoi", rarity_bonus: 1 },
    ]
}

fn get_cat_colors() -> Vec<CatColor> {
    vec![
        CatColor { name: "noir", rarity_bonus: 0 },
        CatColor { name: "blanc", rarity_bonus: 1 },
        CatColor { name: "gris", rarity_bonus: 0 },
        CatColor { name: "bleu", rarity_bonus: 1 },
        CatColor { name: "roux", rarity_bonus: 2 },
        CatColor { name: "crème", rarity_bonus: 1 },
        CatColor { name: "chocolat", rarity_bonus: 2 },
        CatColor { name: "lilas", rarity_bonus: 3 },
        CatColor { name: "cannelle", rarity_bonus: 2 },
        CatColor { name: "fauve", rarity_bonus: 3 },
        CatColor { name: "tigré", rarity_bonus: 1 },
        CatColor { name: "marbré", rarity_bonus: 2 },
        CatColor { name: "moucheté", rarity_bonus: 2 },
        CatColor { name: "ticked", rarity_bonus: 3 },
        CatColor { name: "écaille de tortue", rarity_bonus: 3 },
        CatColor { name: "calico", rarity_bonus: 4 },
        CatColor { name: "dilute calico", rarity_bonus: 4 },
        CatColor { name: "torbie", rarity_bonus: 4 },
        CatColor { name: "bicolore noir et blanc", rarity_bonus: 1 },
        CatColor { name: "bicolore roux et blanc", rarity_bonus: 2 },
        CatColor { name: "bicolore bleu et blanc", rarity_bonus: 2 },
        CatColor { name: "bicolore crème et blanc", rarity_bonus: 2 },
        CatColor { name: "smoke", rarity_bonus: 3 },
        CatColor { name: "silver", rarity_bonus: 3 },
        CatColor { name: "golden", rarity_bonus: 4 },
        CatColor { name: "colourpoint", rarity_bonus: 4 },
        CatColor { name: "seal point", rarity_bonus: 4 },
        CatColor { name: "blue point", rarity_bonus: 4 },
        CatColor { name: "chocolate point", rarity_bonus: 5 },
        CatColor { name: "lilac point", rarity_bonus: 5 },
        CatColor { name: "red point", rarity_bonus: 4 },
        CatColor { name: "cream point", rarity_bonus: 4 },
        CatColor { name: "sepia", rarity_bonus: 3 },
        CatColor { name: "mink", rarity_bonus: 3 },
    ]
}

fn get_cat_names() -> Vec<&'static str> {
    vec![
        // Noms courts (bonus 0)
        "Max", "Leo", "Mia", "Sox", "Rex", "Zoe", "Ace", "Rio",
        
        // Noms moyens (bonus 1)
        "Luna", "Felix", "Bella", "Oscar", "Milo", "Chloe", "Zeus", "Nala", "Tiger", "Smoky", "Pearl", "Storm",
        
        // Noms longs (bonus 2)
        "Whiskers", "Shadow", "Princess", "Midnight", "Snowball", "Pumpkin", "Biscuit", "Caramel", "Thunder", "Duchess","Reyna","Aslan",
        
        // Noms très longs (bonus 3)
        "Buttercup", "Cinnamon", "Marshmallow", "Thunderbolt", "Strawberry", "Firecracker", "Blueberry", "Chocolate",
        
        // Noms avec voyelles (bonus +1)
        "Oliver", "Emma", "Oreo", "Angel", "Echo", "Iris", "Amber", "Opal", "Uma", "Ivy", "Aria", "Aspen", "Oakley", "Ember"
    ]
}

// Sélection pondérée pour les races (plus rarity_bonus est élevé, plus c'est rare)
fn select_weighted_breed(breeds: &[CatBreed], rng: &mut impl Rng) -> CatBreed {
    // Calculer les poids inversés : plus rarity_bonus est élevé, plus le poids est faible
    let weights: Vec<f32> = breeds.iter()
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
fn select_weighted_color(colors: &[CatColor], rng: &mut impl Rng) -> CatColor {
    // Calculer les poids inversés avec pénalité exponentielle pour les couleurs rares
    let weights: Vec<f32> = colors.iter()
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
    let pool = data.get::<DatabasePool>().expect("Impossible d'obtenir le pool");

    let user = match get_user_by_discord_id(&pool, msg.author.id.0).await {
        Ok(Some(u)) => u,
        Ok(None) => {
            let user_id = new_user(&pool, msg.author.id.0, &msg.author.name).await?;
            crate::database::User { id: user_id, id_utilisateur: msg.author.id.0.to_string(), pseudo: msg.author.name.clone(), score: 0 }
        },
        Err(_) => {
            msg.channel_id.say(&ctx.http, "Erreur avec la base de données").await.ok();
            return Ok(());
        }
    };

    if has_daily_cat_today(&pool, user.id).await.unwrap_or(false) {
        let debug_info = match debug_daily_cats(&pool, user.id).await {
            Ok(dates) => format!("\nDerniers chats obtenus :\n{}", dates.join("\n")),
            Err(_) => "".to_string()
        };
        msg.channel_id.say(&ctx.http, format!("Tu as déjà obtenu ton chat quotidien, reviens demain{}", debug_info)).await.ok();
        return Ok(());
    }

    // Donner le daily cat comme avant
    if add_daily_cat(&pool, user.id).await.is_err() {
        msg.channel_id.say(&ctx.http, "Impossible d'ajouter le Cat").await.ok();
        return Ok(());
    }

    let total = match get_daily_cat_count(&pool, user.id).await {
        Ok(c) => c,
        Err(_) => {
            msg.channel_id.say(&ctx.http, "Erreur avec la base de données").await.ok();
            return Ok(());
        }
    };

    // Message principal comme avant
    let mut response = format!("Tu as gagné un 🐱 ! Total: {}", total);

    // 70% de chance d'obtenir un chat secret EN PLUS (événement spécial première journée)
    let secret_cat_chance = {
        let mut rng = thread_rng();
        rng.gen_range(0..100) < 70
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
            secret_cat.rarity_score
        ).await {
            Ok(cat_id) => {
                let rarity_emoji = get_rarity_emoji(secret_cat.rarity_score);
                response += &format!(
                    "\n\n🏠 **Un chat errant a rejoint ton foyer !** 🏠\n{} **{}** (#{}) s'est installé chez toi !\n\n💫 Utilise `^^mycats` pour voir qui vit dans ta maison !",
                    rarity_emoji,
                    secret_cat.format_description(),
                    cat_id
                );
            },
            Err(_) => {
                // Pas grave si ça rate, on garde juste le daily cat
            }
        }
    }

    msg.channel_id.say(&ctx.http, response).await.ok();

    Ok(())
}

fn get_rarity_badge(score: i32) -> &'static str {
    match score {
        0..=5 => "",                    // Commun - pas de badge
        6..=8 => "✨ Spécial",         // Peu commun
        9..=11 => "🌟 Remarquable",    // Rare
        12..=14 => "💎 Exceptionnel",  // Épique
        15..=17 => "👑 Extraordinaire", // Légendaire
        18..=20 => "🔥 Mythique",      // Ultra-légendaire
        _ => "",
    }
}

fn get_rarity_emoji(score: i32) -> &'static str {
    match score {
        0..=5 => "🐾",      // Commun
        6..=10 => "✨",     // Peu commun
        11..=15 => "🌟",   // Rare
        16..=18 => "💎",   // Épique
        19..=20 => "👑",   // Légendaire
        _ => "🐾",
    }
}


#[command]
#[description = "Affiche ta collection de chats"]
pub async fn mycats(ctx: &Context, msg: &Message) -> CommandResult {
    let data = ctx.data.read().await;
    let pool = data.get::<DatabasePool>().expect("Impossible d'obtenir le pool");

    let user = match get_user_by_discord_id(&pool, msg.author.id.0).await {
        Ok(Some(u)) => u,
        Ok(None) => {
            msg.channel_id.say(&ctx.http, "Tu n'as pas encore de profil. Utilise ^^np pour en créer un").await.ok();
            return Ok(());
        },
        Err(_) => {
            msg.channel_id.say(&ctx.http, "Erreur avec la base de données").await.ok();
            return Ok(());
        }
    };

    match get_user_cats(&pool, user.id).await {
        Ok(cats) => {
            if cats.is_empty() {
                msg.channel_id.say(&ctx.http, "Aucun chat ne vit encore chez toi ! Utilise `^^cat` pour peut-être en adopter un.").await.ok();
            } else {
                let mut response = format!("🏠 **Les chats qui vivent chez {}** 🏠\n\n", msg.author.name);
                
                for (index, cat) in cats.iter().take(10).enumerate() {
                    let rarity_emoji = get_rarity_emoji(cat.rarity_score);
                    
                    let age_display = if cat.age_months <= 12 {
                        format!("{} mois", cat.age_months)
                    } else {
                        let years = cat.age_months / 12;
                        let months = cat.age_months % 12;
                        if months == 0 {
                            format!("{} an{}", years, if years > 1 { "s" } else { "" })
                        } else {
                            format!("{} an{} et {} mois", years, if years > 1 { "s" } else { "" }, months)
                        }
                    };
                    
                    let rarity_badge = get_rarity_badge(cat.rarity_score);
                    let badge_display = if !rarity_badge.is_empty() {
                        format!(" • {}", rarity_badge)
                    } else {
                        String::new()
                    };
                    
                    response += &format!(
                        "{}. {} **{} {} {} de {}**{} (#{}) \n", 
                        index + 1,
                        rarity_emoji,
                        cat.name,
                        cat.breed, 
                        cat.color, 
                        age_display,
                        badge_display,
                        cat.id
                    );
                }
                
                if cats.len() > 10 {
                    response += &format!("\n... et {} autres résidents !", cats.len() - 10);
                }
                
                response += &format!("\n\n🏠 **Total: {} chats vivent chez toi**", cats.len());
                
                msg.channel_id.say(&ctx.http, response).await.ok();
            }
        },
        Err(_) => {
            msg.channel_id.say(&ctx.http, "Erreur lors de la récupération de ta collection").await.ok();
        }
    }

    Ok(())
}

#[command]
#[description = "Affiche le nombre de Daily Cats"]
pub async fn cats(ctx: &Context, msg: &Message) -> CommandResult {
    let data = ctx.data.read().await;
    let pool = data.get::<DatabasePool>().expect("Impossible d'obtenir le pool");

    match get_user_by_discord_id(&pool, msg.author.id.0).await {
        Ok(Some(user)) => {
            match get_daily_cat_count(&pool, user.id).await {
                Ok(total) => {
                    let mut response = format!("Tu as {} 🐱.", total);
                    
                    // Ajouter info sur les chats secrets
                    match get_user_cat_count(&pool, user.id).await {
                        Ok(secret_count) => {
                            if secret_count > 0 {
                                response += &format!("\n🏠 Et {} chats vivent chez toi ! (Use `^^mycats`)", secret_count);
                            }
                        },
                        Err(_) => {}
                    }
                    
                    msg.channel_id.say(&ctx.http, response).await.ok();
                },
                Err(_) => {
                    msg.channel_id.say(&ctx.http, "Erreur avec la base de données").await.ok();
                }
            }
        },
        Ok(None) => {
            msg.channel_id.say(&ctx.http, "Tu n'as pas encore de profil. Utilise ^^np pour en créer un").await.ok();
        },
        Err(_) => {
            msg.channel_id.say(&ctx.http, "Erreur avec la base de données").await.ok();
        }
    }

    Ok(())
}

#[command]
#[description = "Donne un de tes chats à un autre utilisateur : ^^trade @user chat_id"]
pub async fn trade(ctx: &Context, msg: &Message) -> CommandResult {
    let data = ctx.data.read().await;
    let pool = data.get::<DatabasePool>().expect("Impossible d'obtenir le pool");

    let args: Vec<&str> = msg.content.split_whitespace().collect();
    if args.len() != 3 {
        msg.channel_id.say(&ctx.http, "Usage: `^^trade @utilisateur chat_id`\nExemple: `^^trade @John 15`\nCela donnera ton chat à cet utilisateur.").await.ok();
        return Ok(());
    }

    let user = match get_user_by_discord_id(&pool, msg.author.id.0).await {
        Ok(Some(u)) => u,
        Ok(None) => {
            msg.channel_id.say(&ctx.http, "Tu n'as pas encore de profil. Utilise ^^np pour en créer un").await.ok();
            return Ok(());
        },
        Err(_) => {
            msg.channel_id.say(&ctx.http, "Erreur avec la base de données").await.ok();
            return Ok(());
        }
    };

    // Parse cat_id
    let cat_id = match args[2].parse::<i32>() {
        Ok(id) => id,
        Err(_) => {
            msg.channel_id.say(&ctx.http, "ID de chat invalide !").await.ok();
            return Ok(());
        }
    };

    // Vérifier que le chat appartient à l'utilisateur
    match get_cat_by_id(&pool, cat_id).await {
        Ok(Some(cat)) => {
            if cat.user_id != user.id as i32 {
                msg.channel_id.say(&ctx.http, "Ce chat ne t'appartient pas !").await.ok();
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
                    msg.channel_id.say(&ctx.http, "Cet utilisateur n'a pas de profil !").await.ok();
                    return Ok(());
                },
                Err(_) => {
                    msg.channel_id.say(&ctx.http, "Erreur avec la base de données").await.ok();
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
                            format!("{} an{} et {} mois", years, if years > 1 { "s" } else { "" }, months)
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
                },
                Err(_) => {
                    msg.channel_id.say(&ctx.http, "Erreur lors du transfert !").await.ok();
                }
            }
        },
        Ok(None) => {
            msg.channel_id.say(&ctx.http, "Chat introuvable !").await.ok();
        },
        Err(_) => {
            msg.channel_id.say(&ctx.http, "Erreur avec la base de données").await.ok();
        }
    }

    Ok(())
} 