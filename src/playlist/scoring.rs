use super::{PlaylistConfig, PlaylistMetadata};
use crate::models::Song;
use std::collections::HashMap;

/// Scoring and calculation functionality
pub struct PlaylistScoring;

impl PlaylistScoring {
    /// Calculate a preference score for a song based on configurable weights
    pub fn calculate_preference_score(song: &Song, config: &PlaylistConfig) -> f32 {
        let weights = &config.preference_weights;
        let mut score = 0.0;

        // Starred tracks get a boost
        if song.starred.is_some() {
            score += weights.starred_boost;
        }

        // Play count contributes to score
        if let Some(play_count) = song.play_count {
            if weights.discovery_mode {
                // In discovery mode, invert play count logic - lower play counts get higher scores
                // Use a base value and subtract play count to favor less-played songs
                let base_discovery_score = 50.0; // Base score for discovery
                score += (base_discovery_score - (play_count as f32).min(base_discovery_score))
                    * weights.play_count_weight;
            } else {
                // Normal mode: higher play counts get higher scores
                score += play_count as f32 * weights.play_count_weight;
            }
        } else if weights.discovery_mode {
            // In discovery mode, give unplayed songs the highest boost
            score += 50.0 * weights.play_count_weight;
        }

        // Apply recency penalty (simplified since we don't have recency_penalty_days/strength)
        if let Some(played_str) = &song.played {
            if let Ok(days_since_played) = Self::parse_days_since_played(played_str) {
                // Apply recency penalty - songs played recently get lower scores
                if days_since_played < 14.0 {
                    // Use 7 days as default
                    let penalty_factor = 1.0 - (days_since_played / 7.0);
                    score -= weights.recency_penalty_weight * penalty_factor;
                }
            }
        }

        // Add randomness factor
        let randomness_multiplier = if weights.discovery_mode { 20 } else { 10 };
        score += (song.id.len() % randomness_multiplier) as f32 * weights.randomness_factor;

        score
    }

    /// Parse the last played timestamp and calculate days since played
    pub fn parse_days_since_played(played_str: &str) -> Result<f32, Box<dyn std::error::Error>> {
        use chrono::{DateTime, Utc};

        // Try to parse as ISO 8601 format (most common for OpenSubsonic)
        let played_time = if let Ok(dt) = DateTime::parse_from_rfc3339(played_str) {
            dt.with_timezone(&Utc)
        } else if let Ok(dt) = DateTime::parse_from_str(played_str, "%Y-%m-%dT%H:%M:%S%.fZ") {
            // Alternative format without timezone
            dt.with_timezone(&Utc)
        } else if let Ok(dt) = DateTime::parse_from_str(played_str, "%Y-%m-%d %H:%M:%S") {
            // Space-separated format
            dt.with_timezone(&Utc)
        } else {
            // If all parsing fails, assume it was played recently (conservative approach)
            return Ok(0.5); // Half a day ago (will trigger recency penalty)
        };

        let now = Utc::now();
        let duration = now.signed_duration_since(played_time);
        let days = duration.num_days() as f32 + (duration.num_hours() % 24) as f32 / 24.0;

        Ok(days.max(0.0)) // Ensure we don't return negative days
    }

