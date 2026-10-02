use serenity::framework::standard::{macros::command, CommandResult, Args};
use serenity::model::prelude::*;
use serenity::prelude::*;
use tracing::error;
use rand::{Rng, thread_rng};

use crate::utils::{Card, generate_card};
use crate::commands::profile::update_profile_score;

#[command]
#[description = "Jeu pour deviner un nombre entre 1 et 1000"]
pub async fn juste(ctx: &Context, msg: &Message) -> CommandResult {
    let answer = thread_rng().gen_range(1..1001);
    
    if let Err(why) = msg.channel_id.say(&ctx.http, "Choisis un nombre entre 1 et 1000.").await {
        error!("Erreur lors de l'envoi du message: {:?}", why);
        return Ok(());
    }
    
    let mut _tries = 0;
    
    loop {
        _tries += 1;
        
        let guess_msg = match get_message_response(ctx, msg, 30.0).await {
            Some(response) => response,
            None => {
                if let Err(why) = msg.channel_id.say(&ctx.http, &format!("Trop lent ! La réponse était {}.", answer)).await {
                    error!("Erreur lors de l'envoi du message: {:?}", why);
                }
                return Ok(());
            }
        };
        
        let guess: i32 = match guess_msg.content.parse() {
            Ok(num) => num,
            Err(_) => continue, // Ignorer les entrées non numériques
        };
        
        if guess > answer {
            if let Err(why) = msg.channel_id.say(&ctx.http, "-").await {
                error!("Erreur lors de l'envoi du message: {:?}", why);
                return Ok(());
            }
        } else if guess < answer {
            if let Err(why) = msg.channel_id.say(&ctx.http, "+").await {
                error!("Erreur lors de l'envoi du message: {:?}", why);
                return Ok(());
            }
        } else {
            if let Err(why) = msg.channel_id.say(&ctx.http, "Bien joué, tu as trouvé le nombre !").await {
                error!("Erreur lors de l'envoi du message: {:?}", why);
            }
            
            update_profile_score(ctx, msg, 3).await?;
            break;
        }
    }
    
    Ok(())
}

#[command]
#[description = "Tentez votre chance en devinant un nombre entre 1 et 10"]
pub async fn usd(ctx: &Context, msg: &Message) -> CommandResult {
    if let Err(why) = msg.channel_id.say(&ctx.http, "Choisis un nombre entre 1 et 10.").await {
        error!("Erreur lors de l'envoi du message: {:?}", why);
        return Ok(());
    }
    
    let answer = thread_rng().gen_range(1..11);
    
    let guess_msg = match get_message_response(ctx, msg, 15.0).await {
        Some(response) => response,
        None => {
            if let Err(why) = msg.channel_id.say(&ctx.http, &format!("Trop lent ! La réponse était {}.", answer)).await {
                error!("Erreur lors de l'envoi du message: {:?}", why);
            }
            return Ok(());
        }
    };
    
    let guess: i32 = match guess_msg.content.parse() {
        Ok(num) => num,
        Err(_) => {
            if let Err(why) = msg.channel_id.say(&ctx.http, "Ce n'est pas un nombre valide !").await {
                error!("Erreur lors de l'envoi du message: {:?}", why);
            }
            return Ok(());
        }
    };
    
    if guess == answer {
        if let Err(why) = msg.channel_id.say(&ctx.http, "Bravo, tu as gagné !").await {
            error!("Erreur lors de l'envoi du message: {:?}", why);
        }
        
        update_profile_score(ctx, msg, 10).await?;
    } else {
        if let Err(why) = msg.channel_id.say(&ctx.http, &format!("Dommage, c'était {}. Mieux vaut tenter ta chance la prochaine fois !", answer)).await {
            error!("Erreur lors de l'envoi du message: {:?}", why);
        }
    }
    
    Ok(())
}

