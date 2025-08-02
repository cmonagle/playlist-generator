// Test examples for non-song filtering
// This file demonstrates what kinds of tracks get filtered out

use crate::models::Song;
use crate::playlist::filters::SongFilters;

#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_song(title: &str, duration: Option<u32>) -> Song {
        Song {
            id: "test".to_string(),
            title: title.to_string(),
            artist: "Test Artist".to_string(),
            album: "Test Album".to_string(),
            genre: None,
            genres: None,
            bpm: Some(120),
            duration,
            year: Some(2023),
            track: Some(1),
            play_count: None,
            disc_number: None,
            album_id: None,
            artist_id: None,
            played: None,
            starred: None,
            bit_rate: None,
            content_type: None,
        }
    }

    #[test]
    fn test_filters_interludes() {
        let interlude = create_test_song("Interlude", Some(60));
        let intro = create_test_song("Intro", Some(30));
        let outro = create_test_song("Outro (Extended)", Some(45));
        let actual_song = create_test_song("Beautiful Song", Some(180));
        
        assert!(!SongFilters::is_actual_song(&interlude));
        assert!(!SongFilters::is_actual_song(&intro));
        assert!(!SongFilters::is_actual_song(&outro));
        assert!(SongFilters::is_actual_song(&actual_song));
    }

    #[test]
    fn test_filters_sketches() {
        let sketch = create_test_song("Comedy Sketch #3", Some(90));
        let fragment = create_test_song("Song Fragment", Some(25));
        let actual_song = create_test_song("Real Song Title", Some(210));
        
        assert!(!SongFilters::is_actual_song(&sketch));
        assert!(!SongFilters::is_actual_song(&fragment));
        assert!(SongFilters::is_actual_song(&actual_song));
    }

    #[test]
    fn test_filters_by_duration() {
        let too_short = create_test_song("Short Track", Some(15)); // 15 seconds
        let too_long = create_test_song("Long Mix", Some(1200)); // 20 minutes
        let good_length = create_test_song("Normal Song", Some(240)); // 4 minutes
        
        assert!(!SongFilters::is_actual_song(&too_short));
        assert!(!SongFilters::is_actual_song(&too_long));
        assert!(SongFilters::is_actual_song(&good_length));
    }

    #[test]
    fn test_filters_spoken_content() {
        let interview = create_test_song("Artist Interview", Some(300));
        let monologue = create_test_song("Opening Monologue", Some(120));
        let spoken_word_song = create_test_song("Spoken Word Piece", Some(180)); // This contains "piece" so should be filtered
        let song_about_speech = create_test_song("Song About Speaking", Some(180)); // This should pass
        
        assert!(!SongFilters::is_actual_song(&interview));
        assert!(!SongFilters::is_actual_song(&monologue));
        assert!(!SongFilters::is_actual_song(&spoken_word_song)); // Contains "piece"
        assert!(SongFilters::is_actual_song(&song_about_speech)); // "speaking" vs "speech" should pass
    }

    #[test]
    fn test_handles_parenthetical_indicators() {
        let interlude_parens = create_test_song("Song Title (Interlude)", Some(60));
        let intro_parens = create_test_song("Album Opener (Intro)", Some(45));
        let instrumental_short = create_test_song("Brief (Instrumental)", Some(60)); // 1 minute, should be filtered
        let instrumental_long = create_test_song("Epic Journey (Instrumental)", Some(420)); // 7 minutes, should pass
        
        assert!(!SongFilters::is_actual_song(&interlude_parens));
        assert!(!SongFilters::is_actual_song(&intro_parens));
        assert!(!SongFilters::is_actual_song(&instrumental_short)); // Short instrumental filtered
        assert!(SongFilters::is_actual_song(&instrumental_long)); // Long instrumental passes
    }

    #[test]
    fn test_discovery_mode_scoring() {
        use crate::playlist::{PlaylistConfig, PreferenceWeights, QualityWeights, TransitionRules};
        use crate::playlist::scoring::PlaylistScoring;
        
        // Create discovery mode config
        let discovery_config = PlaylistConfig {
            name: "Discovery Test".to_string(),
            acceptable_genres: None,
            bpm_thresholds: None,
            quality_weights: QualityWeights {
                artist_diversity: 0.5,
                bpm_transition_smoothness: 0.5,
                genre_coherence: 0.5,
                popularity_balance: 0.5,
                era_cohesion: 0.5,
            },
            transition_rules: TransitionRules {
                max_bpm_jump: 30,
                preferred_bpm_change: 0,
                avoid_artist_repeats_within: 3,
            },
            preference_weights: PreferenceWeights {
                starred_boost: 0.0,
                play_count_weight: 1.0,
                recency_penalty_weight: 0.0,
                randomness_factor: 0.0,
                discovery_mode: true,
            },
            target_length: Some(20),
        };
        
        // Create normal mode config
        let normal_config = PlaylistConfig {
            preference_weights: PreferenceWeights {
                starred_boost: 0.0,
                play_count_weight: 1.0,
                recency_penalty_weight: 0.0,
                randomness_factor: 0.0,
                discovery_mode: false,
            },
            ..discovery_config.clone()
        };
        
        // Create songs with different play counts
        let unplayed_song = create_test_song("Unplayed Song", Some(180));
        let mut low_played_song = create_test_song("Low Played Song", Some(180));
        low_played_song.play_count = Some(2);
        let mut high_played_song = create_test_song("High Played Song", Some(180));
        high_played_song.play_count = Some(50);
        
        // Test discovery mode - lower play counts should score higher
        let unplayed_discovery_score = PlaylistScoring::calculate_preference_score(&unplayed_song, &discovery_config);
        let low_discovery_score = PlaylistScoring::calculate_preference_score(&low_played_song, &discovery_config);
        let high_discovery_score = PlaylistScoring::calculate_preference_score(&high_played_song, &discovery_config);
        
        assert!(unplayed_discovery_score > low_discovery_score, 
            "Unplayed song should score higher than low played song in discovery mode. Unplayed: {}, Low: {}", 
            unplayed_discovery_score, low_discovery_score);
        assert!(low_discovery_score > high_discovery_score, 
            "Low played song should score higher than high played song in discovery mode. Low: {}, High: {}", 
            low_discovery_score, high_discovery_score);
        
        // Test normal mode - higher play counts should score higher
        let unplayed_normal_score = PlaylistScoring::calculate_preference_score(&unplayed_song, &normal_config);
        let low_normal_score = PlaylistScoring::calculate_preference_score(&low_played_song, &normal_config);
        let high_normal_score = PlaylistScoring::calculate_preference_score(&high_played_song, &normal_config);
        
        assert!(high_normal_score > low_normal_score, 
            "High played song should score higher than low played song in normal mode. High: {}, Low: {}", 
            high_normal_score, low_normal_score);
        assert!(low_normal_score > unplayed_normal_score, 
            "Low played song should score higher than unplayed song in normal mode. Low: {}, Unplayed: {}", 
            low_normal_score, unplayed_normal_score);
    }
}