    /// Calculate metadata for a playlist
    pub fn calculate_metadata(songs: &[Song]) -> PlaylistMetadata {
        if songs.is_empty() {
            return PlaylistMetadata {
                total_duration: 0,
                average_bpm: 0.0,
                bpm_range: (0, 0),
                genre_distribution: HashMap::new(),
                artist_count: 0,
                era_span: (None, None),
                avg_popularity: 0.0,
                total_songs: 0,
            };
        }

        let total_duration: u32 = songs.iter().filter_map(|s| s.duration).sum();

        let bpms: Vec<u32> = songs.iter().filter_map(|s| s.bpm).collect();

        let average_bpm = if bpms.is_empty() {
            0.0
        } else {
            bpms.iter().sum::<u32>() as f32 / bpms.len() as f32
        };

        let bpm_range = if bpms.is_empty() {
            (0, 0)
        } else {
            (*bpms.iter().min().unwrap(), *bpms.iter().max().unwrap())
        };

        let mut genre_distribution = HashMap::new();
        for song in songs {
            for genre in song.get_all_genres() {
                *genre_distribution.entry(genre.to_lowercase()).or_insert(0) += 1;
            }
        }

        let artist_count = songs
            .iter()
            .map(|s| &s.artist)
            .collect::<std::collections::HashSet<_>>()
            .len();

        let years: Vec<u32> = songs.iter().filter_map(|s| s.year).collect();

        let era_span = if years.is_empty() {
            (None, None)
        } else {
            (
                Some(*years.iter().min().unwrap()),
                Some(*years.iter().max().unwrap()),
            )
        };

        let avg_popularity =
            songs.iter().filter_map(|s| s.play_count).sum::<u32>() as f32 / songs.len() as f32;

        PlaylistMetadata {
            total_duration,
            average_bpm,
            bpm_range,
            genre_distribution,
            artist_count,
            era_span,
            avg_popularity,
            total_songs: songs.len(),
        }
    }

    /// Calculate a quality score for the playlist (0.0 to 1.0)
    pub fn calculate_quality_score(
        songs: &[Song],
        metadata: &PlaylistMetadata,
        config: &PlaylistConfig,
    ) -> f32 {
        if songs.is_empty() {
            return 0.0;
        }

        // Calculate core quality metrics with their weights
        let weights = &config.quality_weights;
        let genre_coherence_score = Self::calculate_genre_coherence_score(&metadata.genre_distribution, songs.len()) * weights.genre_coherence;
        let era_cohesion_score = Self::calculate_era_cohesion_score(&metadata.era_span) * weights.era_cohesion;
        let popularity_balance_score = Self::calculate_popularity_balance_score(songs) * weights.popularity_balance;
        let artist_diversity_score = Self::calculate_artist_diversity_score(songs) * weights.artist_diversity;
        let bpm_smoothness_score = Self::calculate_bpm_transition_smoothness_score(songs) * weights.bpm_transition_smoothness;

        // Sum weighted scores and normalize by total weight sum
        let total_score = genre_coherence_score + popularity_balance_score + era_cohesion_score + artist_diversity_score + bpm_smoothness_score;
        let total_weight = weights.genre_coherence + weights.popularity_balance + weights.era_cohesion + weights.artist_diversity + weights.bpm_transition_smoothness;
        
        if total_weight > 0.0 {
            total_score / total_weight
        } else {
            0.5 // Neutral score if no weights are set
        }
    }

    /// Calculate genre coherence preference score based on distribution
    pub fn calculate_genre_coherence_score(
        genre_distribution: &HashMap<String, usize>,
        total_songs: usize,
    ) -> f32 {
        if genre_distribution.is_empty() || total_songs == 0 {
            return 0.5; // Neutral when no data
        }

        let genre_count = genre_distribution.len();

        // Calculate how coherent/diverse the genres are
        // More genres = more diversity (lower coherence)
        // Fewer genres = more coherence (lower diversity)

        if genre_count == 1 {
            return 1.0; // Maximum coherence - single genre
        }

        // Calculate distribution entropy (higher = more diverse)
        let mut entropy = 0.0;
        for &count in genre_distribution.values() {
            let probability = count as f32 / total_songs as f32;
            if probability > 0.0 {
                entropy -= probability * probability.log2();
            }
        }

        // Normalize entropy based on genre count
        let max_entropy = (genre_count as f32).log2();
        let normalized_entropy = if max_entropy > 0.0 {
            entropy / max_entropy
        } else {
            0.0
        };

        // Return coherence score: 1.0 = highly coherent, 0.0 = highly diverse
        // The preference weight will determine if this is good or bad
        1.0 - normalized_entropy
    }