#[command]
#[description = "Jouez au Blackjack contre le bot"]
pub async fn bj(ctx: &Context, msg: &Message, args: Args) -> CommandResult {
    // Vérifier si une mise a été fournie
    let bet: Option<i32> = if !args.is_empty() {
        match args.current().unwrap_or("0").parse::<i32>() {
            Ok(amount) if amount > 0 => Some(amount),
            _ => {
                if let Err(why) = msg.channel_id.say(&ctx.http, "Mise invalide. Utilisez un nombre entier positif.").await {
                    error!("Erreur lors de l'envoi du message: {:?}", why);
                }
                return Ok(());
            }
        }
    } else {
        None
    };
    
    // Message de début de jeu
    if let Some(bet_amount) = bet {
        if let Err(why) = msg.channel_id.say(&ctx.http, &format!("🎲 Vous avez misé **{}** points. Bonne chance!", bet_amount)).await {
            error!("Erreur lors de l'envoi du message: {:?}", why);
        }
    }
    
    // Initialisation du jeu
    let mut game_state = BlackjackGame::new();
    
    // Distribuer les cartes initiales
    game_state.dealer_cards.push(generate_card());
    game_state.dealer_cards.push(generate_card());
    game_state.player_hands.push(BlackjackHand {
        cards: vec![generate_card(), generate_card()],
        total: 0,
        aces: 0,
        doubled: false,
        is_blackjack: false,
    });
    
    // Calculer les totaux initiaux
    game_state.calculate_dealer_total();
    game_state.calculate_player_total(0);
    
    // Afficher la table de jeu initiale avec un message plus visuel
    let mut game_message = match msg.channel_id.send_message(&ctx.http, |m| {
        m.embed(|e| {
            e.title("🎮 Blackjack")
             .color((0, 128, 0)) // Couleur verte
             .field("🎭 Croupier", 
                    format!("Montre: **{} de {}**\nCarte cachée: **???**", 
                            game_state.dealer_cards[0].0.value, game_state.dealer_cards[0].0.suit), 
                    false)
             .field("👤 Votre main", 
                    format!("**{} de {}** + **{} de {}**\n**Total: {}**", 
                            game_state.player_hands[0].cards[0].0.value, game_state.player_hands[0].cards[0].0.suit,
                            game_state.player_hands[0].cards[1].0.value, game_state.player_hands[0].cards[1].0.suit,
                            game_state.player_hands[0].total),
                    false)
             .field("Actions", 
                    "🎯 Tirer une carte\n🛑 Rester\n2️⃣ Doubler", 
                    false)
             .footer(|f| f.text("Réagissez pour faire votre choix"))
        })
    }).await {
        Ok(message) => message,
        Err(why) => {
            error!("Erreur lors de l'envoi du message de jeu: {:?}", why);
            return Ok(());
        }
    };
    
    // Ajouter les réactions pour les actions du joueur dans un ordre fixe
    let mut reactions = vec![
        ReactionType::Unicode("🎯".to_string()),  // Tirer une carte
        ReactionType::Unicode("🛑".to_string()),  // Rester
        ReactionType::Unicode("2️⃣".to_string()), // Doubler
    ];
    
    // Ajouter une réaction pour split si les deux cartes ont la même valeur numérique
    let can_split = game_state.player_hands[0].cards[0].1 == game_state.player_hands[0].cards[1].1;
    if can_split {
        reactions.push(ReactionType::Unicode("🔀".to_string())); // Split
    }
    
    // Ajouter l'option d'assurance si la carte visible du croupier est un As
    let can_insure = game_state.dealer_cards[0].0.value == "As";
    if can_insure {
        reactions.push(ReactionType::Unicode("🛡️".to_string())); // Assurance
    }
    
    // Ajouter toutes les réactions de façon séquentielle pour garantir l'ordre
    for reaction in &reactions {
        if let Err(why) = game_message.react(&ctx.http, reaction.clone()).await {
            error!("Erreur lors de l'ajout d'une réaction: {:?}", why);
        }
    }
    
    // Jouer le jeu
    let result = play_blackjack(ctx, msg, &mut game_state, &mut game_message, reactions, bet).await?;
    
    // Traiter le résultat final et mettre à jour le score UNE SEULE FOIS
    if result > 0 {
        update_profile_score(ctx, msg, result).await?;
    }
    
    Ok(())
}

// Structure pour stocker l'état du jeu de Blackjack
struct BlackjackGame {
    dealer_cards: Vec<(Card, i32)>,
    player_hands: Vec<BlackjackHand>,
    dealer_total: i32,
    dealer_aces: i32,
    has_split: bool,
    dealer_has_blackjack: bool,
}

struct BlackjackHand {
    cards: Vec<(Card, i32)>,
    total: i32,
    aces: i32,
    doubled: bool,
    is_blackjack: bool,
}

impl BlackjackGame {
    fn new() -> Self {
        BlackjackGame {
            dealer_cards: Vec::new(),
            player_hands: Vec::new(),
            dealer_total: 0,
            dealer_aces: 0,
            has_split: false,
            dealer_has_blackjack: false,
        }
    }
    
    // Calculer le total du croupier
    fn calculate_dealer_total(&mut self) {
        self.dealer_total = self.dealer_cards.iter().map(|(_, value)| *value).sum();
        self.dealer_aces = self.dealer_cards.iter().filter(|(card, _)| card.value == "As").count() as i32;
        
        // Ajuster les As si nécessaire
        while self.dealer_total > 21 && self.dealer_aces > 0 {
            self.dealer_total -= 10;
            self.dealer_aces -= 1;
        }
        
        // Vérifier si le croupier a un blackjack naturel
        self.dealer_has_blackjack = self.dealer_cards.len() == 2 && self.dealer_total == 21;
    }
    
    // Calculer le total du joueur pour une main spécifique
    fn calculate_player_total(&mut self, hand_index: usize) {
        if hand_index >= self.player_hands.len() {
            return;
        }
        
        let hand = &mut self.player_hands[hand_index];
        hand.total = hand.cards.iter().map(|(_, value)| *value).sum();
        hand.aces = hand.cards.iter().filter(|(card, _)| card.value == "As").count() as i32;
        
        // Ajuster les As si nécessaire
        while hand.total > 21 && hand.aces > 0 {
            hand.total -= 10;
            hand.aces -= 1;
        }
        
        // Vérifier si la main est un blackjack naturel (2 cartes totalisant 21)
        hand.is_blackjack = hand.cards.len() == 2 && hand.total == 21;
    }
    
