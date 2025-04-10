use image::{ImageBuffer, Rgba, RgbaImage};
use image::imageops::{overlay, FilterType};
use image::ImageFormat;
use reqwest::Client;
use std::io::Cursor;
use tracing::error;

pub async fn generate_score_image(avatar_url: &str, score: i32) -> Option<Vec<u8>> {
    // Télécharger l'image de profil
    match download_image(avatar_url).await {
        Some(avatar_data) => {
            // Créer une image à partir des données
            let avatar = match image::load_from_memory(&avatar_data) {
                Ok(img) => img.resize(120, 120, FilterType::Lanczos3),
                Err(e) => {
                    error!("Erreur lors du chargement de l'image de profil: {:?}", e);
                    return None;
                }
            };
            
            // Créer l'image de fond avec un gradient moderne
            let mut background = RgbaImage::new(500, 200);
            let width = background.width() as f32;
            let height = background.height() as f32;
            
            // Créer un gradient pour un look plus moderne (dégradé de bleu)
            for (x, _y, pixel) in background.enumerate_pixels_mut() {
                let x_ratio = x as f32 / width;
                let r = 45;
                let g = 49 + (x_ratio * 20.0) as u8;
                let b = 66 + (x_ratio * 40.0) as u8;
                *pixel = Rgba([r, g, b, 255]);
            }
            
            // Ajouter des éléments décoratifs (effet de lumière dans le coin)
            for (x, y, pixel) in background.enumerate_pixels_mut() {
                let distance_from_corner = ((x as f32 - 400.0).powi(2) + (y as f32 - 40.0).powi(2)).sqrt();
                if distance_from_corner < 80.0 {
                    let intensity = 1.0 - (distance_from_corner / 80.0);
                    let current = pixel.0;
                    *pixel = Rgba([
                        (current[0] as f32 + 20.0 * intensity) as u8,
                        (current[1] as f32 + 20.0 * intensity) as u8,
                        (current[2] as f32 + 20.0 * intensity) as u8,
                        255
                    ]);
                }
            }
            
            // Créer une version circulaire de l'avatar
            let circular_avatar = create_circular_avatar(avatar);
            
            // Placer l'avatar sur l'image de fond avec un effet d'ombre
            // Créer un cercle d'ombre légèrement plus grand
            let shadow_size = 124;
            let mut shadow = ImageBuffer::new(shadow_size, shadow_size);
            let shadow_center = shadow_size as f32 / 2.0;
            let shadow_radius = shadow_size as f32 / 2.0;
            
            for (x, y, pixel) in shadow.enumerate_pixels_mut() {
                let dx = x as f32 - shadow_center;
                let dy = y as f32 - shadow_center;
                let distance = (dx * dx + dy * dy).sqrt();
                
                if distance <= shadow_radius {
                    // Effet d'ombre qui s'estompe vers les bords
                    let alpha = if distance > shadow_radius - 4.0 {
                        let fade = 1.0 - (distance - (shadow_radius - 4.0)) / 4.0;
                        (fade * 80.0) as u8
                    } else {
                        80 // Alpha de l'ombre
                    };
                    *pixel = Rgba([0, 0, 0, alpha]);
                }
            }
            
            // Placer l'ombre et l'avatar
            overlay(&mut background, &shadow, 38, 38);
            overlay(&mut background, &circular_avatar, 40, 40);
            
            // Dessiner un contour brillant autour de l'avatar
            let center_x = 40 + 60; // x position + radius
            let center_y = 40 + 60; // y position + radius
            let radius = 61; // Légèrement plus grand que l'avatar
            
            for angle in 0..360 {
                let rad = angle as f32 * std::f32::consts::PI / 180.0;
                let x = center_x as f32 + radius as f32 * rad.cos();
                let y = center_y as f32 + radius as f32 * rad.sin();
                
                if x >= 0.0 && x < width && y >= 0.0 && y < height {
                    let brightness = 180 + (angle % 60) as u8;
                    background.put_pixel(x as u32, y as u32, Rgba([brightness, brightness, brightness, 255]));
                }
            }
            
            // Version simplifiée de l'affichage du score
            let text_x = 180u32;
            let text_y = 80u32;
            
            // Créer un rectangle arrondi pour le fond du score
            let rect_width = 250u32;
            let rect_height = 60u32;
            
            // Dessiner un rectangle pour le fond du score (version simplifiée)
            for x in 0..rect_width {
                for y in 0..rect_height {
                    if x + text_x < width as u32 && y + text_y < height as u32 {
                        // Dégradé horizontal pour le fond du score
                        let x_ratio = x as f32 / rect_width as f32;
                        let r = 60 + (x_ratio * 40.0) as u8;
                        let g = 70 + (x_ratio * 40.0) as u8;
                        let b = 110 + (x_ratio * 40.0) as u8;
                        background.put_pixel(x + text_x, y + text_y, Rgba([r, g, b, 220]));
                    }
                }
            }
            
            // Afficher le score dans une barre centrale
            let score_text = format!("{}", score);
            let score_bar_width = 150u32;
            let score_bar_height = 40u32;
            let score_bar_x = text_x + (rect_width - score_bar_width) / 2;
            let score_bar_y = text_y + (rect_height - score_bar_height) / 2;
            
            // Dessiner la barre de score (fond sombre semi-transparent avec un léger dégradé)
            for x in 0..score_bar_width {
                for y in 0..score_bar_height {
                    if score_bar_x + x < width as u32 && score_bar_y + y < height as u32 {
                        // Dégradé horizontal pour la barre
                        let x_ratio = x as f32 / score_bar_width as f32;
                        let r = 40 + (x_ratio * 10.0) as u8;
                        let g = 45 + (x_ratio * 10.0) as u8;
                        let b = 60 + (x_ratio * 10.0) as u8;
                        background.put_pixel(score_bar_x + x, score_bar_y + y, Rgba([r, g, b, 230]));
                    }
                }
            }
            
            // Dessiner un bord subtil autour de la barre
            for x in 0..score_bar_width {
                // Bord supérieur
                if score_bar_x + x < width as u32 && score_bar_y < height as u32 {
                    background.put_pixel(score_bar_x + x, score_bar_y, Rgba([100, 120, 180, 255]));
                }
                // Bord inférieur
                if score_bar_x + x < width as u32 && score_bar_y + score_bar_height - 1 < height as u32 {
                    background.put_pixel(score_bar_x + x, score_bar_y + score_bar_height - 1, Rgba([100, 120, 180, 255]));
                }
            }
            
            for y in 0..score_bar_height {
                // Bord gauche
                if score_bar_x < width as u32 && score_bar_y + y < height as u32 {
                    background.put_pixel(score_bar_x, score_bar_y + y, Rgba([100, 120, 180, 255]));
                }
                // Bord droit
                if score_bar_x + score_bar_width - 1 < width as u32 && score_bar_y + y < height as u32 {
                    background.put_pixel(score_bar_x + score_bar_width - 1, score_bar_y + y, Rgba([100, 120, 180, 255]));
                }
            }
            
            // Dessiner les chiffres du score
            let digit_patterns = [
                // 0
                [
                    0,1,1,1,0,
                    1,0,0,0,1,
                    1,0,0,0,1,
                    1,0,0,0,1,
                    1,0,0,0,1,
                    1,0,0,0,1,
                    0,1,1,1,0,
                ],
                // 1
                [
                    0,0,1,0,0,
                    0,1,1,0,0,
                    0,0,1,0,0,
                    0,0,1,0,0,
                    0,0,1,0,0,
                    0,0,1,0,0,
                    0,1,1,1,0,
                ],
                // 2
                [
                    0,1,1,1,0,
                    1,0,0,0,1,
                    0,0,0,0,1,
                    0,0,1,1,0,
                    0,1,0,0,0,
                    1,0,0,0,0,
                    1,1,1,1,1,
                ],
                // 3
                [
                    0,1,1,1,0,
                    1,0,0,0,1,
                    0,0,0,0,1,
                    0,0,1,1,0,
                    0,0,0,0,1,
                    1,0,0,0,1,
                    0,1,1,1,0,
                ],
                // 4
                [
                    0,0,0,1,0,
                    0,0,1,1,0,
                    0,1,0,1,0,
                    1,0,0,1,0,
                    1,1,1,1,1,
                    0,0,0,1,0,
                    0,0,0,1,0,
                ],
                // 5
                [
                    1,1,1,1,1,
                    1,0,0,0,0,
                    1,0,0,0,0,
                    1,1,1,1,0,
                    0,0,0,0,1,
                    1,0,0,0,1,
                    0,1,1,1,0,
                ],
                // 6
                [
                    0,1,1,1,0,
                    1,0,0,0,0,
                    1,0,0,0,0,
                    1,1,1,1,0,
                    1,0,0,0,1,
                    1,0,0,0,1,
                    0,1,1,1,0,
                ],
                // 7
                [
                    1,1,1,1,1,
                    0,0,0,0,1,
                    0,0,0,1,0,
                    0,0,1,0,0,
                    0,1,0,0,0,
                    0,1,0,0,0,
                    0,1,0,0,0,
                ],
                // 8
                [
                    0,1,1,1,0,
                    1,0,0,0,1,
                    1,0,0,0,1,
                    0,1,1,1,0,
                    1,0,0,0,1,
                    1,0,0,0,1,
                    0,1,1,1,0,
                ],
                // 9
                [
                    0,1,1,1,0,
                    1,0,0,0,1,
                    1,0,0,0,1,
                    0,1,1,1,1,
                    0,0,0,0,1,
                    0,0,0,1,0,
                    0,1,1,0,0,
                ],
            ];
            
            // Calcul de la largeur totale des chiffres pour centrage
            let digit_width = 5;
            let digit_height = 7;
            let digit_scale = 3; // Facteur d'échelle pour agrandir les chiffres
            let digit_spacing = 2 * digit_scale; // Espace entre les chiffres 
            let total_width = (score_text.len() as u32) * (digit_width * digit_scale + digit_spacing) - digit_spacing;
            
            // Position de départ pour centrer les chiffres
            let start_x = score_bar_x + (score_bar_width - total_width) / 2;
            let start_y = score_bar_y + (score_bar_height - digit_height * digit_scale) / 2;
            
            let mut current_x = start_x;
            
            // Dessiner chaque chiffre
            for digit_char in score_text.chars() {
                if let Some(digit) = digit_char.to_digit(10) {
                    let digit = digit as usize;
                    
                    // Dessiner le chiffre agrandi
                    for y in 0..digit_height {
                        for x in 0..digit_width {
                            let idx = (y * digit_width + x) as usize;
                            if digit_patterns[digit][idx] == 1 {
                                // Dessiner un pixel agrandi (carré de digit_scale x digit_scale)
                                for sy in 0..digit_scale {
                                    for sx in 0..digit_scale {
                                        let px = current_x + x * digit_scale + sx;
                                        let py = start_y + y * digit_scale + sy;
                                        
                                        if px < width as u32 && py < height as u32 {
                                            background.put_pixel(px, py, Rgba([255, 255, 255, 255]));
                                        }
                                    }
                                }
                            }
                        }
                    }
                    
                    current_x += digit_width * digit_scale + digit_spacing;
                }
            }
            
            // Convertir l'image en bytes PNG
            let mut bytes: Vec<u8> = Vec::new();
            
            match image::write_buffer_with_format(&mut Cursor::new(&mut bytes), 
                                                 &background.clone().into_raw(), 
                                                 background.width(), 
                                                 background.height(), 
                                                 image::ColorType::Rgba8, 
                                                 ImageFormat::Png) {
                Ok(_) => Some(bytes),
                Err(e) => {
                    error!("Erreur lors de la conversion de l'image en bytes: {:?}", e);
                    None
                }
            }
        },
        None => None
    }
}

