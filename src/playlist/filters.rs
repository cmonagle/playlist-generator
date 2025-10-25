use super::{PlaylistConfig, PlayCountFilter};
use crate::models::Song;
use std::collections::HashMap;

/// Song filtering functionality using static helper functions
pub struct SongFilters;

impl SongFilters {
    /// Check if a track is an actual song (not an interlude, sketch, etc.)
    pub fn is_actual_song(song: &Song) -> bool {
        let title_lower = song.title.to_lowercase();

        // Check for non-song indicators in the title
        let non_song_patterns = [
            // Interludes and transitions (exact matches or with separators)
            "interlude",
            "intro",
            "outro",
            "prelude",
            "postlude",
            "bridge",
            "transition",
            "segue",
            // Sketches and fragments
            "sketch",
            "fragment",
            "snippet",
            "bits",
            "piece",
            // Spoken word and dialogue (but not songs with "spoken" in title)
            "monologue",
            "dialogue",
            "speech",
            "interview",
            "conversation",
            "discussion",
            // Ambient/atmospheric non-songs
            "atmosphere",
            "soundscape",
            "field recording",
            "rain",
            "ocean",
            "wind",
            "nature sounds",
            // Instrumentals that are likely non-songs
            "meditation",
            "mantra",
            "prayer",
            "chant",
            // Other non-musical content
            "silence",
            "pause",
            "break",
            "intermission",
            "announcement",
            "commercial",
            "ad",
            "test",
            "testing",
            "tuning",
            // Common abbreviated forms
            "int.",
            "intro.",
            "outro.",
            "interl.",
            // Track markers and numbering that suggest non-songs
            "untitled",
        ];

        // Check if title contains any non-song patterns (as whole words)
        let contains_non_song_pattern = non_song_patterns.iter().any(|pattern| {
            // Check if the pattern appears as a whole word at the beginning, end, or surrounded by spaces
            title_lower == *pattern ||
                title_lower.starts_with(&format!("{pattern} ")) ||
                title_lower.ends_with(&format!(" {pattern}")) ||
                title_lower.contains(&format!(" {pattern} ")) ||
                // Also check for patterns that are the entire title or standalone words
                title_lower.split_whitespace().any(|word| word == *pattern) ||
                // Check for patterns followed by colon (like "Interlude: Title")
                title_lower.starts_with(&format!("{pattern}:"))
        });

        // Additional heuristics
        let too_short = song.duration.is_some_and(|d| d < 60); // Less than 60 seconds
        let too_long = song.duration.is_some_and(|d| d > 600); // More than 10 minutes (likely DJ mix/compilation)

        // Check for titles that are just numbers or very short
        let is_just_number_or_short = title_lower.trim().len() <= 2
            || title_lower
                .trim()
                .chars()
                .all(|c| c.is_numeric() || c == '.' || c == '-');

        // Check for common non-song title patterns in parentheses
        let has_parenthetical_indicators = title_lower.contains("(interlude)") ||
            title_lower.contains("(intro)") ||
            title_lower.contains("(outro)") ||
            title_lower.contains("(sketch)") ||
            // Only filter short instrumentals (likely interludes), not long ones (likely actual songs)
            (title_lower.contains("(instrumental)") && song.duration.is_some_and(|d| d < 90));

        // Check if title starts with "track " followed by a number (common for untitled tracks)
        let is_track_number = title_lower.starts_with("track ")
            && title_lower
                .chars()
                .skip(6)
                .all(|c| c.is_numeric() || c.is_whitespace());

        // A song is considered "actual" if it doesn't match any exclusion criteria
        !contains_non_song_pattern
            && !too_short
            && !too_long
            && !is_just_number_or_short
            && !has_parenthetical_indicators
            && !is_track_number
    }

    /// Check if a song matches the acceptable genres filter
    pub fn matches_acceptable_genres(song: &Song, config: &PlaylistConfig) -> bool {
        // If no genre filter is set, accept all songs
        let Some(acceptable_genres) = &config.acceptable_genres else {
            return true;
        };

        // Check if the song matches any of the acceptable genre patterns
        song.matches_genre_patterns_string(acceptable_genres)
    }