    // Ajouter une carte à une main spécifique
    fn add_card_to_hand(&mut self, hand_index: usize) -> (Card, i32) {
        let new_card = generate_card();
        let card_copy = new_card.clone();
        self.player_hands[hand_index].cards.push(new_card);
        self.calculate_player_total(hand_index);
        card_copy
    }
    
    // Ajouter une carte au croupier
    fn add_card_to_dealer(&mut self) -> (Card, i32) {
        let new_card = generate_card();
        let card_copy = new_card.clone();
        self.dealer_cards.push(new_card);
        self.calculate_dealer_total();
        card_copy
    }
}

// Jouer une partie de Blackjack
async fn play_blackjack(
    ctx: &Context, 
    msg: &Message,
    game: &mut BlackjackGame,
    game_message: &mut Message,
    reactions: Vec<ReactionType>,
    bet: Option<i32>
) -> Result<i32, Box<dyn std::error::Error + Send + Sync>> {
    let mut current_hand_index = 0;
    let mut player_stands = false;
    
    // Phase du joueur
    while !player_stands && current_hand_index < game.player_hands.len() {
        let hand = &game.player_hands[current_hand_index];
        if hand.total >= 21 {
            // Si le joueur a 21+, passer automatiquement
            if let Err(why) = msg.channel_id.say(&ctx.http, &format!(
                "Main {}: **{}** points! Passage automatique à la main suivante.", 
                current_hand_index + 1, hand.total
            )).await {
                error!("Erreur lors de l'envoi du message: {:?}", why);
            }
            
            if game.has_split && current_hand_index < game.player_hands.len() - 1 {
                current_hand_index += 1;
                update_game_message(ctx, msg, game, game_message, current_hand_index).await?;
                continue;
            } else {
                player_stands = true;
                continue;
            }
        }
        
        // Attendre la réaction du joueur avec un timeout plus court (30 secondes au lieu de 60)
        let reaction_result = wait_for_reaction_with_options(
            ctx, 
            game_message, 
            msg.author.id, 
            reactions.clone(), 
            30.0
        ).await;
        
        match reaction_result {
            Some((reaction, _)) => {
                if reaction == ReactionType::Unicode("🎯".to_string()) {
                    // Tirer une carte
                    let (new_card, _) = game.add_card_to_hand(current_hand_index);
                    let hand = &game.player_hands[current_hand_index];
                    
                    let hand_prefix = if game.has_split {
                        format!("Main {} - ", current_hand_index + 1)
                    } else {
                        "".to_string()
                    };
                    
                    if let Err(why) = msg.channel_id.say(&ctx.http, &format!(
                        "{}Nouvelle carte: **{} de {}** (Total: **{}**)",
                        hand_prefix, new_card.value, new_card.suit, hand.total
                    )).await {
                        error!("Erreur lors de l'envoi du message: {:?}", why);
                    }
                    
                    if hand.total > 21 {
                        if let Err(why) = msg.channel_id.say(&ctx.http, 
                            &format!("{}❌ **Dépassement!** Vous avez **{}** points, c'est trop.", 
                                    hand_prefix, hand.total)
                        ).await {
                            error!("Erreur lors de l'envoi du message: {:?}", why);
                        }
                        
                        if game.has_split && current_hand_index < game.player_hands.len() - 1 {
                            // Passer à la main suivante si split
                            current_hand_index += 1;
                            
                            if let Err(why) = msg.channel_id.say(&ctx.http, 
                                &format!("Passage à la main {}...", current_hand_index + 1)
                            ).await {
                                error!("Erreur lors de l'envoi du message: {:?}", why);
                            }
                            
                            update_game_message(ctx, msg, game, game_message, current_hand_index).await?;
                        } else {
                            player_stands = true;
                        }
                    } else if hand.total == 21 {
                        if let Err(why) = msg.channel_id.say(&ctx.http, 
                            &format!("{}🎉 **21!** Vous avez 21 points!", hand_prefix)
                        ).await {
                            error!("Erreur lors de l'envoi du message: {:?}", why);
                        }
                        
                        if game.has_split && current_hand_index < game.player_hands.len() - 1 {
                            // Passer à la main suivante si split
                            current_hand_index += 1;
                            
                            if let Err(why) = msg.channel_id.say(&ctx.http, 
                                &format!("Passage à la main {}...", current_hand_index + 1)
                            ).await {
                                error!("Erreur lors de l'envoi du message: {:?}", why);
                            }
                            
                            update_game_message(ctx, msg, game, game_message, current_hand_index).await?;
                        } else {
                            player_stands = true;
                        }
                    } else {
                        // Continuer le jeu
                        update_game_message(ctx, msg, game, game_message, current_hand_index).await?;
                    }
                }
                else if reaction == ReactionType::Unicode("🛑".to_string()) {
                    let hand_prefix = if game.has_split {
                        format!("Main {} - ", current_hand_index + 1)
                    } else {
                        "".to_string()
                    };
                    
                    if let Err(why) = msg.channel_id.say(&ctx.http, 
                        &format!("{}Vous restez à **{}** points.", 
                                hand_prefix, game.player_hands[current_hand_index].total)
                    ).await {
                        error!("Erreur lors de l'envoi du message: {:?}", why);
                    }
                    
                    if game.has_split && current_hand_index < game.player_hands.len() - 1 {
                        // Passer à la main suivante si split
                        current_hand_index += 1;
                        
                        if let Err(why) = msg.channel_id.say(&ctx.http, 
                            &format!("Passage à la main {}...", current_hand_index + 1)
                        ).await {
                            error!("Erreur lors de l'envoi du message: {:?}", why);
                        }
                        
                        update_game_message(ctx, msg, game, game_message, current_hand_index).await?;
                    } else {
                        player_stands = true;
                    }
                }
                else if reaction == ReactionType::Unicode("2️⃣".to_string()) {
                    // Double: doubler la mise, tirer une seule carte puis rester
                    game.player_hands[current_hand_index].doubled = true;
                    
                    let hand_prefix = if game.has_split {
                        format!("Main {} - ", current_hand_index + 1)
                    } else {
                        "".to_string()
                    };
                    
                    if let Err(why) = msg.channel_id.say(&ctx.http, 
                        &format!("{}Vous doublez votre mise!", hand_prefix)
                    ).await {
                        error!("Erreur lors de l'envoi du message: {:?}", why);
                    }
                    
                    // Tirer une carte et terminer
                    let (new_card, _) = game.add_card_to_hand(current_hand_index);
                    let hand = &game.player_hands[current_hand_index];
                    
                    if let Err(why) = msg.channel_id.say(&ctx.http, &format!(
                        "{}Carte doublée: **{} de {}** (Total: **{}**)",
                        hand_prefix, new_card.value, new_card.suit, hand.total
                    )).await {
                        error!("Erreur lors de l'envoi du message: {:?}", why);
                    }
                    
                    if game.has_split && current_hand_index < game.player_hands.len() - 1 {
                        // Passer à la main suivante si split
                        current_hand_index += 1;
                        
                        if let Err(why) = msg.channel_id.say(&ctx.http, 
                            &format!("Passage à la main {}...", current_hand_index + 1)
                        ).await {
                            error!("Erreur lors de l'envoi du message: {:?}", why);
                        }
                        
                        update_game_message(ctx, msg, game, game_message, current_hand_index).await?;
                    } else {
                        player_stands = true;
                    }
                }
                else if reaction == ReactionType::Unicode("🔀".to_string()) && 
                        !game.has_split && // Empêcher de split plusieurs fois
                        game.player_hands[0].cards[0].1 == game.player_hands[0].cards[1].1 {
                    // Split: séparer les deux cartes et jouer deux mains
                    handle_split(ctx, msg, game).await?;
                    
                    // Après le split, commencer par jouer la première main
                    current_hand_index = 0;
                    
                    if let Err(why) = msg.channel_id.say(&ctx.http, 
                        "Commençons par jouer la main 1..."
                    ).await {
                        error!("Erreur lors de l'envoi du message: {:?}", why);
                    }
                    
                    // Mise à jour de l'interface avec la nouvelle main active
                    update_game_message(ctx, msg, game, game_message, current_hand_index).await?;
                }
                else if reaction == ReactionType::Unicode("🛡️".to_string()) &&
                        game.dealer_cards[0].0.value == "As" {
                    // Assurance contre un Blackjack du croupier
                    if let Err(why) = msg.channel_id.say(&ctx.http, 
                        "Vous prenez une assurance contre un Blackjack du croupier."
                    ).await {
                        error!("Erreur lors de l'envoi du message: {:?}", why);
                    }
                    
                    // Vérifier si le croupier a un Blackjack
                    let dealer_has_blackjack = game.dealer_cards[1].1 == 10 || game.dealer_cards[1].0.value == "As";
                    
                    if dealer_has_blackjack {
                        if let Err(why) = msg.channel_id.say(&ctx.http, &format!(
                            "Le croupier révèle sa carte cachée: **{} de {}**\nLe croupier a un Blackjack! Votre assurance vous paie.", 
                            game.dealer_cards[1].0.value, game.dealer_cards[1].0.suit
                        )).await {
                            error!("Erreur lors de l'envoi du message: {:?}", why);
                        }
                        
                        // Retourner directement les points de l'assurance
                        return Ok(10);
                    } else {
                        if let Err(why) = msg.channel_id.say(&ctx.http, 
                            "Le croupier n'a pas de Blackjack. Vous perdez votre assurance. Le jeu continue."
                        ).await {
                            error!("Erreur lors de l'envoi du message: {:?}", why);
                        }
                    }
                }
            },
            None => {
                // Timeout - l'utilisateur n'a pas réagi à temps
                player_stands = true;
                if let Err(why) = msg.channel_id.say(&ctx.http, 
                    "⏰ Temps écoulé! Vous restez automatiquement avec vos cartes actuelles."
                ).await {
                    error!("Erreur lors de l'envoi du message de timeout: {:?}", why);
                }
            }
        }
    }
    
    // Phase de jeu du croupier - uniquement si le joueur n'a pas dépassé 21 dans toutes ses mains
    let all_hands_busted = game.player_hands.iter().all(|hand| hand.total > 21);
    if !all_hands_busted {
        // Phase du croupier en un seul message
        let dealer_play_result = play_dealer_phase(ctx, msg, game).await?;
        
        if let Err(why) = msg.channel_id.say(&ctx.http, dealer_play_result).await {
            error!("Erreur lors de l'envoi du résumé du croupier: {:?}", why);
        }
    }
    
    // Calculer le résultat final et les points à retourner
    let total_points = calculate_final_points(game, bet);
    
    // Afficher le résultat final
    show_final_result(ctx, msg, game, bet, total_points).await?;
    
    Ok(total_points)
}

