use crate::models::Song;
use super::{PlaylistConfig, Playlist};
use super::filters::SongFilters;
use super::scoring::PlaylistScoring;
use super::utils::PlaylistNaming;

/// Main playlist generator
pub struct PlaylistGenerator {
    config: PlaylistConfig,
}

impl PlaylistGenerator {
    pub fn new(config: PlaylistConfig) -> Self {
        Self { config }
    }
    
    pub fn with_default_config() -> Self {
        Self::new(PlaylistConfig::default())
    }
    
    /// Generate a playlist from a collection of songs
    pub fn generate_playlist(
        &self,
        songs: Vec<Song>,
        playlist_name: Option<String>,
        target_length: Option<usize>,
    ) -> Playlist {
        let target_length = target_length.unwrap_or(20);
        
        // Filter songs and remove non-songs
        let mut filtered_songs: Vec<Song> = songs
            .into_iter()
            .filter(|song| SongFilters::should_include_song(song, &self.config))
            .collect();
        
        // Sort songs by preference score using configurable weights
        filtered_songs.sort_by(|a, b| {
            let score_a = PlaylistScoring::calculate_preference_score(a, &self.config);
            let score_b = PlaylistScoring::calculate_preference_score(b, &self.config);
            score_b.partial_cmp(&score_a).unwrap_or(std::cmp::Ordering::Equal)
        });
        
        // Take the target length
        filtered_songs.truncate(target_length);
        
        let metadata = PlaylistScoring::calculate_metadata(&filtered_songs);
        let quality_score = PlaylistScoring::calculate_quality_score(&filtered_songs, &metadata, &self.config);
        
        Playlist {
            songs: filtered_songs,
            name: playlist_name.unwrap_or_else(|| PlaylistNaming::generate_playlist_name(&metadata)),
            metadata,
            quality_score,
        }
    }
}