fn create_circular_avatar(image: image::DynamicImage) -> RgbaImage {
    let width = image.width();
    let height = image.height();
    let mut circular_image = ImageBuffer::new(width, height);
    
    let center_x = width as f32 / 2.0;
    let center_y = height as f32 / 2.0;
    let radius = width.min(height) as f32 / 2.0;
    
    for (x, y, pixel) in image.to_rgba8().enumerate_pixels() {
        let dx = x as f32 - center_x;
        let dy = y as f32 - center_y;
        let distance = (dx * dx + dy * dy).sqrt();
        
        if distance <= radius {
            circular_image.put_pixel(x, y, *pixel);
        } else {
            circular_image.put_pixel(x, y, Rgba([0, 0, 0, 0]));
        }
    }
    
    circular_image
}

async fn download_image(url: &str) -> Option<Vec<u8>> {
    match Client::new().get(url).send().await {
        Ok(response) => {
            match response.bytes().await {
                Ok(bytes) => Some(bytes.to_vec()),
                Err(e) => {
                    error!("Erreur lors de la lecture des bytes de l'image: {:?}", e);
                    None
                }
            }
        },
        Err(e) => {
            error!("Erreur lors du téléchargement de l'image: {:?}", e);
            None
        }
    }
}

#[derive(Clone)]
pub struct Card {
    pub value: String,
    pub suit: String,
}