// Mettre à jour le message du jeu
async fn update_game_message(
    ctx: &Context, 
    msg: &Message,
    game: &BlackjackGame,
    game_message: &mut Message,
    hand_index: usize
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let hand = &game.player_hands[hand_index];
    
    let new_message = msg.channel_id.send_message(&ctx.http, |m| {
        m.embed(|e| {
            let title = if game.has_split {
                format!("🎮 Blackjack - Main {} sur {}", hand_index + 1, game.player_hands.len())
            } else {
                "🎮 Blackjack - Tour suivant".to_string()
            };
            
            e.title(title)
             .color((0, 128, 0))
             .field("🎭 Croupier", 
                    format!("Montre: **{} de {}**\nCarte cachée: **???**", 
                            game.dealer_cards[0].0.value, game.dealer_cards[0].0.suit), 
                    false);
            
            if game.has_split {
                // Afficher toutes les mains avec la main actuelle en surbrillance
                for (i, h) in game.player_hands.iter().enumerate() {
                    // Construire un affichage détaillé pour chaque main
                    let cards_display = h.cards.iter()
                        .map(|(card, _)| format!("**{} de {}**", card.value, card.suit))
                        .collect::<Vec<String>>().join(" + ");
                    
                    let title = if i == hand_index {
                        format!("👤 Main {} - ACTIVE", i + 1)
                    } else if i < hand_index {
                        format!("👤 Main {} - Jouée", i + 1)
                    } else {
                        format!("👤 Main {} - En attente", i + 1)
                    };
                    
                    e.field(title, 
                            format!("{}\n**Total: {}**", cards_display, h.total),
                            false);
                }
            } else {
                // Construire un affichage détaillé pour la main actuelle
                let cards_display = hand.cards.iter()
                    .map(|(card, _)| format!("**{} de {}**", card.value, card.suit))
                    .collect::<Vec<String>>().join(" + ");
                
                e.field("👤 Votre main", 
                        format!("{}\n**Total: {}**", cards_display, hand.total),
                        false);
            }
            
            // Ajouter les options disponibles
            let mut actions_text = String::from("🎯 Tirer une carte\n🛑 Rester\n");
            
            // L'option de doubler n'est disponible que si le joueur a exactement 2 cartes
            if hand.cards.len() == 2 {
                actions_text.push_str("2️⃣ Doubler\n");
            }
            
            e.field("Actions", &actions_text, false)
             .footer(|f| {
                 if game.has_split {
                     f.text(format!("Jouez votre main {}. Cartes restantes: {}", hand_index + 1, hand.cards.len()))
                 } else {
                     f.text("Réagissez pour faire votre choix")
                 }
             })
        })
    }).await?;
    
    // Ajouter les réactions dans un ordre fixe
    let mut reactions = vec![
        ReactionType::Unicode("🎯".to_string()),  // Tirer une carte
        ReactionType::Unicode("🛑".to_string()),  // Rester
    ];
    
    // L'option de doubler n'est disponible que si le joueur a exactement 2 cartes
    if hand.cards.len() == 2 {
        reactions.push(ReactionType::Unicode("2️⃣".to_string())); // Doubler
    }
    
    // Ajouter toutes les réactions de façon séquentielle pour garantir l'ordre
    for reaction in &reactions {
        if let Err(why) = new_message.react(&ctx.http, reaction.clone()).await {
            error!("Erreur lors de l'ajout d'une réaction: {:?}", why);
        }
    }
    
    // Remplacer le message actuel par le nouveau
    *game_message = new_message;
    
    Ok(())
}

