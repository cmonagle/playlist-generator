use std::vec;

use chrono::Local;
use rand::seq::SliceRandom;

/// Helper trait for string formatting
pub trait ToTitleCase {
    fn to_title_case(&self) -> String;
}

impl ToTitleCase for str {
    fn to_title_case(&self) -> String {
        self.split_whitespace()
            .map(|word| {
                let mut chars = word.chars();
                match chars.next() {
                    None => String::new(),
                    Some(first) => {
                        first.to_uppercase().collect::<String>() + &chars.as_str().to_lowercase()
                    }
                }
            })
            .collect::<Vec<_>>()
            .join(" ")
    }
}

/// Playlist naming utilities
pub struct PlaylistNaming;

impl PlaylistNaming {
    /// Generate a descriptive name for the playlist based on metadata
    pub fn generate_playlist_name(
        name: String,
        metadata: &crate::playlist::PlaylistMetadata,
    ) -> String {
        let day_of_week = Local::now().format("%A").to_string();
        // Include dominant genre only if it covers at least 51% of songs
        if let Some((genre, &count)) = metadata.genre_distribution.iter().max_by_key(|(_, c)| *c) {
            if metadata.total_songs > 0 && (count as f32 / metadata.total_songs as f32) >= 0.51 {
                return format!("{} {} {}", name, day_of_week, genre.to_title_case())
                    .to_lowercase();
            } else {
                let backup_playlist_name_suffixes = vec![
                    "tunes", "vibes", "jams", "melodies", "grooves", "beats", "rhythms", "sounds",
                    "tracks",
                ];
                // Pick a random suffix from the backup list
                let mut rng = rand::thread_rng();
                let random_suffix = backup_playlist_name_suffixes.choose(&mut rng).unwrap();
                return format!("{} {} {}", name, day_of_week, random_suffix).to_lowercase();
            }
        }
        // Fallback to base name
        name.clone()
    }
}