    /// Calculate era cohesion preference score
    pub fn calculate_era_cohesion_score(era_span: &(Option<u32>, Option<u32>)) -> f32 {
        match era_span {
            (Some(min_year), Some(max_year)) => {
                let span = max_year - min_year;

                // Return cohesion score: 1.0 = highly cohesive (same era), 0.0 = highly diverse (many eras)
                // The preference weight will determine if this is desirable

                if span <= 2 {
                    1.0 // Very cohesive - same or adjacent years
                } else if span <= 10 {
                    0.8 - (span as f32 - 2.0) / 8.0 * 0.3 // Score 0.5-0.8 for decade span
                } else if span <= 20 {
                    0.5 - (span as f32 - 10.0) / 10.0 * 0.3 // Score 0.2-0.5 for two decades
                } else {
                    (0.2 - (span as f32 - 20.0) / 50.0 * 0.2).max(0.0) // Score 0.0-0.2 for longer spans
                }
            }
            _ => {
                // If we don't have year data, return neutral
                0.5
            }
        }
    }

    /// Calculate popularity balance preference score
    pub fn calculate_popularity_balance_score(songs: &[Song]) -> f32 {
        let play_counts: Vec<u32> = songs.iter().filter_map(|s| s.play_count).collect();

        if play_counts.is_empty() {
            return 0.5; // Neutral when no play count data
        }

        if play_counts.len() == 1 {
            return 0.5; // Can't measure balance with one song
        }

        let mean = play_counts.iter().sum::<u32>() as f32 / play_counts.len() as f32;
        let variance = play_counts
            .iter()
            .map(|&pc| {
                let diff = pc as f32 - mean;
                diff * diff
            })
            .sum::<f32>()
            / play_counts.len() as f32;

        let std_dev = variance.sqrt();

        // Calculate how balanced the popularity distribution is
        let coefficient_of_variation = if mean > 0.0 { std_dev / mean } else { 0.0 };

        // Return balance score: 1.0 = perfectly balanced mix, 0.0 = extremely unbalanced
        // The preference weight will determine if balance is desired

        if coefficient_of_variation <= 0.3 {
            // Very low variation = all songs similar popularity (not balanced)
            coefficient_of_variation / 0.3 * 0.5
        } else if coefficient_of_variation <= 1.0 {
            // Good variation = balanced mix of popular and unpopular
            0.5 + (coefficient_of_variation - 0.3) / 0.7 * 0.5
        } else {
            // Very high variation = extremely unbalanced
            (2.0 - coefficient_of_variation).clamp(0.0, 1.0)
        }
    }

    /// Calculate artist diversity score
    pub fn calculate_artist_diversity_score(songs: &[Song]) -> f32 {
        if songs.len() <= 1 {
            return 1.0; // Single song = maximum "diversity" (no repetition possible)
        }

        let total_songs = songs.len();
        let unique_artists = songs
            .iter()
            .map(|s| &s.artist)
            .collect::<std::collections::HashSet<_>>()
            .len();

        // Return diversity score: 1.0 = all different artists, 0.0 = all same artist
        unique_artists as f32 / total_songs as f32
    }

    /// Calculate BPM transition smoothness score
    pub fn calculate_bpm_transition_smoothness_score(songs: &[Song]) -> f32 {
        if songs.len() <= 1 {
            return 1.0; // Single song = maximum smoothness (no transitions)
        }

        let bpm_transitions: Vec<u32> = songs
            .windows(2)
            .filter_map(|pair| {
                if let (Some(bpm1), Some(bpm2)) = (pair[0].bpm, pair[1].bpm) {
                    Some((bpm1 as i32 - bpm2 as i32).abs() as u32)
                } else {
                    None
                }
            })
            .collect();

        if bpm_transitions.is_empty() {
            return 0.5; // Neutral when no BPM data available
        }

        // Calculate average BPM jump
        let avg_jump = bpm_transitions.iter().sum::<u32>() as f32 / bpm_transitions.len() as f32;

        // Return smoothness score: 1.0 = very smooth (small jumps), 0.0 = very jarring (large jumps)
        // Use a reasonable scale where 30 BPM jump = 0.5 score
        let smoothness = (60.0 - avg_jump) / 60.0;
        smoothness.clamp(0.0_f32, 1.0_f32)
    }


}
