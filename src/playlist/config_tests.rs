#[cfg(test)]
mod tests {
    use super::super::*;
    use crate::models::{Song, Genre};
    use approx::assert_relative_eq;

    // Mock song creation helper
    fn create_mock_song(
        id: &str,
        title: &str,
        artist: &str,
        album: &str,
        genres: Vec<&str>,
        bpm: Option<u32>,
        year: Option<u32>,
        duration: Option<u32>,
        play_count: Option<u32>,
        starred: bool,
        played: Option<&str>,
    ) -> Song {
        Song {
            id: id.to_string(),
            title: title.to_string(),
            artist: artist.to_string(),
            album: album.to_string(),
            genre: genres.first().map(|g| g.to_string()),
            genres: if genres.is_empty() { 
                None 
            } else { 
                Some(genres.iter().map(|g| Genre { name: g.to_string() }).collect())
            },
            bpm,
            duration,
            year,
            track: Some(1),
            play_count,
            disc_number: None,
            album_id: None,
            artist_id: None,
            played: played.map(|p| p.to_string()),
            starred: if starred { Some("starred".to_string()) } else { None },
            bit_rate: Some(320),
            content_type: Some("audio/mpeg".to_string()),
        }
    }

    fn create_test_config() -> PlaylistConfig {
        PlaylistConfig {
            name: "Test Playlist".to_string(),
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
                max_bpm_jump: 20,
                preferred_bpm_change: 0,
                avoid_artist_repeats_within: 5,
            },
            preference_weights: PreferenceWeights {
                starred_boost: 50.0,
                play_count_weight: 10.0,
                recency_penalty_weight: 5.0,
                randomness_factor: 0.1,
                discovery_mode: false,
            },
            target_length: Some(20),
        }
    }

    fn create_diverse_song_collection() -> Vec<Song> {
        vec![
            // Artist diversity test songs
            create_mock_song("1", "Song 1", "Artist A", "Album 1", vec!["Rock"], Some(120), Some(2020), Some(200), Some(5), false, None),
            create_mock_song("2", "Song 2", "Artist A", "Album 2", vec!["Rock"], Some(125), Some(2021), Some(210), Some(3), false, None),
            create_mock_song("3", "Song 3", "Artist B", "Album 3", vec!["Pop"], Some(130), Some(2019), Some(180), Some(7), true, None),
            create_mock_song("4", "Song 4", "Artist C", "Album 4", vec!["Jazz"], Some(90), Some(2022), Some(240), Some(1), false, None),
            
            // BPM transition test songs
            create_mock_song("5", "Slow Song", "Artist D", "Album 5", vec!["Ambient"], Some(60), Some(2020), Some(300), Some(2), false, None),
            create_mock_song("6", "Medium Song", "Artist E", "Album 6", vec!["Indie"], Some(120), Some(2021), Some(220), Some(4), false, None),
            create_mock_song("7", "Fast Song", "Artist F", "Album 7", vec!["Electronic"], Some(180), Some(2019), Some(190), Some(6), false, None),
            
            // Genre coherence test songs
            create_mock_song("8", "Rock Song 1", "Artist G", "Album 8", vec!["Rock"], Some(140), Some(2020), Some(200), Some(3), false, None),
            create_mock_song("9", "Rock Song 2", "Artist H", "Album 9", vec!["Rock"], Some(135), Some(2021), Some(195), Some(4), false, None),
            create_mock_song("10", "Classical", "Artist I", "Album 10", vec!["Classical"], Some(80), Some(1990), Some(400), Some(1), false, None),
            
            // Era cohesion test songs
            create_mock_song("11", "80s Song", "Artist J", "Album 11", vec!["Synthpop"], Some(125), Some(1985), Some(210), Some(2), false, None),
            create_mock_song("12", "90s Song", "Artist K", "Album 12", vec!["Grunge"], Some(130), Some(1995), Some(220), Some(3), false, None),
            create_mock_song("13", "2020s Song", "Artist L", "Album 13", vec!["Hyperpop"], Some(160), Some(2023), Some(150), Some(8), false, None),
            
            // Popularity balance test songs (varying play counts)
            create_mock_song("14", "Hit Song", "Artist M", "Album 14", vec!["Pop"], Some(128), Some(2022), Some(200), Some(50), true, None),
            create_mock_song("15", "Deep Cut", "Artist N", "Album 15", vec!["Indie"], Some(110), Some(2021), Some(250), Some(1), false, None),
            create_mock_song("16", "Moderate Hit", "Artist O", "Album 16", vec!["Rock"], Some(140), Some(2020), Some(230), Some(10), false, None),
            
            // Recently played songs for recency test
            create_mock_song("17", "Recent Song", "Artist P", "Album 17", vec!["Pop"], Some(120), Some(2023), Some(180), Some(5), false, Some("2025-08-01T12:00:00Z")),
            create_mock_song("18", "Old Song", "Artist Q", "Album 18", vec!["Rock"], Some(125), Some(2022), Some(200), Some(3), false, Some("2025-06-01T12:00:00Z")),
        ]
    }

    #[test]
    fn test_bpm_threshold_filtering() {
        let mut config = create_test_config();
        config.bpm_thresholds = Some(BpmThresholds {
            min_bpm: 100,
            max_bpm: 150,
        });
        
        let generator = PlaylistGenerator::new(config);
        let songs = create_diverse_song_collection();
        
        // Test individual BPM filtering
        let slow_song = &songs[4]; // 60 BPM - should be filtered out
        let medium_song = &songs[5]; // 120 BPM - should pass
        let fast_song = &songs[6]; // 180 BPM - should be filtered out
        
        assert!(!generator.matches_bpm_thresholds(slow_song));
        assert!(generator.matches_bpm_thresholds(medium_song));
        assert!(!generator.matches_bpm_thresholds(fast_song));
    }

    #[test]
    fn test_bpm_threshold_none_accepts_all() {
        let config = create_test_config(); // No BPM thresholds
        let generator = PlaylistGenerator::new(config);
        let songs = create_diverse_song_collection();
        
        // All songs should pass when no BPM filter is set
        for song in &songs {
            assert!(generator.matches_bpm_thresholds(song));
        }
    }

    #[test]
    fn test_genre_filtering() {
        let mut config = create_test_config();
        config.acceptable_genres = Some(vec!["Rock".to_string(), "Pop".to_string()]);
        
        let generator = PlaylistGenerator::new(config);
        let songs = create_diverse_song_collection();
        
        let rock_song = &songs[0]; // Rock genre
        let pop_song = &songs[2]; // Pop genre  
        let jazz_song = &songs[3]; // Jazz genre - should be filtered out
        
        assert!(generator.matches_acceptable_genres(rock_song));
        assert!(generator.matches_acceptable_genres(pop_song));
        assert!(!generator.matches_acceptable_genres(jazz_song));
    }

    #[test]
    fn test_genre_filtering_none_accepts_all() {
        let config = create_test_config(); // No genre filter
        let generator = PlaylistGenerator::new(config);
        let songs = create_diverse_song_collection();
        
        // All songs should pass when no genre filter is set
        for song in &songs {
            assert!(generator.matches_acceptable_genres(song));
        }
    }

    #[test]
    fn test_unacceptable_genres_filtering() {
        let mut config = create_test_config();
        config.unacceptable_genres = Some(vec!["Metal".to_string(), "Electronic".to_string()]);
        
        use crate::playlist::filters::SongFilters;
        let songs = create_diverse_song_collection();
        
        let rock_song = &songs[0]; // Rock genre - should pass
        let metal_song = create_mock_song("metal1", "Metal Song", "Metal Artist", "Album", 
                                        vec!["Metal"], Some(140), Some(2020), Some(200), Some(5), false, None);
        let electronic_song = create_mock_song("elec1", "Electronic Song", "Electronic Artist", "Album", 
                                             vec!["Electronic"], Some(128), Some(2020), Some(200), Some(5), false, None);
        
        assert!(SongFilters::does_not_match_unacceptable_genres(rock_song, &config));
        assert!(!SongFilters::does_not_match_unacceptable_genres(&metal_song, &config));
        assert!(!SongFilters::does_not_match_unacceptable_genres(&electronic_song, &config));
    }

    #[test]
    fn test_unacceptable_genres_none_accepts_all() {
        let config = create_test_config(); // No unacceptable genre filter
        use crate::playlist::filters::SongFilters;
        let songs = create_diverse_song_collection();
        
        // All songs should pass when no unacceptable genre filter is set
        for song in &songs {
            assert!(SongFilters::does_not_match_unacceptable_genres(song, &config));
        }
    }

    #[test]
    fn test_combined_genre_filtering() {
        let mut config = create_test_config();
        config.acceptable_genres = Some(vec!["Rock".to_string(), "Pop".to_string(), "Metal".to_string()]);
        config.unacceptable_genres = Some(vec!["Metal".to_string()]);
        
        use crate::playlist::filters::SongFilters;
        
        let rock_song = create_mock_song("rock1", "Rock Song", "Rock Artist", "Album", 
                                       vec!["Rock"], Some(120), Some(2020), Some(200), Some(5), false, None);
        let pop_song = create_mock_song("pop1", "Pop Song", "Pop Artist", "Album", 
                                      vec!["Pop"], Some(110), Some(2020), Some(200), Some(5), false, None);
        let metal_song = create_mock_song("metal1", "Metal Song", "Metal Artist", "Album", 
                                        vec!["Metal"], Some(140), Some(2020), Some(200), Some(5), false, None);
        let jazz_song = create_mock_song("jazz1", "Jazz Song", "Jazz Artist", "Album", 
                                       vec!["Jazz"], Some(100), Some(2020), Some(200), Some(5), false, None);
        
        // Rock should pass (acceptable and not unacceptable)
        assert!(SongFilters::should_include_song(&rock_song, &config));
        
        // Pop should pass (acceptable and not unacceptable) 
        assert!(SongFilters::should_include_song(&pop_song, &config));
        
        // Metal should fail (acceptable but also unacceptable - unacceptable takes precedence)
        assert!(!SongFilters::should_include_song(&metal_song, &config));
        
        // Jazz should fail (not acceptable)
        assert!(!SongFilters::should_include_song(&jazz_song, &config));
    }

    #[test]
    fn test_artist_diversity_calculation() {
        let config = create_test_config();
        let generator = PlaylistGenerator::new(config);
        
        // Test high diversity (all different artists)
        let diverse_songs = vec![
            create_mock_song("1", "Song 1", "Artist A", "Album", vec!["Rock"], Some(120), Some(2020), Some(200), Some(5), false, None),
            create_mock_song("2", "Song 2", "Artist B", "Album", vec!["Rock"], Some(125), Some(2020), Some(200), Some(5), false, None),
            create_mock_song("3", "Song 3", "Artist C", "Album", vec!["Rock"], Some(130), Some(2020), Some(200), Some(5), false, None),
        ];
        
        let diverse_metadata = generator.calculate_metadata(&diverse_songs);
        assert_eq!(diverse_metadata.artist_count, 3); // 3 unique artists out of 3 songs = 100% diversity
        
        // Test low diversity (same artist)
        let similar_songs = vec![
            create_mock_song("1", "Song 1", "Artist A", "Album", vec!["Rock"], Some(120), Some(2020), Some(200), Some(5), false, None),
            create_mock_song("2", "Song 2", "Artist A", "Album", vec!["Rock"], Some(125), Some(2020), Some(200), Some(5), false, None),
            create_mock_song("3", "Song 3", "Artist A", "Album", vec!["Rock"], Some(130), Some(2020), Some(200), Some(5), false, None),
        ];
        
        let similar_metadata = generator.calculate_metadata(&similar_songs);
        assert_eq!(similar_metadata.artist_count, 1); // 1 unique artist out of 3 songs = 33% diversity
    }

    #[test]
    fn test_bpm_transition_smoothness() {
        let config = create_test_config();
        let generator = PlaylistGenerator::new(config);
        
        // Test smooth transitions (small BPM differences)
        let smooth_songs = vec![
            create_mock_song("1", "Song 1", "Artist A", "Album", vec!["Rock"], Some(120), Some(2020), Some(200), Some(5), false, None),
            create_mock_song("2", "Song 2", "Artist B", "Album", vec!["Rock"], Some(125), Some(2020), Some(200), Some(5), false, None),
            create_mock_song("3", "Song 3", "Artist C", "Album", vec!["Rock"], Some(130), Some(2020), Some(200), Some(5), false, None),
        ];
        
        let smooth_score = generator.calculate_bpm_transition_score(&smooth_songs);
        
        // Test jarring transitions (large BPM differences)
        let jarring_songs = vec![
            create_mock_song("1", "Song 1", "Artist A", "Album", vec!["Rock"], Some(60), Some(2020), Some(200), Some(5), false, None),
            create_mock_song("2", "Song 2", "Artist B", "Album", vec!["Rock"], Some(180), Some(2020), Some(200), Some(5), false, None),
            create_mock_song("3", "Song 3", "Artist C", "Album", vec!["Rock"], Some(80), Some(2020), Some(200), Some(5), false, None),
        ];
        
        let jarring_score = generator.calculate_bpm_transition_score(&jarring_songs);
        
        // Smooth transitions should score higher than jarring ones
        assert!(smooth_score > jarring_score);
    }

    #[test]
    fn test_genre_coherence_calculation() {
        let config = create_test_config();
        let generator = PlaylistGenerator::new(config);
        
        // Test high coherence (same genre)
        let coherent_songs = vec![
            create_mock_song("1", "Song 1", "Artist A", "Album", vec!["Rock"], Some(120), Some(2020), Some(200), Some(5), false, None),
            create_mock_song("2", "Song 2", "Artist B", "Album", vec!["Rock"], Some(125), Some(2020), Some(200), Some(5), false, None),
            create_mock_song("3", "Song 3", "Artist C", "Album", vec!["Rock"], Some(130), Some(2020), Some(200), Some(5), false, None),
        ];
        
        let coherent_metadata = generator.calculate_metadata(&coherent_songs);
        let coherent_score = generator.calculate_genre_coherence_score(&coherent_metadata.genre_distribution, coherent_songs.len());
        
        // Test low coherence (different genres)
        let diverse_songs = vec![
            create_mock_song("1", "Song 1", "Artist A", "Album", vec!["Rock"], Some(120), Some(2020), Some(200), Some(5), false, None),
            create_mock_song("2", "Song 2", "Artist B", "Album", vec!["Jazz"], Some(125), Some(2020), Some(200), Some(5), false, None),
            create_mock_song("3", "Song 3", "Artist C", "Album", vec!["Classical"], Some(130), Some(2020), Some(200), Some(5), false, None),
        ];
        
        let diverse_metadata = generator.calculate_metadata(&diverse_songs);
        let diverse_score = generator.calculate_genre_coherence_score(&diverse_metadata.genre_distribution, diverse_songs.len());
        
        // Coherent genres should score higher than diverse genres
        assert!(coherent_score > diverse_score);
    }

    #[test]
    fn test_era_cohesion_calculation() {
        let config = create_test_config();
        let generator = PlaylistGenerator::new(config);
        
        // Test high cohesion (same era)
        let cohesive_songs = vec![
            create_mock_song("1", "Song 1", "Artist A", "Album", vec!["Rock"], Some(120), Some(2020), Some(200), Some(5), false, None),
            create_mock_song("2", "Song 2", "Artist B", "Album", vec!["Rock"], Some(125), Some(2021), Some(200), Some(5), false, None),
            create_mock_song("3", "Song 3", "Artist C", "Album", vec!["Rock"], Some(130), Some(2022), Some(200), Some(5), false, None),
        ];
        
        let cohesive_metadata = generator.calculate_metadata(&cohesive_songs);
        let cohesive_score = generator.calculate_era_cohesion_score(&cohesive_metadata.era_span, &cohesive_songs);
        
        // Test low cohesion (different eras)
        let spanning_songs = vec![
            create_mock_song("1", "Song 1", "Artist A", "Album", vec!["Rock"], Some(120), Some(1980), Some(200), Some(5), false, None),
            create_mock_song("2", "Song 2", "Artist B", "Album", vec!["Rock"], Some(125), Some(2000), Some(200), Some(5), false, None),
            create_mock_song("3", "Song 3", "Artist C", "Album", vec!["Rock"], Some(130), Some(2023), Some(200), Some(5), false, None),
        ];
        
        let spanning_metadata = generator.calculate_metadata(&spanning_songs);
        let spanning_score = generator.calculate_era_cohesion_score(&spanning_metadata.era_span, &spanning_songs);
        
        // Cohesive eras should score higher than spanning eras
        assert!(cohesive_score > spanning_score);
    }

    #[test]
    fn test_popularity_balance_calculation() {
        let config = create_test_config();
        let generator = PlaylistGenerator::new(config);
        
        // Test balanced popularity (good mix of play counts)
        let balanced_songs = vec![
            create_mock_song("1", "Song 1", "Artist A", "Album", vec!["Rock"], Some(120), Some(2020), Some(200), Some(1), false, None),  // Low play count
            create_mock_song("2", "Song 2", "Artist B", "Album", vec!["Rock"], Some(125), Some(2020), Some(200), Some(10), false, None), // Medium play count
            create_mock_song("3", "Song 3", "Artist C", "Album", vec!["Rock"], Some(130), Some(2020), Some(200), Some(5), false, None),  // Medium play count
        ];
        
        let balanced_score = generator.calculate_popularity_balance_score(&balanced_songs);
        
        // Test unbalanced popularity (extreme differences)
        let unbalanced_songs = vec![
            create_mock_song("1", "Song 1", "Artist A", "Album", vec!["Rock"], Some(120), Some(2020), Some(200), Some(1), false, None),   // Very low
            create_mock_song("2", "Song 2", "Artist B", "Album", vec!["Rock"], Some(125), Some(2020), Some(200), Some(100), false, None), // Very high
            create_mock_song("3", "Song 3", "Artist C", "Album", vec!["Rock"], Some(130), Some(2020), Some(200), Some(1), false, None),   // Very low
        ];
        
        let unbalanced_score = generator.calculate_popularity_balance_score(&unbalanced_songs);
        
        // Balanced popularity should score higher than unbalanced
        assert!(balanced_score > unbalanced_score);
    }

    #[test]
    fn test_starred_boost_preference() {
        let config = create_test_config();
        let expected_difference = config.preference_weights.starred_boost;
        let generator = PlaylistGenerator::new(config);
        
        let starred_song = create_mock_song("1", "Starred Song", "Artist A", "Album", vec!["Rock"], Some(120), Some(2020), Some(200), Some(5), true, None);
        let unstarred_song = create_mock_song("2", "Regular Song", "Artist B", "Album", vec!["Rock"], Some(120), Some(2020), Some(200), Some(5), false, None);
        
        let starred_score = generator.calculate_preference_score(&starred_song);
        let unstarred_score = generator.calculate_preference_score(&unstarred_song);
        
        // Starred song should have higher preference score
        assert!(starred_score > unstarred_score);
        
        // The difference should be approximately the starred_boost value
        assert_relative_eq!(starred_score - unstarred_score, expected_difference, epsilon = 1.0);
    }

    #[test]
    fn test_discovery_mode_preference() {
        let mut config = create_test_config();
        config.preference_weights.discovery_mode = true;
        
        let generator = PlaylistGenerator::new(config);
        
        let unplayed_song = create_mock_song("1", "Unplayed Song", "Artist A", "Album", vec!["Rock"], Some(120), Some(2020), Some(200), Some(0), false, None);
        let popular_song = create_mock_song("2", "Popular Song", "Artist B", "Album", vec!["Rock"], Some(120), Some(2020), Some(200), Some(50), false, None);
        
        let unplayed_score = generator.calculate_preference_score(&unplayed_song);
        let popular_score = generator.calculate_preference_score(&popular_song);
        
        // In discovery mode, unplayed songs should score higher than popular songs
        assert!(unplayed_score > popular_score);
    }

    #[test]
    fn test_recency_penalty() {
        let config = create_test_config();
        let generator = PlaylistGenerator::new(config);
        
        // Recent song (should be penalized)
        let recent_song = create_mock_song("1", "Recent Song", "Artist A", "Album", vec!["Rock"], Some(120), Some(2020), Some(200), Some(5), false, Some("2025-08-01T12:00:00Z"));
        
        // Old song (should not be penalized)
        let old_song = create_mock_song("2", "Old Song", "Artist B", "Album", vec!["Rock"], Some(120), Some(2020), Some(200), Some(5), false, Some("2025-01-01T12:00:00Z"));
        
        let recent_score = generator.calculate_preference_score(&recent_song);
        let old_score = generator.calculate_preference_score(&old_song);
        
        // Old song should score higher (less penalty) than recent song
        assert!(old_score > recent_score);
    }

    #[test]
    fn test_quality_weights_impact() {
        // Test high artist diversity preference
        let mut high_diversity_config = create_test_config();
        high_diversity_config.quality_weights.artist_diversity = 1.0;
        high_diversity_config.quality_weights.genre_coherence = 0.0;
        high_diversity_config.quality_weights.bpm_transition_smoothness = 0.0;
        high_diversity_config.quality_weights.popularity_balance = 0.0;
        high_diversity_config.quality_weights.era_cohesion = 0.0;
        
        let high_diversity_generator = PlaylistGenerator::new(high_diversity_config);
        
        // Create songs with high artist diversity but poor other metrics
        let diverse_artists_songs = vec![
            create_mock_song("1", "Song 1", "Artist A", "Album", vec!["Rock"], Some(60), Some(1980), Some(200), Some(1), false, None),
            create_mock_song("2", "Song 2", "Artist B", "Album", vec!["Jazz"], Some(180), Some(2020), Some(200), Some(100), false, None),
            create_mock_song("3", "Song 3", "Artist C", "Album", vec!["Classical"], Some(80), Some(1990), Some(200), Some(50), false, None),
        ];
        
        let diverse_metadata = high_diversity_generator.calculate_metadata(&diverse_artists_songs);
        let diverse_quality = high_diversity_generator.calculate_quality_score(&diverse_artists_songs, &diverse_metadata);
        
        // Test low artist diversity preference
        let mut low_diversity_config = create_test_config();
        low_diversity_config.quality_weights.artist_diversity = 0.0;
        low_diversity_config.quality_weights.genre_coherence = 1.0;
        low_diversity_config.quality_weights.bpm_transition_smoothness = 1.0;
        low_diversity_config.quality_weights.popularity_balance = 1.0;
        low_diversity_config.quality_weights.era_cohesion = 1.0;
        
        let low_diversity_generator = PlaylistGenerator::new(low_diversity_config);
        
        // Create songs with low artist diversity but good other metrics
        let same_artist_songs = vec![
            create_mock_song("1", "Song 1", "Artist A", "Album", vec!["Rock"], Some(120), Some(2020), Some(200), Some(5), false, None),
            create_mock_song("2", "Song 2", "Artist A", "Album", vec!["Rock"], Some(125), Some(2020), Some(200), Some(6), false, None),
            create_mock_song("3", "Song 3", "Artist A", "Album", vec!["Rock"], Some(130), Some(2020), Some(200), Some(4), false, None),
        ];
        
        let same_metadata = low_diversity_generator.calculate_metadata(&same_artist_songs);
        let same_quality = low_diversity_generator.calculate_quality_score(&same_artist_songs, &same_metadata);
        
        // The results should reflect the different preferences
        // This test ensures that quality weights actually impact the final score
        assert!(diverse_quality != same_quality, "Quality weights should produce different scores for different configurations");
    }

    #[test]
    fn test_full_playlist_generation_respects_config() {
        let mut config = create_test_config();
        config.acceptable_genres = Some(vec!["Rock".to_string()]);
        config.bpm_thresholds = Some(BpmThresholds { min_bpm: 100, max_bpm: 150 });
        config.target_length = Some(3);
        
        let generator = PlaylistGenerator::new(config);
        let songs = create_diverse_song_collection();
        
        let playlist = generator.generate_playlist(songs, Some("Test Playlist".to_string()), Some(3));
        
        // Check that playlist respects target length
        assert_eq!(playlist.songs.len(), 3);
        
        // Check that all songs in playlist match genre filter
        for song in &playlist.songs {
            assert!(song.get_all_genres().iter().any(|g| g.to_lowercase() == "rock"));
        }
        
        // Check that all songs in playlist match BPM filter
        for song in &playlist.songs {
            if let Some(bpm) = song.bpm {
                assert!(bpm >= 100 && bpm <= 150);
            }
        }
        
        // Check that playlist has metadata
        assert!(playlist.metadata.total_duration > 0);
        assert!(playlist.metadata.artist_count > 0);
    }

    #[test]
    fn test_empty_songs_handled_gracefully() {
        let config = create_test_config();
        let generator = PlaylistGenerator::new(config);
        
        let playlist = generator.generate_playlist(vec![], Some("Empty Playlist".to_string()), Some(10));
        
        assert_eq!(playlist.songs.len(), 0);
        assert_eq!(playlist.metadata.total_duration, 0);
        assert_eq!(playlist.metadata.artist_count, 0);
        assert_eq!(playlist.quality_score, 0.0);
    }

    #[test]
    fn test_preference_weights_boundaries() {
        let mut config = create_test_config();
        
        // Test extreme preference weights
        config.preference_weights.starred_boost = 1000.0;
        config.preference_weights.randomness_factor = 1.0;
        
        let generator = PlaylistGenerator::new(config);
        
        let starred_song = create_mock_song("1", "Starred", "Artist", "Album", vec!["Rock"], Some(120), Some(2020), Some(200), Some(1), true, None);
        let unstarred_song = create_mock_song("2", "Unstarred", "Artist", "Album", vec!["Rock"], Some(120), Some(2020), Some(200), Some(10), false, None);
        
        let starred_score = generator.calculate_preference_score(&starred_song);
        let unstarred_score = generator.calculate_preference_score(&unstarred_song);
        
        // Extreme starred boost should completely dominate other factors
        assert!(starred_score > unstarred_score + 900.0);
    }

    #[test]
    fn test_transition_rules_impact() {
        let mut config = create_test_config();
        config.transition_rules.max_bpm_jump = 5; // Very restrictive
        config.transition_rules.avoid_artist_repeats_within = 10; // Avoid repeats
        
        let generator = PlaylistGenerator::new(config);
        
        // This test mainly ensures the config structure is valid
        // The actual transition logic would need to be implemented in the ordering phase
        let songs = vec![
            create_mock_song("1", "Song 1", "Artist A", "Album", vec!["Rock"], Some(120), Some(2020), Some(200), Some(5), false, None),
            create_mock_song("2", "Song 2", "Artist A", "Album", vec!["Rock"], Some(122), Some(2020), Some(200), Some(5), false, None), // Small BPM jump
        ];
        
        let playlist = generator.generate_playlist(songs, Some("Transition Test".to_string()), Some(2));
        assert_eq!(playlist.songs.len(), 2);
    }

    #[test]
    fn test_config_validation() {
        // Test that all config fields can be set to valid values
        let config = PlaylistConfig {
            name: "Test".to_string(),
            acceptable_genres: Some(vec!["Rock".to_string(), "Pop".to_string()]),
            bpm_thresholds: Some(BpmThresholds { min_bpm: 60, max_bpm: 180 }),
            quality_weights: QualityWeights {
                artist_diversity: 1.0,
                bpm_transition_smoothness: 0.0,
                genre_coherence: 0.5,
                popularity_balance: 0.3,
                era_cohesion: 0.8,
            },
            transition_rules: TransitionRules {
                max_bpm_jump: 50,
                preferred_bpm_change: -5,
                avoid_artist_repeats_within: 3,
            },
            preference_weights: PreferenceWeights {
                starred_boost: 100.0,
                play_count_weight: 25.0,
                recency_penalty_weight: 10.0,
                randomness_factor: 0.5,
                discovery_mode: true,
            },
            target_length: Some(15),
        };
        
        let generator = PlaylistGenerator::new(config);
        
        // Just ensure it can be created without panicking
        let empty_playlist = generator.generate_playlist(vec![], Some("Empty".to_string()), Some(5));
        assert_eq!(empty_playlist.songs.len(), 0);
    }
}
