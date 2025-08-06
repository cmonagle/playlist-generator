use crate::models::Song;
use super::config::PlaylistConfig;
use super::scoring::PlaylistScoring;
use std::collections::HashMap;

/// Handles transition scoring and rules between songs in a playlist
pub struct PlaylistTransitions;

impl PlaylistTransitions {
    /// Calculate how well a candidate song would fit with the current working playlist
    pub fn calculate_transition_score(
        config: &PlaylistConfig,
        current_playlist: &[Song],
        candidate: &Song,
    ) -> f32 {
        if current_playlist.is_empty() {
            return 0.5; // Neutral score for first song
        }

        let mut total_score = 0.0_f32;

        // 1. BPM transition - only check against the last song
        if let Some(last_song) = current_playlist.last() {
            let bpm_score = Self::calculate_bpm_transition_score(config, last_song, candidate);
            total_score += bpm_score; // Increased weight since artist repetition is now a hard constraint
        }

        // 2. Genre compatibility - check against the overall playlist genre distribution
        let genre_score = Self::calculate_genre_compatibility_score(config, current_playlist, candidate);
        total_score += genre_score; // Increased weight since artist repetition is now a hard constraint

        // Return the weighted sum (should be between 0.0 and 1.0 if weights sum to 1.0)
        total_score / 2.0 // Normalize by number of components
    }

    /// Calculate BPM transition score between two songs
    pub fn calculate_bpm_transition_score(config: &PlaylistConfig, song_a: &Song, song_b: &Song) -> f32 {
        if let (Some(bpm_a), Some(bpm_b)) = (song_a.bpm, song_b.bpm) {
            let actual_change = bpm_b as i32 - bpm_a as i32; // Note: changed to b - a for actual progression
            let ideal_change = config.transition_rules.preferred_bpm_change;
            
            // Perfect match
            if actual_change == ideal_change {
                return 1.0;
            }
            
            // Moving in the correct direction
            if (ideal_change > 0 && actual_change > 0) || (ideal_change < 0 && actual_change < 0) {
                // Score based on how close we are to ideal change (0.5 to 0.9)
                // Use max_bpm_jump as the range for normalization
                let max_jump = config.transition_rules.max_bpm_jump as f32;
                let closeness = 1.0 - (actual_change - ideal_change).abs() as f32 / max_jump;
                return 0.5 + (closeness * 0.4).min(0.4);
            } 
            
            // Moving in the wrong direction - scale penalty by how far wrong it goes
            if (ideal_change > 0 && actual_change < 0) || (ideal_change < 0 && actual_change > 0) {
                let max_jump = config.transition_rules.max_bpm_jump as f32;
                let wrong_magnitude = actual_change.abs() as f32;
                // Scale from 0.4 (small wrong direction) down to 0.1 (large wrong direction)
                let penalty = 0.4 - (0.3 * (wrong_magnitude / max_jump)).min(0.3);
                return penalty;
            }
            
            // No change when change is desired, or no preference on direction
            0.4
        } else {
            0.5 // Neutral when BPM data is missing
        }
    }

    /// Calculate genre compatibility with the playlist's overall genre profile
    pub fn calculate_genre_compatibility_score(
        config: &PlaylistConfig,
        current_playlist: &[Song],
        candidate: &Song,
    ) -> f32 {
        if current_playlist.is_empty() {
            return 0.5;
        }

        // Get candidate genres
        let candidate_genres = candidate.get_all_genres();
        if candidate_genres.is_empty() {
            return 0.5; // Neutral when no genre info
        }

        // Build genre frequency map from current playlist
        let mut playlist_genres = HashMap::new();
        for song in current_playlist {
            for genre in song.get_all_genres() {
                *playlist_genres.entry(genre.to_lowercase()).or_insert(0) += 1;
            }
        }

        if playlist_genres.is_empty() {
            return 0.5; // Neutral when no genre info in playlist
        }

        // Check if any genres match and find the highest frequency match
        let coherence_pref = config.quality_weights.genre_coherence;
        let mut best_frequency = 0.0;
        let mut has_matching_genre = false;

        for candidate_genre in candidate_genres {
            let genre_lower = candidate_genre.to_lowercase();
            if let Some(&frequency) = playlist_genres.get(&genre_lower) {
                has_matching_genre = true;
                let frequency_score = (frequency as f32 / current_playlist.len() as f32).min(1.0);
                best_frequency = if frequency_score > best_frequency { frequency_score } else { best_frequency };
            }
        }

        if has_matching_genre {
            // Song shares at least one genre - score based on the best frequency match
            // Higher coherence preference = higher reward for matching genres (0.5-1.0 range)
            0.5 + (best_frequency * 0.5 * coherence_pref)
        } else {
            // No matching genres - score based on coherence preference
            // High coherence (1.0) = minimal score (0.1) for genre mismatch
            // Low coherence (0.0) = high score (0.9) to encourage variety
            // At 0.5 coherence, score is neutral (0.5)
            0.9 - (0.8 * coherence_pref)
        }
    }

    /// Check if a candidate would violate artist repetition rules
    pub fn would_violate_artist_repetition(
        avoid_within: usize,
        current_playlist: &[Song],
        candidate: &Song,
    ) -> bool {
        // Check the last N songs (where N = avoid_artist_repeats_within)
        let check_count = avoid_within.min(current_playlist.len());
        let recent_songs = &current_playlist[current_playlist.len() - check_count..];

        // Check if candidate artist appears in recent songs
        for recent_song in recent_songs {
            if recent_song.artist == candidate.artist {
                return true; // Artist repetition found
            }
        }

        false // No artist repetition found
    }

    /// Check if a candidate would violate album repetition rules
    pub fn would_violate_album_repetition(
        avoid_within: usize,
        current_playlist: &[Song],
        candidate: &Song,
    ) -> bool {
        // Check the last N songs (where N = avoid_album_repeats_within)
        let check_count = avoid_within.min(current_playlist.len());
        let recent_songs = &current_playlist[current_playlist.len() - check_count..];

        // Check if candidate album appears in recent songs
        for recent_song in recent_songs {
            if recent_song.album == candidate.album {
                return true; // Album repetition found
            }
        }

        false // No album repetition found
    }

    /// Check if a candidate would violate the minimum days since last play rule
    pub fn would_violate_min_days_since_last_play(
        min_days: u32,
        candidate: &Song,
    ) -> bool {
        if let Some(played_str) = &candidate.played {
            if let Ok(days_since_played) = PlaylistScoring::parse_days_since_played(played_str) {
                return days_since_played < min_days as f32;
            }
        }
        false // Assume no violation if no play data is available
    }
}
