//! Génération aléatoire des chats : races, couleurs, noms et rareté.

use rand::{thread_rng, Rng};



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

        capped_score.clamp(1, 20)
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
