use crate::models::Song;
use std::collections::HashMap;

/// Represents a generated playlist with metadata
#[derive(Debug)]
pub struct Playlist {
    pub name: String,
    pub songs: Vec<Song>,
    pub quality_score: f32,
    pub metadata: PlaylistMetadata,
}

/// Metadata about the playlist composition
#[derive(Debug)]
pub struct PlaylistMetadata {
    pub total_duration: u32,  // in seconds
    pub average_bpm: f32,
    pub bpm_range: (u32, u32),
    pub genre_distribution: HashMap<String, usize>,
    pub artist_count: usize,
    pub era_span: (Option<u32>, Option<u32>),  // (min_year, max_year)
    pub avg_popularity: f32,
}