    /// Check if a song doesn't match any unacceptable genres
    pub fn does_not_match_unacceptable_genres(song: &Song, config: &PlaylistConfig) -> bool {
        // If no unacceptable genre filter is set, accept all songs
        let Some(unacceptable_genres) = &config.unacceptable_genres else {
            return true;
        };

        // Check if the song does NOT match any of the unacceptable genre patterns
        !song.matches_genre_patterns_string(unacceptable_genres)
    }

    /// Check if a song matches the BPM thresholds filter
    pub fn matches_bpm_thresholds(song: &Song, config: &PlaylistConfig) -> bool {
        // If no BPM filter is set, accept all songs
        let Some(bpm_thresholds) = &config.bpm_thresholds else {
            return true;
        };

        // If song has no BPM data, accept it (neutral)
        let Some(song_bpm) = song.bpm else {
            return true;
        };

        // Check if song BPM is within the configured range
        song_bpm >= bpm_thresholds.min_bpm && song_bpm <= bpm_thresholds.max_bpm
    }

    /// Check if a song matches the play count filter
    pub fn matches_play_count_filter(song: &Song, config: &PlaylistConfig, all_songs: &[Song]) -> bool {
        // If no play count filter is set, accept all songs
        let Some(play_count_filter) = &config.preference_weights.play_count_filter else {
            return true;
        };

        match play_count_filter {
            PlayCountFilter::None => true,
            
            PlayCountFilter::Exact { count } => {
                match count {
                    Some(target_count) => song.play_count == Some(*target_count),
                    None => song.play_count.is_none(), // Zero plays (never played)
                }
            },
            
            PlayCountFilter::Range { min, max } => {
                let song_play_count = song.play_count.unwrap_or(0);
                let min_ok = min.map_or(true, |m| song_play_count >= m);
                let max_ok = max.map_or(true, |m| song_play_count <= m);
                min_ok && max_ok
            },
            
            PlayCountFilter::Threshold { operator, count } => {
                let song_play_count = song.play_count.unwrap_or(0);
                match operator.as_str() {
                    "above" => song_play_count > *count,
                    "below" => song_play_count < *count,
                    "at_least" => song_play_count >= *count,
                    "at_most" => song_play_count <= *count,
                    _ => true, // Invalid operator, default to accepting
                }
            },
            
            PlayCountFilter::Percentile { direction, percent } => {
                // Calculate percentile thresholds from all songs
                let mut play_counts: Vec<u32> = all_songs
                    .iter()
                    .map(|s| s.play_count.unwrap_or(0))
                    .collect();
                play_counts.sort_unstable();
                
                let song_play_count = song.play_count.unwrap_or(0);
                
                match direction.as_str() {
                    "top" => {
                        // Top percentile = highest play counts
                        let threshold_index = ((1.0 - percent) * play_counts.len() as f32) as usize;
                        let threshold = play_counts.get(threshold_index).copied().unwrap_or(0);
                        song_play_count >= threshold
                    },
                    "bottom" => {
                        // Bottom percentile = lowest play counts  
                        let threshold_index = (percent * play_counts.len() as f32) as usize;
                        let threshold = play_counts.get(threshold_index).copied().unwrap_or(0);
                        song_play_count <= threshold
                    },
                    _ => true, // Invalid direction, default to accepting
                }
            }
        }
    }

    /// Apply all filters to determine if a song should be included
    pub fn should_include_song(song: &Song, config: &PlaylistConfig) -> bool {
        Self::is_actual_song(song)
            && Self::matches_acceptable_genres(song, config)
            && Self::does_not_match_unacceptable_genres(song, config)
            && Self::matches_bpm_thresholds(song, config)
    }

    /// Apply all filters including play count filter (requires access to all songs for percentile calculations)
    pub fn should_include_song_with_play_count_filter(song: &Song, config: &PlaylistConfig, all_songs: &[Song]) -> bool {
        Self::should_include_song(song, config)
            && Self::matches_play_count_filter(song, config, all_songs)
    }
}
