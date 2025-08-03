use crate::models::Song;
use super::{PlaylistConfig, Playlist};
use super::filters::SongFilters;
use super::scoring::PlaylistScoring;
use super::utils::{PlaylistNaming, PlaylistOrdering};

/// Main playlist generator
pub struct PlaylistGenerator {
    config: PlaylistConfig,
}

impl PlaylistGenerator {
    pub fn new(config: PlaylistConfig) -> Self {
        Self { config }
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
        
        // Take more songs than target to have options for ordering
        let selection_pool_size = (target_length as f32 * 1.5).ceil() as usize;
        filtered_songs.truncate(selection_pool_size.min(filtered_songs.len()));
        
        // Apply transition-aware ordering to create the final playlist
        let ordered_songs = PlaylistOrdering::create_optimal_sequence(
            filtered_songs, 
            target_length,
            &self.config.transition_rules
        );
        
        let metadata = PlaylistScoring::calculate_metadata(&ordered_songs);
        let quality_score = PlaylistScoring::calculate_quality_score(&ordered_songs, &metadata, &self.config);
        
        Playlist {
            songs: ordered_songs,
            name: PlaylistNaming::generate_playlist_name(playlist_name.unwrap_or("Daylist".to_string()), &metadata),
            base_name_pattern: self.config.name.clone(),
            metadata,
            quality_score,
        }
    }
}
