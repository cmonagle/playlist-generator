
use anyhow::Result;
use clap::Parser;

mod config;
mod client;
mod models;
mod playlist;

#[cfg(test)]
mod playlist_tests;

use crate::config::load_config;
use crate::client::SubsonicClient;
use crate::models::Song;
use crate::playlist::{PlaylistGenerator, PlaylistConfig};

#[derive(Parser)]
#[command(name = "playlist-generator")]
#[command(about = "Playlist Generator for OpenSubsonic servers")]
#[command(version)]
struct Args {
    /// Path to the playlist configuration JSON file
    #[arg(short = 'c', long = "config", default_value = "playlists.json")]
    config_file: String,
    
    /// Enable debug mode - print playlist details to stdout instead of uploading
    #[arg(short = 'd', long = "debug")]
    debug: bool,
    
    /// Quiet mode - reduce output verbosity
    #[arg(short = 'q', long = "quiet")]
    quiet: bool,
}

fn main() -> Result<()> {
    let args = Args::parse();
    
    // Validate that the playlist configuration file exists before proceeding
    if !std::path::Path::new(&args.config_file).exists() {
        eprintln!("Error: Playlist configuration file '{}' not found.", args.config_file);
        eprintln!("Please ensure the file exists or specify a different file with --config.");
        return Err(anyhow::anyhow!("Configuration file '{}' not found", args.config_file));
    }
    
    // Load configuration from .env
    let config = load_config()?;

    // Initialize API client
    let client = SubsonicClient::new(config);

    // Test connection first
    println!("Testing API connection...");
    match client.ping() {
        Ok(_) => println!("✓ API connection successful"),
        Err(e) => {
            eprintln!("✗ API connection failed: {}", e);
            return Err(e);
        }
    }

    // Fetch random songs from the API
    println!("\nFetching songs for playlist generation...");
    let songs = client.fetch_songs(Some(500))?; // Fetch 500 random songs for better variety
    
    println!("Fetched {} songs total.", songs.len());
    
    // Show sample of fetched songs with more metadata
    println!("\nSample of fetched songs with metadata:");
    for song in &songs[..std::cmp::min(3, songs.len())] {
        let all_genres = song.get_all_genres();
        let genres_display = if all_genres.is_empty() { 
            "None".to_string() 
        } else { 
            all_genres.join(", ") 
        };
        
        println!("- {} by {} [{}]", song.title, song.artist, song.album);
        println!("  Genres: {} | BPM: {:?} | Year: {:?}", genres_display, song.bpm, song.year);
        println!("  Play Count: {:?} | Starred: {} | Duration: {:?}s", 
            song.play_count, 
            if song.starred.is_some() { "★" } else { "☆" },
            song.duration
        );
        println!("  Track: {:?} | Bit Rate: {:?} kbps", song.track, song.bit_rate);
        println!();
    }

    // Load playlist configurations from JSON file
    println!("\nLoading playlist configurations from: {}", args.config_file);
    let playlist_configs = match PlaylistConfig::load_all_from_file(&args.config_file) {
        Ok(configs) => {
            println!("Loaded {} playlist configurations", configs.len());
            configs
        }
        Err(e) => {
            eprintln!("Failed to load playlist configurations: {}", e);
            return Err(anyhow::anyhow!("Failed to load playlist configurations: {}", e));
        }
    };
    
    // First, filter out non-songs and show statistics
    let original_count = songs.len();
    let actual_songs: Vec<Song> = songs.into_iter()
        .filter(|song| crate::playlist::filters::SongFilters::is_actual_song(song))
        .collect();
    
    let filtered_out_count = original_count - actual_songs.len();
    if filtered_out_count > 0 {
        println!("Filtered out {} non-songs (interludes, sketches, etc.)", filtered_out_count);
    }
    println!("Using {} actual songs for playlist generation", actual_songs.len());
    
    // Generate playlists using loaded configurations
    println!("\nGenerating playlists...");
    let playlists: Vec<_> = playlist_configs.into_iter()
        .map(|config| {
            let generator = PlaylistGenerator::new(config.clone());
            generator.generate_playlist(
                actual_songs.clone(), 
                Some(config.name.clone()), 
                config.target_length
            )
        })
        .collect();
    
    // Display generation results
    println!("\n=== GENERATION RESULTS ===");
    println!("Generated {} playlists", playlists.len());
    
    // Create playlists via API and log results
    let mut creation_results = Vec::new();
    
    for playlist in &playlists {
        println!("\n{}", playlist.name);
        println!("{}", "=".repeat(playlist.name.len()));
        println!("Quality Score: {:.1}/100", playlist.quality_score * 100.0);
        
        if playlist.songs.is_empty() {
            println!("No songs found for this category - skipping playlist creation.");
            creation_results.push((playlist.name.clone(), false, "No songs available".to_string()));
            continue;
        }
        
        // Display playlist metadata
        println!("\n📊 Playlist Details:");
        println!("   Songs: {} | Duration: {}m{}s | Avg BPM: {:.1}", 
            playlist.songs.len(),
            playlist.metadata.total_duration / 60,
            playlist.metadata.total_duration % 60,
            playlist.metadata.average_bpm
        );
        println!("   Unique Artists: {} | BPM Range: {}-{}", 
            playlist.metadata.artist_count,
            playlist.metadata.bpm_range.0,
            playlist.metadata.bpm_range.1
        );
        
        if let (Some(min_year), Some(max_year)) = playlist.metadata.era_span {
            if min_year == max_year {
                println!("   Era: {}", min_year);
            } else {
                println!("   Era: {} - {}", min_year, max_year);
            }
        }
        
        // Show top genres
        let mut top_genres: Vec<_> = playlist.metadata.genre_distribution.iter().collect();
        top_genres.sort_by(|a, b| b.1.cmp(a.1));
        if !top_genres.is_empty() {
            let top_3: Vec<String> = top_genres.iter()
                .take(3)
                .map(|(genre, count)| format!("{} ({})", genre, count))
                .collect();
            println!("   Top Genres: {}", top_3.join(", "));
        }
        
        // Collect song IDs for API call
        let song_ids: Vec<String> = playlist.songs.iter().map(|song| song.id.clone()).collect();
        
        // Determine the base name pattern for cleanup (remove genre-specific parts)
        let base_name_pattern = if playlist.name.contains("Chill") {
            "Daylist: Chill"
        } else if playlist.name.contains("Upbeat") {
            "Daylist: Upbeat"
        } else if playlist.name.contains("Workout") {
            "Workout Mix"
        } else if playlist.name.contains("Discovery") {
            "Discovery Mix"
        } else {
            &playlist.name // For other playlists, use the full name
        };
        
        if args.debug {
            // Debug mode: print playlist details instead of uploading
            println!("\n🔍 DEBUG MODE: Playlist '{}' (would create via API)", playlist.name);
            println!("   Would clean up existing playlists matching pattern: '{}'", base_name_pattern);
            println!("   Song IDs that would be added to playlist:");
            
            // Print detailed song information
            for (i, song) in playlist.songs.iter().enumerate() {
                let all_genres = song.get_all_genres();
                let genres_display = if all_genres.is_empty() { 
                    "Unknown".to_string() 
                } else { 
                    all_genres.join(", ") 
                };
                
                let starred_indicator = if song.starred.is_some() { " ★" } else { "" };
                let play_count_display = song.play_count
                    .map(|pc| format!(" ({}x played)", pc))
                    .unwrap_or_default();
                
                println!("     {}. \"{}\" by {}{}{}", 
                    i + 1,
                    song.title, 
                    song.artist,
                    starred_indicator,
                    play_count_display
                );
                println!("        Album: {} | Genres: {} | BPM: {} | Duration: {}s | ID: {}", 
                    song.album,
                    genres_display,
                    song.bpm.unwrap_or(0),
                    song.duration.unwrap_or(0),
                    song.id
                );
                
                // Show year and additional metadata if available
                let mut extra_info = Vec::new();
                if let Some(year) = song.year {
                    extra_info.push(format!("Year: {}", year));
                }
                if let Some(track) = song.track {
                    extra_info.push(format!("Track: {}", track));
                }
                if let Some(bit_rate) = song.bit_rate {
                    extra_info.push(format!("{}kbps", bit_rate));
                }
                if let Some(played) = &song.played {
                    extra_info.push(format!("Last played: {}", played));
                }
                
                if !extra_info.is_empty() {
                    println!("        {}", extra_info.join(" | "));
                }
                println!(); // Empty line between songs
            }
            
            creation_results.push((playlist.name.clone(), true, "Debug mode - not uploaded".to_string()));
        } else {
            // Normal mode: Create playlist via API with pattern-based cleanup
            println!("\n🎵 Creating playlist '{}' via API...", playlist.name);
            println!("   Cleaning up existing playlists matching pattern: '{}'", base_name_pattern);
            match client.create_playlist_with_pattern_cleanup(&playlist.name, base_name_pattern, &song_ids) {
                Ok(playlist_id) => {
                    println!("✓ Successfully created playlist '{}' with ID: {}", playlist.name, playlist_id);
                    creation_results.push((playlist.name.clone(), true, format!("Created with ID: {}", playlist_id)));
                }
                Err(e) => {
                    eprintln!("✗ Failed to create playlist '{}': {}", playlist.name, e);
                    creation_results.push((playlist.name.clone(), false, format!("Error: {}", e)));
                }
            }
        }
    }

    // Summary of playlist creation results (suitable for cron job monitoring)
    println!("\n=== PLAYLIST CREATION SUMMARY ===");
    let successful_creations = creation_results.iter().filter(|(_, success, _)| *success).count();
    let total_attempts = creation_results.len();
    
    println!("Successfully created {}/{} playlists", successful_creations, total_attempts);
    
    for (name, success, message) in &creation_results {
        let status = if *success { "✓" } else { "✗" };
        println!("{} {}: {}", status, name, message);
    }
    
    if successful_creations == total_attempts && total_attempts > 0 {
        println!("\n🎉 All playlists created successfully! Daily playlist generation complete.");
    } else if successful_creations > 0 {
        println!("\n⚠️ Partial success: {}/{} playlists created.", successful_creations, total_attempts);
    } else {
        println!("\n❌ No playlists were created successfully.");
        return Err(anyhow::anyhow!("Playlist creation failed"));
    }

    Ok(())
}
