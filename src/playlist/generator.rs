use super::filters::SongFilters;
use super::scoring::PlaylistScoring;
use super::{Playlist, PlaylistConfig, PlaylistSong};
use crate::models::Song;
use crate::playlist::utils::PlaylistNaming;

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
            score_b
                .partial_cmp(&score_a)
                .unwrap_or(std::cmp::Ordering::Equal)
        });

        // Experimental: Use iterative quality-aware generation instead of simple ordering
        let ordered_songs = self.generate_playlist_iteratively(filtered_songs, target_length);

        // Extract songs for metadata calculation
        let songs_for_metadata: Vec<Song> =
            ordered_songs.iter().map(|ps| ps.song.clone()).collect();
        let metadata = PlaylistScoring::calculate_metadata(&songs_for_metadata);
        let quality_score =
            PlaylistScoring::calculate_quality_score(&songs_for_metadata, &metadata, &self.config);

        Playlist {
            songs: ordered_songs,
            name: PlaylistNaming::generate_playlist_name(
                playlist_name.unwrap_or("Daylist".to_string()),
                &metadata,
            ),
            base_name_pattern: self.config.name.clone(),
            metadata,
            quality_score,
        }
    }

    /// Experimental: Generate playlist iteratively, evaluating quality at each step
    fn generate_playlist_iteratively(
        &self,
        candidate_songs: Vec<Song>,
        target_length: usize,
    ) -> Vec<PlaylistSong> {
        let mut playlist: Vec<PlaylistSong> = Vec::new();
        let mut remaining_songs = candidate_songs;

        while playlist.len() < target_length && !remaining_songs.is_empty() {
            let mut best_candidate_index: Option<usize> = None;
            let mut best_quality_score = 0.0;
            let mut best_transition_score = 0.0;

            // Calculate current playlist quality for comparison
            let current_playlist_songs: Vec<Song> =
                playlist.iter().map(|ps| ps.song.clone()).collect();
            let current_quality = if current_playlist_songs.is_empty() {
                0.0
            } else {
                let current_metadata = PlaylistScoring::calculate_metadata(&current_playlist_songs);
                PlaylistScoring::calculate_quality_score(
                    &current_playlist_songs,
                    &current_metadata,
                    &self.config,
                )
            };

            // Try candidates in order of preference score (already sorted)
            for (i, candidate) in remaining_songs.iter().enumerate() {
                if i >= 10 {
                    break; // Could make this smarter - maybe continue if we haven't found ANY viable candidate
                }

                // Hard constraint: Skip candidates that would violate artist repetition rules
                let artist_score = self.calculate_artist_repetition_score(&current_playlist_songs, candidate);
                if artist_score < 0.3 {
                    println!("      SKIPPING '{}' by {} due to artist repetition constraint (score: {})", 
                             candidate.title, candidate.artist, artist_score);
                    continue;
                }

                // Calculate transition score for this candidate against the working playlist
                let transition_score =
                    self.calculate_playlist_transition_score(&current_playlist_songs, candidate);

                // Create a test playlist with this candidate added at the end (simpler approach)
                let mut test_playlist = current_playlist_songs.clone();
                test_playlist.push(candidate.clone());

                // Calculate quality of the test playlist
                let test_metadata = PlaylistScoring::calculate_metadata(&test_playlist);
                let test_quality = PlaylistScoring::calculate_quality_score(
                    &test_playlist,
                    &test_metadata,
                    &self.config,
                );

                // Combine quality score with transition score
                // Use configurable quality vs transition weighting (70/30 split for now)
                let combined_score = test_quality * 0.7 + transition_score * 0.3;

                // Apply quality criteria based on configurable thresholds
                let quality_improvement = test_quality - current_quality;
                let acceptable_quality = test_quality >= 0.5;

                // Consider different strategies for different playlist types
                let should_accept = if current_playlist_songs.is_empty() {
                    // Always accept the first song if it meets minimum threshold
                    acceptable_quality
                } else {
                    // For subsequent songs, require improvement or at least no significant degradation
                    acceptable_quality
                        && (quality_improvement >= -0.1 || combined_score > best_quality_score)
                };

                if should_accept && combined_score > best_quality_score {
                    best_candidate_index = Some(i);
                    best_quality_score = combined_score;
                    best_transition_score = transition_score;
                }
            }

            // Add the best candidate we found, or fallback to first available
            match best_candidate_index {
                Some(index) => {
                    let chosen_song = remaining_songs.remove(index);
                    let quality_contribution = best_quality_score - current_quality;
                    // Since we're building iteratively, just append to the end
                    playlist.push(PlaylistSong::with_metadata(
                        chosen_song,
                        best_transition_score,
                        quality_contribution,
                        "best candidate".to_string(),
                    ));
                }
                None => {
                    // Implement fallback strategy - take the highest-scored remaining song
                    if !remaining_songs.is_empty() {
                        // Fallback: take the highest-scored remaining song
                        let fallback_song = remaining_songs.remove(0);
                        playlist.push(PlaylistSong::with_metadata(
                            fallback_song,
                            0.0, // No transition score calculated for fallback
                            0.0, // No quality contribution calculated for fallback
                            "fallback".to_string(),
                        ));
                    }
                }
            }
        }

        playlist
    }

    /// Calculate how well a candidate song would fit with the current working playlist
    fn calculate_playlist_transition_score(
        &self,
        current_playlist: &[Song],
        candidate: &Song,
    ) -> f32 {
        if current_playlist.is_empty() {
            return 0.5; // Neutral score for first song
        }

        let mut total_score = 0.0_f32;

        // 1. BPM transition - only check against the last song
        if let Some(last_song) = current_playlist.last() {
            let bpm_score = self.calculate_bpm_transition_score(last_song, candidate);
            total_score += bpm_score * 0.25;
        }

        // 2. Artist repetition - check against recent songs based on avoid_artist_repeats_within
        // Give this much higher weight since it's a hard constraint we want to enforce
        let artist_score = self.calculate_artist_repetition_score(current_playlist, candidate);
        total_score += artist_score * 0.6; // Increased from 0.33 to 0.6

        // 3. Genre compatibility - check against the overall playlist genre distribution
        let genre_score = self.calculate_genre_compatibility_score(current_playlist, candidate);
        total_score += genre_score * 0.15; // Reduced from 0.33 to 0.15

        // Return the weighted sum (should be between 0.0 and 1.0 if weights sum to 1.0)
        total_score
    }

    /// Calculate BPM transition score between two songs
    fn calculate_bpm_transition_score(&self, song_a: &Song, song_b: &Song) -> f32 {
        if let (Some(bpm_a), Some(bpm_b)) = (song_a.bpm, song_b.bpm) {
            let bpm_diff = (bpm_a as i32 - bpm_b as i32).unsigned_abs();

            // Use transition rules from config
            let max_jump = self.config.transition_rules.max_bpm_jump;

            if bpm_diff <= 5 {
                1.0 // Very smooth transition
            } else if bpm_diff <= 15 {
                0.8 // Good transition
            } else if bpm_diff <= max_jump {
                0.5 // Acceptable transition
            } else {
                0.2 // Poor transition but not completely unacceptable
            }
        } else {
            0.5 // Neutral when BPM data is missing
        }
    }

    /// Calculate artist repetition penalty based on recent songs
    fn calculate_artist_repetition_score(
        &self,
        current_playlist: &[Song],
        candidate: &Song,
    ) -> f32 {
        let avoid_within = self.config.transition_rules.avoid_artist_repeats_within;

        // Check the last N songs (where N = avoid_artist_repeats_within)
        let check_count = avoid_within.min(current_playlist.len());
        let recent_songs = &current_playlist[current_playlist.len() - check_count..];

        // Debug output
        println!(
            "    Artist repetition check for '{}' against {} recent songs:",
            candidate.artist, check_count
        );

        // Check if candidate artist appears in recent songs
        for (index, recent_song) in recent_songs.iter().enumerate() {
            println!(
                "      Position {}: '{}' == '{}' ? {}",
                current_playlist.len() - check_count + index + 1,
                recent_song.artist,
                candidate.artist,
                recent_song.artist == candidate.artist
            );

            if recent_song.artist == candidate.artist {
                // Artist repetition found - return much stronger penalty based on how recent
                // index 0 = oldest in recent_songs, index (check_count-1) = newest
                // Higher index = more recent = lower score (higher penalty)
                let recency_factor = (index + 1) as f32 / check_count as f32; // Range: 1/check_count to 1.0
                // Convert to penalty: more recent (higher recency_factor) = much lower score
                // Use a much stronger penalty - 0.0 for most recent, up to 0.2 for oldest
                let penalty = 0.2 * (1.0 - recency_factor); // Range: 0.0 (most recent) to ~0.2 (oldest)

                println!(
                    "      ARTIST REPETITION PENALTY: {} matches {} at recent position {}: Score {} (recency: {})",
                    recent_song.artist, candidate.artist, index, penalty, recency_factor
                );

                return penalty;
            }
        }

        println!("      No artist repetition found - full score");

        1.0 // No artist repetition found - full score
    }

    /// Calculate genre compatibility with the playlist's overall genre profile
    fn calculate_genre_compatibility_score(
        &self,
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
        let mut playlist_genres = std::collections::HashMap::new();
        for song in current_playlist {
            for genre in song.get_all_genres() {
                *playlist_genres.entry(genre.to_lowercase()).or_insert(0) += 1;
            }
        }

        if playlist_genres.is_empty() {
            return 0.5; // Neutral when no genre info in playlist
        }

        // Calculate compatibility based on shared genres
        let mut compatibility_score = 0.0_f32;
        let mut genre_checks = 0;

        for candidate_genre in candidate_genres {
            let genre_lower = candidate_genre.to_lowercase();
            if let Some(&frequency) = playlist_genres.get(&genre_lower) {
                // Genre exists in playlist - score based on frequency
                let frequency_score = (frequency as f32 / current_playlist.len() as f32).min(1.0);
                compatibility_score += 0.5 + frequency_score * 0.5; // 0.5-1.0 range
            } else {
                // New genre - slight penalty for diversity, but not harsh
                compatibility_score += 0.3;
            }
            genre_checks += 1;
        }

        if genre_checks > 0 {
            compatibility_score / genre_checks as f32
        } else {
            0.5
        }
    }
}