// Phase du croupier
async fn play_dealer_phase(
    _ctx: &Context, 
    _msg: &Message,
    game: &mut BlackjackGame
) -> Result<String, Box<dyn std::error::Error + Send + Sync>> {
    // Construire un message pour la phase du croupier
    let mut dealer_play_message = "**📝 Phase du croupier:**\n\n".to_string();
    
    // Révéler la deuxième carte du croupier
    dealer_play_message.push_str(&format!("➡️ Le croupier révèle sa carte cachée: **{} de {}**\n", 
                                        game.dealer_cards[1].0.value, game.dealer_cards[1].0.suit));
    dealer_play_message.push_str(&format!("➡️ Total du croupier: **{}**\n\n", game.dealer_total));
    
    // Le croupier tire des cartes tant qu'il a moins de 17 points
    while game.dealer_total < 17 {
        // Ajuster les As si nécessaire
        while game.dealer_total > 21 && game.dealer_aces > 0 {
            game.dealer_total -= 10;
            game.dealer_aces -= 1;
            dealer_play_message.push_str(&format!("➡️ Le croupier convertit un As (nouveau total: **{}**)\n", game.dealer_total));
        }
        
        if game.dealer_total >= 17 {
            break;
        }
        
        // Tirer une nouvelle carte
        let (new_card, _) = game.add_card_to_dealer();
        
        // Ajouter la nouvelle carte au message
        dealer_play_message.push_str(&format!("➡️ Le croupier tire: **{} de {}** (Total: **{}**)\n", 
                                            new_card.value, new_card.suit, game.dealer_total));
    }
    
    // Ajuster les scores finaux pour les As (vérification finale)
    while game.dealer_total > 21 && game.dealer_aces > 0 {
        game.dealer_total -= 10;
        game.dealer_aces -= 1;
        dealer_play_message.push_str(&format!("➡️ Le croupier convertit un As (nouveau total: **{}**)\n", game.dealer_total));
    }
    
    // Ajouter le résultat final du croupier
    if game.dealer_total > 21 {
        dealer_play_message.push_str(&format!("\n❌ **Le croupier a dépassé avec {}!**", game.dealer_total));
    } else {
        dealer_play_message.push_str(&format!("\n🎭 **Le croupier s'arrête à {}**", game.dealer_total));
    }
    
    Ok(dealer_play_message)
}

