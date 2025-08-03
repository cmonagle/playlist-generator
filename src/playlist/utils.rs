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
        // Include dominant genre only if it covers at least 40% of songs
        if let Some((genre, &count)) = metadata.genre_distribution.iter().max_by_key(|(_, c)| *c) {
            if metadata.total_songs > 0 && (count as f32 / metadata.total_songs as f32) >= 0.4 {
                return format!("{} {} {}", name, day_of_week, genre.to_title_case()).to_lowercase();
            } else {
                let backup_playlist_name_suffixes = vec![
                    "tunes",
                    "vibes",
                    "jams",
                    "melodies",
                    "grooves",
                    "beats",
                    "rhythms",
                    "sounds",
                    "tracks"
                ];
                // Pick a random suffix from the backup list
                let mut rng = rand::thread_rng();
                let random_suffix = backup_playlist_name_suffixes
                    .choose(&mut rng)
                    .unwrap();
                return format!("{} {} {}", name, day_of_week, random_suffix)
                    .to_lowercase();
            }
        }
        // Fallback to base name
        name.clone()
    }
}

/// Playlist ordering utilities
pub struct PlaylistOrdering;

impl PlaylistOrdering {
    /// Create an optimal sequence considering multiple transition rules
    /// This balances artist diversity, BPM transitions, and other constraints
    pub fn create_optimal_sequence(
        songs: Vec<crate::models::Song>,
        target_length: usize,
        transition_rules: &crate::playlist::TransitionRules,
    ) -> Vec<crate::models::Song> {
        if songs.is_empty() {
            return Vec::new();
        }

        let target_length = target_length.min(songs.len());

        // Use a greedy approach that considers multiple factors
        let mut selected = Vec::new();
        let mut remaining = songs;

        // Start with the highest-scored song
        if let Some(first_song) = remaining.pop() {
            selected.push(first_song);
        }

        // Build the rest of the playlist considering transitions
        while selected.len() < target_length && !remaining.is_empty() {
            let current_song = selected.last().unwrap();
            let mut best_candidate_index = 0;
            let mut best_score = f32::NEG_INFINITY;

            // Evaluate each remaining song as the next candidate
            for (i, candidate) in remaining.iter().enumerate() {
                let transition_score = Self::calculate_transition_score(
                    current_song,
                    candidate,
                    &selected,
                    transition_rules,
                );

                if transition_score > best_score {
                    best_score = transition_score;
                    best_candidate_index = i;
                }
            }

            // Add the best candidate to the playlist
            let next_song = remaining.remove(best_candidate_index);
            selected.push(next_song);
        }

        selected
    }

    /// Calculate a transition score between two songs considering multiple factors
    fn calculate_transition_score(
        current: &crate::models::Song,
        candidate: &crate::models::Song,
        playlist_so_far: &[crate::models::Song],
        rules: &crate::playlist::TransitionRules,
    ) -> f32 {
        let mut score = 0.0;

        // Artist diversity penalty (check recent window)
        let artist_penalty = Self::calculate_artist_diversity_score(
            candidate,
            playlist_so_far,
            rules.avoid_artist_repeats_within,
        );
        score += artist_penalty * 0.4; // 40% weight

        // BPM transition score
        let bpm_score = Self::calculate_bpm_transition_score(
            current,
            candidate,
            rules.max_bpm_jump,
            rules.preferred_bpm_change,
        );
        score += bpm_score * 0.6; // 60% weight

        score
    }

    /// Calculate artist diversity score (higher is better)
    fn calculate_artist_diversity_score(
        candidate: &crate::models::Song,
        playlist_so_far: &[crate::models::Song],
        avoid_repeats_within: usize,
    ) -> f32 {
        if avoid_repeats_within == 0 {
            return 0.0; // No artist diversity preference
        }

        let candidate_artist = candidate.artist.to_lowercase();
        let window_start = playlist_so_far.len().saturating_sub(avoid_repeats_within);
        let recent_window = &playlist_so_far[window_start..];

        // Check if artist appears in recent window
        for song in recent_window {
            if song.artist.to_lowercase() == candidate_artist {
                return -1.0; // Heavy penalty for recent artist repeat
            }
        }

        1.0 // Bonus for artist diversity
    }

    /// Calculate BPM transition score (higher is better)
    fn calculate_bpm_transition_score(
        current: &crate::models::Song,
        candidate: &crate::models::Song,
        max_bpm_jump: u32,
        preferred_bpm_change: i32,
    ) -> f32 {
        // Handle cases where BPM data is missing
        let (current_bpm, candidate_bpm) = match (current.bpm, candidate.bpm) {
            (Some(curr), Some(cand)) => (curr as i32, cand as i32),
            _ => return 0.0, // Neutral score if BPM data unavailable
        };

        let bpm_change = candidate_bpm - current_bpm;
        let bpm_jump = bpm_change.abs() as u32;

        // Penalty for exceeding max BPM jump
        if bpm_jump > max_bpm_jump {
            return -0.5;
        }

        // Bonus for following preferred BPM direction
        let direction_bonus = if preferred_bpm_change == 0 {
            0.0 // No preference
        } else if (preferred_bpm_change > 0 && bpm_change > 0)
            || (preferred_bpm_change < 0 && bpm_change < 0)
        {
            0.3 // Following preferred direction
        } else {
            -0.1 // Going against preferred direction
        };

        // Smoothness bonus (smaller jumps are better)
        let smoothness_bonus = 1.0 - (bpm_jump as f32 / max_bpm_jump as f32);

        smoothness_bonus + direction_bonus
    }
}
