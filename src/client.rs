use anyhow::Result;
use crate::config::Config;
use crate::models::{Song, RandomSongsResponse, CreatePlaylistResponse, GetPlaylistsResponse, PlaylistInfo};
use ureq::Agent;
use urlencoding::encode;

/// A simple Subsonic API client using MD5 authentication
pub struct SubsonicClient {
    agent: Agent,
    base_url: String,
    username: String,
    password: String,
}

impl SubsonicClient {
    /// Create a new client with configuration from environment
    pub fn new(config: Config) -> Self {
        let agent = Agent::new();
        
        SubsonicClient {
            agent,
            base_url: config.base_url,
            username: config.username,
            password: config.password,
        }
    }

    /// Generate authentication parameters using salt + token method
    fn generate_auth_params(&self) -> (String, String) {
        // Generate a random salt (at least 6 characters)
        let salt = format!("{:x}", md5::compute(&format!("{}{}", 
            std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_nanos(),
            "playlist-gen"
        )))[..8].to_string();
        
        // Calculate token = md5(password + salt)
        let token = format!("{:x}", md5::compute(&format!("{}{}", self.password, salt)));
        
        println!("Debug - Username: {}, Password: {}, Salt: {}, Token: {}", 
            self.username, self.password, salt, token);
        
        (salt, token)
    }

    /// Test the API connection with a simple ping - try both auth methods
    pub fn ping(&self) -> Result<String> {
        // First try token-based auth (v1.13.0+)
        let (salt, token) = self.generate_auth_params();
        
        let url_token = format!(
            "{}/rest/ping?u={}&t={}&s={}&v=1.16.1&c=PlaylistGenerator&f=json",
            self.base_url.trim_end_matches('/'),
            encode(&self.username),
            token,
            salt
        );

        println!("Trying token auth - Ping URL: {}", url_token);

        let response = self.agent.get(&url_token)
            .call()
            .map_err(|e| anyhow::anyhow!("Ping failed: {}", e))?;
        
        let response_text = response.into_string()?;
        println!("Token auth response: {}", response_text);
        
        // If token auth failed, try password auth
        if response_text.contains("\"status\":\"failed\"") {
            println!("Token auth failed, trying password auth...");
            
            let url_password = format!(
                "{}/rest/ping?u={}&p={}&v=1.12.0&c=PlaylistGenerator&f=json",
                self.base_url.trim_end_matches('/'),
                encode(&self.username),
                encode(&self.password)
            );
            
            println!("Trying password auth - Ping URL: {}", url_password);
            
            let response2 = self.agent.get(&url_password)
                .call()
                .map_err(|e| anyhow::anyhow!("Password ping failed: {}", e))?;
            
            let response_text2 = response2.into_string()?;
            println!("Password auth response: {}", response_text2);
            
            Ok(response_text2)
        } else {
            Ok(response_text)
        }
    }

    /// Fetch random songs from the Subsonic API
    pub fn fetch_songs(&self, count: Option<u32>) -> Result<Vec<Song>> {
        let (salt, token) = self.generate_auth_params();
        
        // Build URL with query parameters
        let size = count.unwrap_or(100);
        let url = format!(
            "{}/rest/getRandomSongs?u={}&t={}&s={}&v=1.16.1&c=PlaylistGenerator&f=json&size={}",
            self.base_url.trim_end_matches('/'),
            self.username,
            token,
            salt,
            size
        );

        println!("Requesting URL: {}", url);

        // Send GET request
        let response = self.agent.get(&url)
            .call()
            .map_err(|e| anyhow::anyhow!("HTTP request failed: {}", e))?;
        
        let response_text = response.into_string()?;
        
        // Debug: print response to see what we're getting
        // println!("API Response: {}", response_text);
        
        // Parse JSON response
        let parsed_response: RandomSongsResponse = serde_json::from_str(&response_text)
            .map_err(|e| anyhow::anyhow!("Failed to parse JSON response: {}", e))?;
        
        // Check if the response is successful
        if parsed_response.subsonic_response.status != "ok" {
            return Err(anyhow::anyhow!("API returned error status: {}", parsed_response.subsonic_response.status));
        }
        
        // Extract songs from response
        match parsed_response.subsonic_response.random_songs {
            Some(random_songs) => Ok(random_songs.song),
            None => Ok(vec![]),
        }
    }

