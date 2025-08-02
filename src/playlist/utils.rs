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
                    Some(first) => first.to_uppercase().collect::<String>() + &chars.as_str().to_lowercase(),
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
    pub fn generate_playlist_name(metadata: &crate::playlist::PlaylistMetadata) -> String {
        // Generate a name based on metadata
        if let Some(dominant_genre) = metadata.genre_distribution
            .iter()
            .max_by_key(|(_, count)| *count)
            .map(|(genre, _)| genre)
        {
            format!("{} Playlist", dominant_genre.to_title_case())
        } else {
            "My Playlist".to_string()
        }
    }
}
