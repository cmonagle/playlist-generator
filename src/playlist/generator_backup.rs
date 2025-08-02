use crate::models::Song;
use super::{PlaylistConfig, Playlist, PlaylistMetadata};
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
    
    /// Check if a track is an actual song (not an interlude, sketch, etc.)
    pub fn is_actual_song(&self, song: &Song) -> bool {
        let title_lower = song.title.to_lowercase();
        
        // Check for non-song indicators in the title
        let non_song_patterns = [
            // Interludes and transitions (exact matches or with separators)
            "interlude", "intro", "outro", "prelude", "postlude",
            "bridge", "transition", "segue",
            
            // Sketches and fragments
            "sketch", "fragment", "snippet", "bits", "piece",
            
            // Spoken word and dialogue (but not songs with "spoken" in title)
            "monologue", "dialogue", "speech", "interview", "conversation", "discussion",
            
            // Ambient/atmospheric non-songs
            "atmosphere", "soundscape", "field recording",
            "rain", "ocean", "wind", "nature sounds",
            
            // Instrumentals that are likely non-songs
            "meditation", "mantra", "prayer", "chant",
            
            // Other non-musical content
            "silence", "pause", "break", "intermission",
            "announcement", "commercial", "ad",
            "test", "testing", "tuning",
            
            // Common abbreviated forms
            "int.", "intro.", "outro.", "interl.",
            
            // Track markers and numbering that suggest non-songs
            "untitled",
        ];
        
        // Check if title contains any non-song patterns (as whole words)
        let contains_non_song_pattern = non_song_patterns.iter()
            .any(|pattern| {
                // Check if the pattern appears as a whole word at the beginning, end, or surrounded by spaces
                title_lower == *pattern || 
                title_lower.starts_with(&format!("{} ", pattern)) ||
                title_lower.ends_with(&format!(" {}", pattern)) ||
                title_lower.contains(&format!(" {} ", pattern)) ||
                // Also check for patterns that are the entire title or standalone words
                title_lower.split_whitespace().any(|word| word == *pattern) ||
                // Check for patterns followed by colon (like "Interlude: Title")
                title_lower.starts_with(&format!("{}:", pattern))
            });
        
        // Additional heuristics
        let too_short = song.duration.map_or(false, |d| d < 60); // Less than 60 seconds
        let too_long = song.duration.map_or(false, |d| d > 600); // More than 10 minutes (likely DJ mix/compilation)
        
        // Check for titles that are just numbers or very short
        let is_just_number_or_short = title_lower.trim().len() <= 2 || 
            title_lower.trim().chars().all(|c| c.is_numeric() || c == '.' || c == '-');
        
        // Check for common non-song title patterns in parentheses
        let has_parenthetical_indicators = title_lower.contains("(interlude)") ||
            title_lower.contains("(intro)") ||
            title_lower.contains("(outro)") ||
            title_lower.contains("(sketch)") ||
            // Only filter short instrumentals (likely interludes), not long ones (likely actual songs)
            (title_lower.contains("(instrumental)") && song.duration.map_or(false, |d| d < 90));
        
        // Check if title starts with "track " followed by a number (common for untitled tracks)
        let is_track_number = title_lower.starts_with("track ") && 
            title_lower.chars().skip(6).all(|c| c.is_numeric() || c.is_whitespace());
        
        // A song is considered "actual" if it doesn't match any exclusion criteria
        !contains_non_song_pattern && 
        !too_short && 
        !too_long && 
        !is_just_number_or_short && 
        !has_parenthetical_indicators &&
        !is_track_number
    }
    
    /// Check if a song matches the acceptable genres filter
    pub fn matches_acceptable_genres(&self, song: &Song) -> bool {
        // If no genre filter is set, accept all songs
        let Some(acceptable_genres) = &self.config.acceptable_genres else {
            return true;
        };
        
        // Check if the song matches any of the acceptable genre patterns
        song.matches_genre_patterns_string(acceptable_genres)
    }
    
    /// Check if a song matches the BPM thresholds filter
    pub fn matches_bpm_thresholds(&self, song: &Song) -> bool {
        // If no BPM filter is set, accept all songs
        let Some(bpm_thresholds) = &self.config.bpm_thresholds else {
            return true;
        };
        
        // If song has no BPM data, accept it (neutral)
        let Some(song_bpm) = song.bpm else {
            return true;
        };
        
        // Check if song BPM is within the configured range
        song_bpm >= bpm_thresholds.min_bpm && song_bpm <= bpm_thresholds.max_bpm
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
            .filter(|song| {
                // First check if it's actually a song (not an interlude, sketch, etc.)
                self.is_actual_song(song) &&
                // Then check if it matches acceptable genres
                self.matches_acceptable_genres(song) &&
                // Then check if it matches BPM thresholds
                self.matches_bpm_thresholds(song)
            })
            .collect();
        
        // Sort songs by preference score using configurable weights
        filtered_songs.sort_by(|a, b| {
            let score_a = self.calculate_preference_score(a);
            let score_b = self.calculate_preference_score(b);
            score_b.partial_cmp(&score_a).unwrap_or(std::cmp::Ordering::Equal)
        });
        
        // Take the target length
        filtered_songs.truncate(target_length);
        
        let metadata = self.calculate_metadata(&filtered_songs);
        let quality_score = self.calculate_quality_score(&filtered_songs, &metadata);
        
        Playlist {
            name: playlist_name.unwrap_or_else(|| self.generate_playlist_name(&metadata)),
            songs: filtered_songs,
            quality_score,
            metadata,
        }
    }
    
    /// Calculate a preference score for a song based on configurable weights
    pub fn calculate_preference_score(&self, song: &Song) -> f32 {
        let weights = &self.config.preference_weights;
        let mut score = 0.0;
        
        // Starred tracks get a boost
        if song.starred.is_some() {
            score += weights.starred_boost;
        }
        
        // Play count contributes to score
        if let Some(play_count) = song.play_count {
            if weights.discovery_mode {
                // Discovery mode: prioritize low play counts
                if play_count == 0 {
                    score += 50.0;  // Unplayed songs get highest priority
                } else if play_count <= 2 {
                    score += 30.0 - (play_count as f32 * 5.0);  // Low play count gets good priority
                } else {
                    // Higher play count gets lower priority (inverted)
                    score += 10.0 - (play_count as f32).log10().min(10.0) * 2.0;
                }
            } else {
                // Normal mode: higher play count = better
                score += (play_count as f32).log10().max(0.0) * weights.play_count_weight;
            }
        } else if weights.discovery_mode {
            // Songs without play count data get medium priority in discovery mode
            score += 25.0;
        }
        
        // Apply recency penalty
        if let Some(played_str) = &song.played {
            if let Ok(days_since_played) = self.parse_days_since_played(played_str) {
                let penalty_threshold = if weights.discovery_mode { 3.0 } else { 7.0 };
                if days_since_played < penalty_threshold {
                    let penalty_multiplier = if weights.discovery_mode { 2.0 } else { weights.recency_penalty_weight };
                    let recency_penalty = (penalty_threshold - days_since_played) * penalty_multiplier;
                    score -= recency_penalty;
                }
            }
        }
        
        // Add randomness factor
        let randomness_multiplier = if weights.discovery_mode { 20 } else { 10 };
        score += (song.id.len() % randomness_multiplier) as f32 * weights.randomness_factor;
        
        score
    }
    
    /// Parse the last played timestamp and calculate days since played
    fn parse_days_since_played(&self, played_str: &str) -> Result<f32, Box<dyn std::error::Error>> {
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
    pub fn calculate_metadata(&self, songs: &[Song]) -> PlaylistMetadata {
        if songs.is_empty() {
            return PlaylistMetadata {
                total_duration: 0,
                average_bpm: 0.0,
                bpm_range: (0, 0),
                genre_distribution: HashMap::new(),
                artist_count: 0,
                era_span: (None, None),
                avg_popularity: 0.0,
            };
        }
        
        let total_duration: u32 = songs.iter()
            .filter_map(|s| s.duration)
            .sum();
        
        let bpms: Vec<u32> = songs.iter()
            .filter_map(|s| s.bpm)
            .collect();
        
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
        
        let artist_count = songs.iter()
            .map(|s| &s.artist)
            .collect::<std::collections::HashSet<_>>()
            .len();
        
        let years: Vec<u32> = songs.iter()
            .filter_map(|s| s.year)
            .collect();
        
        let era_span = if years.is_empty() {
            (None, None)
        } else {
            (Some(*years.iter().min().unwrap()), Some(*years.iter().max().unwrap()))
        };
        
        let avg_popularity = songs.iter()
            .filter_map(|s| s.play_count)
            .sum::<u32>() as f32 / songs.len() as f32;
        
        PlaylistMetadata {
            total_duration,
            average_bpm,
            bpm_range,
            genre_distribution,
            artist_count,
            era_span,
            avg_popularity,
        }
    }
    
    /// Calculate a quality score for the playlist (0.0 to 1.0)
    pub fn calculate_quality_score(&self, songs: &[Song], metadata: &PlaylistMetadata) -> f32 {
        if songs.is_empty() {
            return 0.0;
        }
        
        let artist_diversity_score = (metadata.artist_count as f32 / songs.len() as f32).min(1.0);
        
        // Calculate genre coherence based on distribution balance
        let genre_coherence_score = self.calculate_genre_coherence_score(&metadata.genre_distribution, songs.len());
        
        // Calculate era cohesion (how well the years fit together)
        let era_cohesion_score = self.calculate_era_cohesion_score(&metadata.era_span, songs);
        
        // Calculate popularity balance (avoid too many popular or unpopular songs)
        let popularity_balance_score = self.calculate_popularity_balance_score(songs);
        
        // Calculate BPM transition smoothness
        let bpm_transition_score = self.calculate_bpm_transition_score(songs);
        
        // Weighted average
        let weights = &self.config.quality_weights;
        weights.artist_diversity * artist_diversity_score +
        weights.bpm_transition_smoothness * bpm_transition_score +
        weights.genre_coherence * genre_coherence_score +
        weights.popularity_balance * popularity_balance_score +
        weights.era_cohesion * era_cohesion_score
    }
    
    /// Generate a descriptive name for the playlist
    fn generate_playlist_name(&self, metadata: &PlaylistMetadata) -> String {
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
    
    /// Calculate genre coherence preference score based on distribution
    pub fn calculate_genre_coherence_score(&self, genre_distribution: &HashMap<String, usize>, total_songs: usize) -> f32 {
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
        let normalized_entropy = if max_entropy > 0.0 { entropy / max_entropy } else { 0.0 };
        
        // Return coherence score: 1.0 = highly coherent, 0.0 = highly diverse
        // The preference weight will determine if this is good or bad
        1.0 - normalized_entropy
    }
    
    /// Calculate era cohesion preference score
    pub fn calculate_era_cohesion_score(&self, era_span: &(Option<u32>, Option<u32>), _songs: &[Song]) -> f32 {
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
    pub fn calculate_popularity_balance_score(&self, songs: &[Song]) -> f32 {
        let play_counts: Vec<u32> = songs.iter()
            .filter_map(|s| s.play_count)
            .collect();
        
        if play_counts.is_empty() {
            return 0.5; // Neutral when no play count data
        }
        
        if play_counts.len() == 1 {
            return 0.5; // Can't measure balance with one song
        }
        
        let mean = play_counts.iter().sum::<u32>() as f32 / play_counts.len() as f32;
        let variance = play_counts.iter()
            .map(|&pc| {
                let diff = pc as f32 - mean;
                diff * diff
            })
            .sum::<f32>() / play_counts.len() as f32;
        
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
            (2.0 - coefficient_of_variation).max(0.0).min(1.0)
        }
    }
    
    /// Calculate BPM transition smoothness preference score
    pub fn calculate_bpm_transition_score(&self, songs: &[Song]) -> f32 {
        let bpms: Vec<u32> = songs.iter()
            .filter_map(|s| s.bpm)
            .collect();
        
        if bpms.len() < 2 {
            return 0.5; // Neutral when can't measure transitions
        }
        
        // Calculate average BPM difference between adjacent songs
        let mut total_smoothness_score = 0.0;
        let mut transition_count = 0;
        
        for i in 0..bpms.len() - 1 {
            let diff = (bpms[i + 1] as i32 - bpms[i] as i32).abs() as u32;
            
            // Calculate smoothness: 1.0 = very smooth, 0.0 = very jarring
            let smoothness = if diff <= 5 {
                1.0 // Very smooth transition
            } else if diff <= 15 {
                1.0 - (diff - 5) as f32 / 10.0 * 0.3 // Score 0.7-1.0
            } else if diff <= 30 {
                0.7 - (diff - 15) as f32 / 15.0 * 0.4 // Score 0.3-0.7
            } else {
                (0.3 - (diff - 30) as f32 / 70.0 * 0.3).max(0.0) // Score 0.0-0.3
            };
            
            total_smoothness_score += smoothness;
            transition_count += 1;
        }
        
        total_smoothness_score / transition_count as f32
    }
}

// Helper trait for string formatting
trait ToTitleCase {
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