    /// Fetch random songs by genre from the Subsonic API
    pub fn fetch_songs_by_genre(&self, genre: &str, count: Option<u32>) -> Result<Vec<Song>> {
        let (salt, token) = self.generate_auth_params();
        
        // Build URL with query parameters including genre filter
        let size = count.unwrap_or(50);
        let url = format!(
            "{}/rest/getRandomSongs?u={}&t={}&s={}&v=1.16.1&c=PlaylistGenerator&f=json&size={}&genre={}",
            self.base_url.trim_end_matches('/'),
            self.username,
            token,
            salt,
            size,
            urlencoding::encode(genre)
        );

        println!("Requesting songs by genre '{}': {}", genre, url);

        // Send GET request
        let response = self.agent.get(&url)
            .call()
            .map_err(|e| anyhow::anyhow!("HTTP request failed: {}", e))?;
        
        let response_text = response.into_string()?;
        
        // Parse JSON response
        let parsed_response: RandomSongsResponse = serde_json::from_str(&response_text)
            .map_err(|e| anyhow::anyhow!("Failed to parse JSON response: {}", e))?;
        
        // Check if the response is successful
        if parsed_response.subsonic_response.status != "ok" {
            return Err(anyhow::anyhow!("API returned error status: {}", parsed_response.subsonic_response.status));
        }
        
        // Extract songs from response
        match parsed_response.subsonic_response.random_songs {
            Some(random_songs) => Ok(random_songs.song),
            None => Ok(vec![]),
        }
    }

    /// Fetch a large, diverse set of songs using multiple strategies
    pub fn fetch_diverse_songs(&self, total_target: u32) -> Result<Vec<Song>> {
        let mut all_songs = Vec::new();
        
        // Strategy 1: Fetch a large base set of random songs
        println!("Fetching {} random songs as base set...", total_target / 2);
        let random_songs = self.fetch_songs(Some(total_target / 2))?;
        all_songs.extend(random_songs);
        
        // Strategy 2: Target specific popular genres to ensure good coverage
        let target_genres = [
            "rock", "pop", "electronic", "hip hop", "jazz", "classical", 
            "folk", "dance", "indie", "alternative", "funk", "soul"
        ];
        
        let songs_per_genre = (total_target / 2) / target_genres.len() as u32;
        let songs_per_genre = songs_per_genre.max(10); // At least 10 songs per genre
        
        println!("Fetching songs by genre to ensure diversity...");
        for genre in &target_genres {
            match self.fetch_songs_by_genre(genre, Some(songs_per_genre)) {
                Ok(genre_songs) => {
                    println!("  Found {} songs for genre '{}'", genre_songs.len(), genre);
                    all_songs.extend(genre_songs);
                }
                Err(e) => {
                    println!("  Warning: Failed to fetch songs for genre '{}': {}", genre, e);
                    // Continue with other genres
                }
            }
        }
        
        // Remove duplicates based on song ID
        all_songs.sort_by(|a, b| a.id.cmp(&b.id));
        all_songs.dedup_by(|a, b| a.id == b.id);
        
        println!("Fetched {} unique songs total (target was {})", all_songs.len(), total_target);
        
        Ok(all_songs)
    }

    /// Get all existing playlists
    pub fn get_playlists(&self) -> Result<Vec<PlaylistInfo>> {
        let (salt, token) = self.generate_auth_params();
        
        let url = format!(
            "{}/rest/getPlaylists?u={}&t={}&s={}&v=1.16.1&c=playlist-generator&f=json",
            self.base_url, 
            encode(&self.username), 
            token, 
            salt
        );

        println!("Getting playlists from: {}", url);

        let response = self.agent.get(&url).call()?;
        let response_text = response.into_string()?;
        
        println!("Playlists response: {}", response_text);

        let parsed_response: GetPlaylistsResponse = serde_json::from_str(&response_text)?;
        
        // Check if the response was successful
        if parsed_response.subsonic_response.status != "ok" {
            return Err(anyhow::anyhow!("API error: Response status was not 'ok'"));
        }

        // Extract playlists from response
        match parsed_response.subsonic_response.playlists {
            Some(playlists_container) => Ok(playlists_container.playlist),
            None => Ok(vec![]),
        }
    }