impl Card {
    pub fn new(value: String, suit: String) -> Self {
        Self { value, suit }
    }
    
    pub fn to_string(&self) -> String {
        format!("{} de {}", self.value, self.suit)
    }
    
    pub fn value(&self) -> i32 {
        match self.value.as_str() {
            "As" => 11,
            "Roi" | "Dame" | "Valet" => 10,
            _ => self.value.parse::<i32>().unwrap_or(0)
        }
    }
}

pub fn generate_card() -> (Card, i32) {
    use rand::{thread_rng, Rng};
    
    let suits = ["Trèfle", "Pique", "Coeur", "Carreau"];
    let values = ["As", "2", "3", "4", "5", "6", "7", "8", "9", "10", "Valet", "Dame", "Roi"];
    
    let mut rng = thread_rng();
    let suit = suits[rng.gen_range(0..suits.len())].to_string();
    let value_idx = rng.gen_range(0..values.len());
    let value = values[value_idx].to_string();
    
    let card = Card::new(value, suit);
    let card_value = card.value();
    
    (card, card_value)
}

fn get_digit_pattern(digit: usize) -> [bool; 35] {
    match digit {
        0 => [
            true,  true,  true,  true,  true,
            true,  false, false, false, true,
            true,  false, false, false, true,
            true,  false, false, false, true,
            true,  false, false, false, true,
            true,  false, false, false, true,
            true,  true,  true,  true,  true,
        ],
        1 => [
            false, false, true,  false, false,
            false, false, true,  false, false,
            false, false, true,  false, false,
            false, false, true,  false, false,
            false, false, true,  false, false,
            false, false, true,  false, false,
            false, false, true,  false, false,
        ],
        2 => [
            true,  true,  true,  true,  true,
            false, false, false, false, true,
            false, false, false, false, true,
            true,  true,  true,  true,  true,
            true,  false, false, false, false,
            true,  false, false, false, false,
            true,  true,  true,  true,  true,
        ],
        3 => [
            true,  true,  true,  true,  true,
            false, false, false, false, true,
            false, false, false, false, true,
            true,  true,  true,  true,  true,
            false, false, false, false, true,
            false, false, false, false, true,
            true,  true,  true,  true,  true,
        ],
        4 => [
            true,  false, false, false, true,
            true,  false, false, false, true,
            true,  false, false, false, true,
            true,  true,  true,  true,  true,
            false, false, false, false, true,
            false, false, false, false, true,
            false, false, false, false, true,
        ],
        5 => [
            true,  true,  true,  true,  true,
            true,  false, false, false, false,
            true,  false, false, false, false,
            true,  true,  true,  true,  true,
            false, false, false, false, true,
            false, false, false, false, true,
            true,  true,  true,  true,  true,
        ],
        6 => [
            true,  true,  true,  true,  true,
            true,  false, false, false, false,
            true,  false, false, false, false,
            true,  true,  true,  true,  true,
            true,  false, false, false, true,
            true,  false, false, false, true,
            true,  true,  true,  true,  true,
        ],
        7 => [
            true,  true,  true,  true,  true,
            false, false, false, false, true,
            false, false, false, false, true,
            false, false, false, false, true,
            false, false, false, false, true,
            false, false, false, false, true,
            false, false, false, false, true,
        ],
        8 => [
            true,  true,  true,  true,  true,
            true,  false, false, false, true,
            true,  false, false, false, true,
            true,  true,  true,  true,  true,
            true,  false, false, false, true,
            true,  false, false, false, true,
            true,  true,  true,  true,  true,
        ],
        9 => [
            true,  true,  true,  true,  true,
            true,  false, false, false, true,
            true,  false, false, false, true,
            true,  true,  true,  true,  true,
            false, false, false, false, true,
            false, false, false, false, true,
            true,  true,  true,  true,  true,
        ],
        _ => [false; 35], // Cas par défaut vide
    }
} 