// Gérer le split
async fn handle_split(
    ctx: &Context, 
    msg: &Message,
    game: &mut BlackjackGame
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    game.has_split = true;
    
    if let Err(why) = msg.channel_id.say(&ctx.http, "🔀 **SPLIT!** Vous divisez votre main en deux!").await {
        error!("Erreur lors de l'envoi du message: {:?}", why);
    }
    
    // Stocker la main originale
    let original_hand = game.player_hands.remove(0);
    
    // Créer deux nouvelles mains
    let card1 = original_hand.cards[0].clone();
    let card2 = original_hand.cards[1].clone();
    
    // Première main avec la première carte
    let mut hand1 = BlackjackHand {
        cards: vec![card1],
        total: 0,
        aces: 0,
        doubled: false,
        is_blackjack: false,
    };
    
    // Deuxième main avec la deuxième carte
    let mut hand2 = BlackjackHand {
        cards: vec![card2],
        total: 0,
        aces: 0,
        doubled: false,
        is_blackjack: false,
    };
    
    // Ajouter une carte à chaque main
    let card1_new = generate_card();
    hand1.cards.push(card1_new);
    
    let card2_new = generate_card();
    hand2.cards.push(card2_new);
    
    // Calculer les totaux
    game.player_hands.push(hand1);
    game.player_hands.push(hand2);
    game.calculate_player_total(0);
    game.calculate_player_total(1);
    
    // Afficher les informations sur les deux mains de façon plus visuelle
    let split_embed = msg.channel_id.send_message(&ctx.http, |m| {
        m.embed(|e| {
            e.title("🔀 Split - Vos mains divisées")
             .color((148, 0, 211)) // Couleur violette pour le split
             .field("👤 Main 1", 
                    format!("**{} de {}** + **{} de {}**\n**Total: {}**", 
                            game.player_hands[0].cards[0].0.value, game.player_hands[0].cards[0].0.suit,
                            game.player_hands[0].cards[1].0.value, game.player_hands[0].cards[1].0.suit,
                            game.player_hands[0].total),
                    false)
             .field("👤 Main 2", 
                    format!("**{} de {}** + **{} de {}**\n**Total: {}**", 
                            game.player_hands[1].cards[0].0.value, game.player_hands[1].cards[0].0.suit,
                            game.player_hands[1].cards[1].0.value, game.player_hands[1].cards[1].0.suit,
                            game.player_hands[1].total),
                    false)
             .footer(|f| f.text("Vous allez maintenant jouer chaque main séparément."))
        })
    }).await;
    
    if let Err(why) = split_embed {
        error!("Erreur lors de l'envoi du message split: {:?}", why);
    }
    
    Ok(())
}