    /// Create a new playlist or update existing one, deleting any playlists that start with the base name
    pub fn create_playlist_with_pattern_cleanup(&self, name: &str, base_name_pattern: &str, song_ids: &[String]) -> Result<String> {
        // First, check for existing playlists that start with the base pattern and delete them
        if let Ok(existing_playlists) = self.get_playlists() {
            let matching_playlists: Vec<_> = existing_playlists.iter()
                .filter(|p| p.name.starts_with(base_name_pattern))
                .collect();
            
            for existing in matching_playlists {
                println!("Found existing playlist '{}' matching pattern '{}' (ID: {}), deleting it...", 
                    existing.name, base_name_pattern, existing.id);
                if let Err(e) = self.delete_playlist(&existing.id) {
                    println!("Warning: Failed to delete existing playlist '{}': {}", existing.name, e);
                    // Continue anyway - we'll try to create the new one
                }
            }
        }

        // Now create the new playlist
        self.create_playlist(name, song_ids)
    }

    /// Create a new playlist or update existing one
    pub fn create_playlist(&self, name: &str, song_ids: &[String]) -> Result<String> {
        // First, check if playlist already exists and delete it
        if let Ok(existing_playlists) = self.get_playlists() {
            if let Some(existing) = existing_playlists.iter().find(|p| p.name == name) {
                println!("Playlist '{}' already exists (ID: {}), deleting it first...", name, existing.id);
                if let Err(e) = self.delete_playlist(&existing.id) {
                    println!("Warning: Failed to delete existing playlist: {}", e);
                    // Continue anyway - we'll try to create the new one
                }
            }
        }

        let (salt, token) = self.generate_auth_params();
        
        // Create the playlist with songs
        let mut url = format!(
            "{}/rest/createPlaylist?u={}&t={}&s={}&v=1.16.1&c=playlist-generator&f=json&name={}",
            self.base_url, 
            encode(&self.username), 
            token, 
            salt,
            encode(name)
        );

        // Add song IDs to the URL
        for song_id in song_ids {
            url.push_str(&format!("&songId={}", encode(song_id)));
        }

        println!("Creating playlist '{}' with {} songs...", name, song_ids.len());
        println!("Create playlist URL: {}", url);

        let response = self.agent.get(&url).call()?;
        let response_text = response.into_string()?;
        
        println!("Create playlist response: {}", response_text);

        let parsed_response: CreatePlaylistResponse = serde_json::from_str(&response_text)?;
        
        // Check if the response was successful
        if parsed_response.subsonic_response.status != "ok" {
            return Err(anyhow::anyhow!("API error: Response status was not 'ok'"));
        }

        // Extract playlist ID from response
        match parsed_response.subsonic_response.playlist {
            Some(playlist) => {
                println!("✓ Successfully created playlist '{}' with ID: {}", name, playlist.id);
                Ok(playlist.id)
            },
            None => Err(anyhow::anyhow!("No playlist returned in create response")),
        }
    }

    /// Delete an existing playlist
    pub fn delete_playlist(&self, playlist_id: &str) -> Result<()> {
        let (salt, token) = self.generate_auth_params();
        
        let url = format!(
            "{}/rest/deletePlaylist?u={}&t={}&s={}&v=1.16.1&c=playlist-generator&f=json&id={}",
            self.base_url, 
            encode(&self.username), 
            token, 
            salt,
            encode(playlist_id)
        );

        println!("Deleting playlist ID: {}", playlist_id);

        let response = self.agent.get(&url).call()?;
        let response_text = response.into_string()?;
        
        println!("Delete playlist response: {}", response_text);

        // For delete, we just need to check the status
        let parsed_response: serde_json::Value = serde_json::from_str(&response_text)?;
        
        if let Some(status) = parsed_response
            .get("subsonic-response")
            .and_then(|r| r.get("status"))
            .and_then(|s| s.as_str()) 
        {
            if status == "ok" {
                println!("✓ Successfully deleted playlist");
                Ok(())
            } else {
                Err(anyhow::anyhow!("API error: Delete playlist status was not 'ok'"))
            }
        } else {
            Err(anyhow::anyhow!("Invalid response format from delete playlist"))
        }
    }
}
