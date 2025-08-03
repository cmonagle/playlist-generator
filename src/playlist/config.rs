use serde::{Deserialize, Serialize};

/// Configuration for playlist generation heuristics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlaylistConfig {
    pub name: String, // Name for this playlist configuration
    pub acceptable_genres: Option<Vec<String>>,
    pub unacceptable_genres: Option<Vec<String>>,
    pub bpm_thresholds: Option<BpmThresholds>,
    pub quality_weights: QualityWeights,
    pub transition_rules: TransitionRules,
    pub preference_weights: PreferenceWeights,
    pub target_length: Option<usize>, // Default target length for this playlist type
}

/// BPM range for playlist filtering
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BpmThresholds {
    pub min_bpm: u32,
    pub max_bpm: u32,
}

/// Preferences for different playlist characteristics (0.0 to 1.0)
/// Each value represents how much you want that characteristic:
/// 0.0 = minimize this characteristic, 1.0 = maximize this characteristic
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QualityWeights {
    pub artist_diversity: f32, // 0.0 = prefer same artists, 1.0 = prefer different artists
    pub bpm_transition_smoothness: f32, // 0.0 = allow big BPM jumps, 1.0 = prefer smooth transitions
    pub genre_coherence: f32, // 0.0 = prefer genre variety, 1.0 = prefer genre consistency
    pub popularity_balance: f32, // 0.0 = allow extreme popularity differences, 1.0 = prefer balanced mix
    pub era_cohesion: f32,       // 0.0 = prefer era variety, 1.0 = prefer same time period
}

/// Rules for transitions between songs
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TransitionRules {
    pub max_bpm_jump: u32,
    pub preferred_bpm_change: i32, // negative for slowdown, positive for speedup
    pub avoid_artist_repeats_within: usize, // number of songs
                                   // pub bpm_weight: f32,                     // Weight for BPM transition scoring (0.0 to 1.0)
                                   // pub artist_weight: f32,                  // Weight for artist repetition penalty (0.0 to 1.0)
                                   // pub genre_weight: f32,                   // Weight for genre compatibility scoring (0.0 to 1.0)
}

/// Weights for preference scoring (0.0 to 1.0)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PreferenceWeights {
    pub starred_boost: f32,          // Boost for starred tracks
    pub play_count_weight: f32,      // Weight for play count contribution
    pub recency_penalty_weight: f32, // Weight for recency penalty
    pub randomness_factor: f32,      // Random variation factor
    pub discovery_mode: bool,        // Use discovery scoring (inverts play count logic)
}

/// Collection of playlist configurations loaded from JSON
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlaylistConfigs {
    pub playlists: Vec<PlaylistConfig>,
}

/// Settings for iterative playlist generation with quality evaluation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IterativeSettings {
    pub min_quality_threshold: f32, // Minimum quality score to accept a song
    pub max_attempts_per_position: usize, // Maximum attempts before settling for best candidate
    #[serde(default)]
    pub debug_output: bool, // Enable debug output for artist repetition checking
}

impl Default for IterativeSettings {
    fn default() -> Self {
        Self {
            min_quality_threshold: 0.3,
            max_attempts_per_position: 10,
            debug_output: true,
        }
    }
}

impl Default for PlaylistConfig {
    fn default() -> Self {
        Self {
            name: "Default Playlist".to_string(),
            acceptable_genres: None,
            unacceptable_genres: None,
            bpm_thresholds: None,
            quality_weights: QualityWeights {
                artist_diversity: 0.30,
                bpm_transition_smoothness: 0.25,
                genre_coherence: 0.20,
                popularity_balance: 0.25,
                era_cohesion: 0.20,
            },
            transition_rules: TransitionRules {
                max_bpm_jump: 20,
                preferred_bpm_change: 0, // neutral by default
                avoid_artist_repeats_within: 3,
                // bpm_weight: 0.4,
                // artist_weight: 0.4,
                // genre_weight: 0.2,
            },
            preference_weights: PreferenceWeights {
                starred_boost: 100.0,
                play_count_weight: 20.0,
                recency_penalty_weight: 5.0,
                randomness_factor: 0.2,
                discovery_mode: false,
            },
            target_length: Some(20),
        }
    }
}

impl PlaylistConfig {
    /// Load playlist configurations directly from a JSON array file
    pub fn load_all_from_file(
        path: &str,
    ) -> Result<Vec<PlaylistConfig>, Box<dyn std::error::Error>> {
        let content = std::fs::read_to_string(path)?;
        let configs: Vec<PlaylistConfig> = serde_json::from_str(&content)?;
        Ok(configs)
    }
}