// Calculer les points finaux
fn calculate_final_points(game: &BlackjackGame, bet: Option<i32>) -> i32 {
    let mut total_points = 0;
    
    for hand in game.player_hands.iter() {
        if hand.total > 21 {
            continue; // Main perdue, pas de points
        }
        
        // Calculer les points en fonction du résultat
        let points = match bet {
            Some(amount) => {
                if hand.is_blackjack && !game.dealer_has_blackjack {
                    // Blackjack naturel: paiement 3:2 (1.5x la mise)
                    (amount as f32 * 1.5) as i32
                } else if game.dealer_has_blackjack && !hand.is_blackjack {
                    // Le croupier a un blackjack mais pas le joueur
                    0
                } else if hand.is_blackjack && game.dealer_has_blackjack {
                    // Égalité de blackjacks
                    amount
                } else if game.dealer_total > 21 || hand.total > game.dealer_total {
                    // Victoire normale
                    if hand.doubled {
                        amount * 2 // Mise doublée
                    } else {
                        amount
                    }
                } else if hand.total == game.dealer_total {
                    // Égalité
                    amount
                } else {
                    // Défaite
                    0
                }
            },
            None => {
                // Mode sans mise
                if hand.is_blackjack && !game.dealer_has_blackjack {
                    15 // Blackjack: 15 points
                } else if game.dealer_has_blackjack && !hand.is_blackjack {
                    0 // Perte contre blackjack du croupier
                } else if hand.is_blackjack && game.dealer_has_blackjack {
                    5 // Égalité de blackjacks
                } else if game.dealer_total > 21 || hand.total > game.dealer_total {
                    if hand.doubled {
                        20 // Victoire doublée
                    } else {
                        10 // Victoire normale
                    }
                } else if hand.total == game.dealer_total {
                    5 // Égalité
                } else {
                    0 // Défaite
                }
            }
        };
        
        // Ajouter les points à la somme totale
        total_points += points;
    }
    
    total_points
}

// Afficher le résultat final
async fn show_final_result(
    ctx: &Context, 
    msg: &Message,
    game: &BlackjackGame,
    bet: Option<i32>,
    total_points: i32
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    // Déterminer le résultat global
    let (title, color) = if total_points > 0 {
        ("🎉 Vous avez gagné!", (0, 255, 0))
    } else if game.player_hands.iter().all(|h| h.total > 21) {
        ("❌ Vous avez perdu!", (255, 0, 0))
    } else if game.dealer_total > 21 {
        ("🎉 Vous avez gagné! Le croupier a dépassé 21", (0, 255, 0))
    } else {
        ("❌ Vous avez perdu!", (255, 0, 0))
    };
    
    let result_message = msg.channel_id.send_message(&ctx.http, |m| {
        m.embed(|e| {
            e.title(title)
             .color(color)
             .field("🎭 Main du croupier", 
                    format!("**Total: {}**", game.dealer_total),
                    false);
            
            // Afficher les cartes du croupier en détail
            let dealer_cards = game.dealer_cards.iter().map(|(card, _)| 
                format!("**{} de {}**", card.value, card.suit)
            ).collect::<Vec<String>>().join(" + ");
            
            e.field("🎭 Cartes du croupier", dealer_cards, false);
            
            // Afficher les mains du joueur
            for (i, hand) in game.player_hands.iter().enumerate() {
                let hand_title = if game.has_split {
                    format!("👤 Main {}", i + 1)
                } else {
                    "👤 Votre main".to_string()
                };
                
                let hand_result = if hand.total > 21 {
                    "❌ Perdu (dépassement)"
                } else if hand.is_blackjack && !game.dealer_has_blackjack {
                    "🎯 BLACKJACK! (3:2)"
                } else if game.dealer_has_blackjack && !hand.is_blackjack {
                    "❌ Perdu (blackjack croupier)"
                } else if hand.is_blackjack && game.dealer_has_blackjack {
                    "🤝 Égalité (blackjack)"
                } else if game.dealer_total > 21 {
                    "🎉 Gagné (croupier dépassé)"
                } else if hand.total > game.dealer_total {
                    "🎉 Gagné"
                } else if hand.total < game.dealer_total {
                    "❌ Perdu"
                } else {
                    "🤝 Égalité"
                };
                
                let player_cards = hand.cards.iter().map(|(card, _)| 
                    format!("**{} de {}**", card.value, card.suit)
                ).collect::<Vec<String>>().join(" + ");
                
                e.field(hand_title, 
                        format!("{}\n**Total: {}**{}\n{}", 
                                player_cards,
                                hand.total,
                                if hand.doubled { " (Doublée)" } else { "" },
                                hand_result),
                        false);
            }
            
            // Afficher les points gagnés avec plus de détails
            if total_points > 0 {
                if let Some(bet_amount) = bet {
                    let ratio = total_points as f32 / bet_amount as f32;
                    let gain_info = if ratio > 1.4 && ratio < 1.6 {
                        format!("Vous avez misé {} et gagné {} points (BlackJack 3:2)!", bet_amount, total_points)
                    } else if ratio >= 2.0 {
                        format!("Vous avez misé {} et gagné {} points (mise doublée)!", bet_amount, total_points)
                    } else {
                        format!("Vous avez misé {} et gagné {} points!", bet_amount, total_points)
                    };
                    
                    e.footer(|f| f.text(gain_info));
                } else {
                    e.footer(|f| f.text(format!("Vous avez gagné {} points!", total_points)));
                }
            } else if bet.is_some() && game.player_hands.iter().any(|h| h.total == game.dealer_total) {
                e.footer(|f| f.text(format!("Égalité! Votre mise de {} points vous est remboursée.", bet.unwrap())));
            }
            
            e
        })
    }).await?;
    
    // Ajouter une réaction pour relancer une partie
    if let Err(why) = result_message.react(&ctx.http, ReactionType::Unicode("🔄".to_string())).await {
        error!("Erreur lors de l'ajout de la réaction pour relancer: {:?}", why);
        return Ok(());
    }
    
    // Message indiquant la possibilité de relancer
    if let Err(why) = msg.channel_id.say(&ctx.http, "Cliquez sur 🔄 pour relancer une partie avec les mêmes paramètres.").await {
        error!("Erreur lors de l'envoi du message de relance: {:?}", why);
    }
    
    // Attendre la réaction de l'utilisateur pour relancer (30 secondes)
    let restart_reaction = wait_for_specific_reaction(
        ctx,
        &result_message,
        msg.author.id,
        ReactionType::Unicode("🔄".to_string()),
        30.0
    ).await;
    
    // Si l'utilisateur a cliqué sur la réaction de relance
    if restart_reaction.is_some() {
        // Relancer une partie avec les mêmes paramètres
        if let Err(why) = msg.channel_id.say(&ctx.http, "Relance de la partie...").await {
            error!("Erreur lors de l'envoi du message de relance: {:?}", why);
        }
        
        // Créer une nouvelle commande args (vide ou avec la mise précédente)
        let mut args = Args::new("", &[]);
        if let Some(bet_amount) = bet {
            args = Args::new(&bet_amount.to_string(), &[]);
        }
        
        // Appeler la commande bj avec les mêmes paramètres
        bj(ctx, msg, args).await?;
    }
    
    Ok(())
}

