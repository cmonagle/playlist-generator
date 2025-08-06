use super::filters::SongFilters;
use super::scoring::PlaylistScoring;
use super::transitions::PlaylistTransitions;
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
        let mut remaining_songs = candidate_songs; // Keep persistent list of remaining songs
        
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
                // Hard constraint: Skip candidates that would violate artist repetition rules
                if self.would_violate_artist_repetition(&current_playlist_songs, candidate) {
                    // println!(
                    //     "      SKIPPING '{}' by {} due to artist repetition constraint",
                    //     candidate.title, candidate.artist
                    // );
                    continue;
                }

                // Hard constraint: Skip candidates that would violate album repetition rules
                if self.would_violate_album_repetition(&current_playlist_songs, candidate) {
                    // println!(
                    //     "      SKIPPING '{}' from album '{}' due to album repetition constraint",
                    //     candidate.title, candidate.album
                    // );
                    continue;
                }

                // BPM transition constraint: Use configured max_bpm_jump as hard constraint
                if let Some(last_song) = current_playlist_songs.last() {
                    if let (Some(bpm_a), Some(bpm_b)) = (last_song.bpm, candidate.bpm) {
                        let bpm_diff = (bpm_a as i32 - bpm_b as i32).abs() as u32;
                        if bpm_diff > self.config.transition_rules.max_bpm_jump {
                            continue;
                        }
                    }
                }

                // Hard constraint: Skip candidates that would violate minimum days since last play
                if self.would_violate_min_days_since_last_play(candidate) {
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

                // Always consider the candidate - just pick the best one available
                let should_accept = best_candidate_index.is_none() || combined_score > best_quality_score;

                if should_accept {
                    best_candidate_index = Some(i);
                    best_quality_score = combined_score;
                    best_transition_score = transition_score;
                }
            }

            // Add the best candidate we found
            if let Some(index) = best_candidate_index {
                let chosen_song = remaining_songs.remove(index);
                let quality_contribution = best_quality_score - current_quality;
                // Since we're building iteratively, just append to the end
                playlist.push(PlaylistSong::with_metadata(
                    chosen_song,
                    best_transition_score,
                    quality_contribution,
                ));
            } else {
                // No valid candidates found (all were filtered out by constraints)
                break;
            }
        }

        // Log summary of playlist generation
        if !playlist.is_empty() {
            println!(
                "Generated {} songs for '{}' (target: {})",
                playlist.len(),
                self.config.name,
                target_length
            );
        }

        playlist
    }

    /// Check if a candidate would violate artist repetition rules
    fn would_violate_artist_repetition(&self, current_playlist: &[Song], candidate: &Song) -> bool {
        PlaylistTransitions::would_violate_artist_repetition(
            self.config.transition_rules.avoid_artist_repeats_within,
            current_playlist,
            candidate,
        )
    }

    /// Check if a candidate would violate album repetition rules
    fn would_violate_album_repetition(&self, current_playlist: &[Song], candidate: &Song) -> bool {
        PlaylistTransitions::would_violate_album_repetition(
            self.config.transition_rules.avoid_album_repeats_within,
            current_playlist,
            candidate,
        )
    }

    /// Check if a candidate would violate the minimum days since last play rule
    fn would_violate_min_days_since_last_play(&self, candidate: &Song) -> bool {
        if let Some(min_days) = self.config.min_days_since_last_play {
            PlaylistTransitions::would_violate_min_days_since_last_play(min_days, candidate)
        } else {
            false // No rule to enforce if min_days_since_last_play is not set
        }
    }

    /// Calculate how well a candidate song would fit with the current working playlist
    fn calculate_playlist_transition_score(
        &self,
        current_playlist: &[Song],
        candidate: &Song,
    ) -> f32 {
        PlaylistTransitions::calculate_transition_score(&self.config, current_playlist, candidate)
    }
}
