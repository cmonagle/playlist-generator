// Test examples for non-song filtering
// This file demonstrates what kinds of tracks get filtered out

use crate::models::Song;
use crate::playlist::PlaylistGenerator;

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
        let generator = PlaylistGenerator::with_default_config();
        
        let interlude = create_test_song("Interlude", Some(60));
        let intro = create_test_song("Intro", Some(30));
        let outro = create_test_song("Outro (Extended)", Some(45));
        let actual_song = create_test_song("Beautiful Song", Some(180));
        
        assert!(!generator.is_actual_song(&interlude));
        assert!(!generator.is_actual_song(&intro));
        assert!(!generator.is_actual_song(&outro));
        assert!(generator.is_actual_song(&actual_song));
    }

    #[test]
    fn test_filters_sketches() {
        let generator = PlaylistGenerator::with_default_config();
        
        let sketch = create_test_song("Comedy Sketch #3", Some(90));
        let fragment = create_test_song("Song Fragment", Some(25));
        let actual_song = create_test_song("Real Song Title", Some(210));
        
        assert!(!generator.is_actual_song(&sketch));
        assert!(!generator.is_actual_song(&fragment));
        assert!(generator.is_actual_song(&actual_song));
    }

    #[test]
    fn test_filters_by_duration() {
        let generator = PlaylistGenerator::with_default_config();
        
        let too_short = create_test_song("Short Track", Some(15)); // 15 seconds
        let too_long = create_test_song("Long Mix", Some(1200)); // 20 minutes
        let good_length = create_test_song("Normal Song", Some(240)); // 4 minutes
        
        assert!(!generator.is_actual_song(&too_short));
        assert!(!generator.is_actual_song(&too_long));
        assert!(generator.is_actual_song(&good_length));
    }

    #[test]
    fn test_filters_spoken_content() {
        let generator = PlaylistGenerator::with_default_config();
        
        let interview = create_test_song("Artist Interview", Some(300));
        let monologue = create_test_song("Opening Monologue", Some(120));
        let spoken_word_song = create_test_song("Spoken Word Piece", Some(180)); // This contains "piece" so should be filtered
        let song_about_speech = create_test_song("Song About Speaking", Some(180)); // This should pass
        
        assert!(!generator.is_actual_song(&interview));
        assert!(!generator.is_actual_song(&monologue));
        assert!(!generator.is_actual_song(&spoken_word_song)); // Contains "piece"
        assert!(generator.is_actual_song(&song_about_speech)); // "speaking" vs "speech" should pass
    }

    #[test]
    fn test_handles_parenthetical_indicators() {
        let generator = PlaylistGenerator::with_default_config();
        
        let interlude_parens = create_test_song("Song Title (Interlude)", Some(60));
        let intro_parens = create_test_song("Album Opener (Intro)", Some(45));
        let instrumental_short = create_test_song("Brief (Instrumental)", Some(60)); // 1 minute, should be filtered
        let instrumental_long = create_test_song("Epic Journey (Instrumental)", Some(420)); // 7 minutes, should pass
        
        assert!(!generator.is_actual_song(&interlude_parens));
        assert!(!generator.is_actual_song(&intro_parens));
        assert!(!generator.is_actual_song(&instrumental_short)); // Short instrumental filtered
        assert!(generator.is_actual_song(&instrumental_long)); // Long instrumental passes
    }
}