// Version optimisée d'attente d'une réaction spécifique
async fn wait_for_specific_reaction(
    ctx: &Context,
    msg: &Message,
    user_id: UserId,
    emoji: ReactionType,
    timeout_seconds: f32
) -> Option<()> {
    let timeout_duration = std::time::Duration::from_secs_f32(timeout_seconds);
    let start_time = std::time::Instant::now();
    
    // Attendre la réaction spécifique avec une meilleure fréquence
    while start_time.elapsed() < timeout_duration {
        // Vérifier plus fréquemment
        tokio::time::sleep(std::time::Duration::from_millis(100)).await;
        
        if let Ok(users) = msg.reaction_users(&ctx.http, emoji.clone(), None, None).await {
            for user in users {
                if user.id == user_id {
                    return Some(());
                }
            }
        }
    }
    
    None
}

async fn get_message_response(ctx: &Context, msg: &Message, timeout_seconds: f32) -> Option<Message> {
    let author_id = msg.author.id;
    let channel_id = msg.channel_id;
    
    // Utiliser tokio timeout pour attendre un message
    tokio::time::timeout(
        std::time::Duration::from_secs_f32(timeout_seconds),
        wait_for_message_simple(ctx, channel_id, author_id)
    ).await.unwrap_or_default()
}

async fn wait_for_message_simple(ctx: &Context, channel_id: ChannelId, author_id: UserId) -> Option<Message> {
    // Version simplifiée sans collector
    let start_time = std::time::Instant::now();
    let timeout_duration = std::time::Duration::from_secs(300); // Failsafe timeout
    
    while start_time.elapsed() < timeout_duration {
        // Vérifier les messages toutes les secondes
        tokio::time::sleep(std::time::Duration::from_secs(1)).await;
        
        // Lire les derniers messages
        if let Ok(messages) = channel_id.messages(&ctx.http, |retriever| retriever.limit(10)).await {
            // Vérifier si un des messages récents est de l'utilisateur attendu
            for message in messages {
                if message.author.id == author_id && message.timestamp.timestamp() as u64 > 
                   (std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_secs() - 300) {
                    return Some(message);
                }
            }
        }
    }
    
    None
}

async fn wait_for_reaction_with_options(ctx: &Context, msg: &Message, user_id: UserId, emojis: Vec<ReactionType>, timeout_seconds: f32) -> Option<(ReactionType, UserId)> {
    let timeout_duration = std::time::Duration::from_secs_f32(timeout_seconds);
    let start_time = std::time::Instant::now();
    
    // Vérifier les réactions avec une meilleure fréquence
    while start_time.elapsed() < timeout_duration {
        // Optimisation: attendre moins entre les vérifications
        tokio::time::sleep(std::time::Duration::from_millis(100)).await;
        
        // Vérifier en priorité les emojis dans l'ordre pour un traitement cohérent
        for emoji in &emojis {
            if let Ok(users) = msg.reaction_users(&ctx.http, emoji.clone(), None, None).await {
                for user in users {
                    if user.id == user_id {
                        return Some((emoji.clone(), user_id));
                    }
                }
            }
        }
    }
    
    None
}

