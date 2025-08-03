use crate::models::Song;
use std::collections::HashMap;

/// Represents a song in a playlist with additional metadata from the generation process
#[derive(Debug, Clone)]
pub struct PlaylistSong {
    pub song: Song,
    pub transition_score: Option<f32>, // Score for how well this song transitions with the playlist
    pub quality_contribution: Option<f32>, // How this song affected overall playlist quality
    pub selection_reason: String, // Why this song was selected (e.g., "best candidate", "fallback")
}

impl PlaylistSong {
    pub fn with_metadata(
        song: Song,
        transition_score: f32,
        quality_contribution: f32,
        selection_reason: String,
    ) -> Self {
        Self {
            song,
            transition_score: Some(transition_score),
            quality_contribution: Some(quality_contribution),
            selection_reason,
        }
    }
}

/// Represents a generated playlist with metadata
#[derive(Debug)]
pub struct Playlist {
    pub name: String,
    pub songs: Vec<PlaylistSong>,
    pub quality_score: f32,
    pub metadata: PlaylistMetadata,
    pub base_name_pattern: String,
}

/// Metadata about the playlist composition
#[derive(Debug)]
pub struct PlaylistMetadata {
    pub total_duration: u32, // in seconds
    pub total_songs: usize,
    pub average_bpm: f32,
    pub bpm_range: (u32, u32),
    pub genre_distribution: HashMap<String, usize>,
    pub artist_count: usize,
    pub era_span: (Option<u32>, Option<u32>), // (min_year, max_year)
    pub avg_popularity: f32,
}
