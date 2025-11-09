use crate::config::Config;
use crate::models::{
    CreatePlaylistResponse, GetPlaylistsResponse, PlaylistInfo, RandomSongsResponse, Song,
};
use anyhow::Result;
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
        let salt = format!(
            "{:x}",
            md5::compute(format!(
                "{}{}",
                std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .unwrap()
                    .as_nanos(),
                "playlist-gen"
            ))
        )[..8]
            .to_string();

        // Calculate token = md5(password + salt)
        let token = format!("{:x}", md5::compute(format!("{}{}", self.password, salt)));

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

        let response = self
            .agent
            .get(&url_token)
            .call()
            .map_err(|e| anyhow::anyhow!("Ping failed: {}", e))?;

        let response_text = response.into_string()?;

        // If token auth failed, try password auth
        if response_text.contains("\"status\":\"failed\"") {
            let url_password = format!(
                "{}/rest/ping?u={}&p={}&v=1.12.0&c=PlaylistGenerator&f=json",
                self.base_url.trim_end_matches('/'),
                encode(&self.username),
                encode(&self.password)
            );

            let response2 = self
                .agent
                .get(&url_password)
                .call()
                .map_err(|e| anyhow::anyhow!("Password ping failed: {}", e))?;

            let response_text2 = response2.into_string()?;

            Ok(response_text2)
        } else {
            Ok(response_text)
        }
    }

    /// Fetch random songs from the Subsonic API
    /// If count > 500, makes multiple requests to accumulate unique songs
    pub fn fetch_songs(&self, count: Option<u32>) -> Result<Vec<Song>> {
        let desired_count = count.unwrap_or(100);
        let max_per_request = 500;
        
        // If we can get it in one request, do that
        if desired_count <= max_per_request {
            return self.fetch_songs_batch(desired_count);
        }
        
        // Otherwise, make multiple requests and collect unique songs
        let mut all_songs = Vec::new();
        let mut seen_ids = std::collections::HashSet::new();
        let mut attempts = 0;
        let max_attempts = 20; // Prevent infinite loops if library is smaller than requested
        
        println!("Fetching {} songs (making multiple requests of {} songs each)...", desired_count, max_per_request);
        
        while all_songs.len() < desired_count as usize && attempts < max_attempts {
            attempts += 1;
            let batch = self.fetch_songs_batch(max_per_request)?;
            
            let initial_count = all_songs.len();
            let batch_size = batch.len();
            
            for song in batch {
                if !seen_ids.contains(&song.id) {
                    seen_ids.insert(song.id.clone());
                    all_songs.push(song);
                    
                    if all_songs.len() >= desired_count as usize {
                        break;
                    }
                }
            }
            
            let added = all_songs.len() - initial_count;
            println!("  Batch {}: got {} songs, {} new (total: {}/{})", 
                     attempts, batch_size, added, all_songs.len(), desired_count);
            
            // If we got very few new songs, the library might be exhausted
            if added < 50 && all_songs.len() < desired_count as usize {
                println!("  Warning: Only got {} new songs in this batch. Library may be smaller than requested count.", added);
            }
        }
        
        if all_songs.len() < desired_count as usize {
            println!("  Note: Retrieved {} songs, less than requested {} (library may be smaller)", 
                     all_songs.len(), desired_count);
        }
        
        Ok(all_songs)
    }
    
    /// Internal helper to fetch a single batch of random songs
    fn fetch_songs_batch(&self, size: u32) -> Result<Vec<Song>> {
        let (salt, token) = self.generate_auth_params();

        // Build URL with query parameters
        let url = format!(
            "{}/rest/getRandomSongs?u={}&t={}&s={}&v=1.16.1&c=PlaylistGenerator&f=json&size={}",
            self.base_url.trim_end_matches('/'),
            self.username,
            token,
            salt,
            size
        );

        // Send GET request
        let response = self
            .agent
            .get(&url)
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
            return Err(anyhow::anyhow!(
                "API returned error status: {}",
                parsed_response.subsonic_response.status
            ));
        }

        // Extract songs from response
        match parsed_response.subsonic_response.random_songs {
            Some(random_songs) => Ok(random_songs.song),
            None => Ok(vec![]),
        }
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

        println!("Getting playlists from: {url}");

        let response = self.agent.get(&url).call()?;
        let response_text = response.into_string()?;

        // println!("Playlists response: {}", response_text);

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
    pub fn create_playlist_with_pattern_cleanup(
        &self,
        name: &str,
        base_name_pattern: &str,
        song_ids: &[String],
    ) -> Result<String> {
        // First, check for existing playlists that start with the base pattern and get their ID
        if let Ok(existing_playlists) = self.get_playlists() {
            let matching_playlists: Vec<_> = existing_playlists
                .iter()
                .filter(|p| {
                    p.name
                        .to_lowercase()
                        .starts_with(base_name_pattern.to_lowercase().as_str())
                })
                .collect();

            if let Some(existing) = matching_playlists.first() {
                println!(
                    "Found existing playlist '{}' matching pattern '{}' (ID: {})",
                    existing.name, base_name_pattern, existing.id
                );
                // Update the existing playlist with new songs
                return self.update_playlist(&existing.id, name, song_ids);
            }
        }

        // If no matching playlist, create a new one
        self.create_playlist(name, song_ids)
    }

    /// Create a new playlist or overwrite existing one
    pub fn create_playlist(&self, name: &str, song_ids: &[String]) -> Result<String> {
        // First, check if playlist already exists and delete it
        if let Ok(existing_playlists) = self.get_playlists() {
            if let Some(existing) = existing_playlists.iter().find(|p| p.name == name) {
                println!(
                    "Playlist '{}' already exists (ID: {}), deleting it first...",
                    name, existing.id
                );
                if let Err(e) = self.delete_playlist(&existing.id) {
                    println!("Warning: Failed to delete existing playlist: {e}");
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

        println!(
            "Creating playlist '{}' with {} songs...",
            name,
            song_ids.len()
        );
        println!("Create playlist URL: {url}");

        let response = self.agent.get(&url).call()?;
        let response_text = response.into_string()?;

        let parsed_response: CreatePlaylistResponse = serde_json::from_str(&response_text)?;

        if parsed_response.subsonic_response.status != "ok" {
            return Err(anyhow::anyhow!("API error: Response status was not 'ok'"));
        }

        match parsed_response.subsonic_response.playlist {
            Some(playlist) => {
                println!(
                    "✓ Successfully created playlist '{}' with ID: {}",
                    name, playlist.id
                );
                Ok(playlist.id)
            }
            None => Err(anyhow::anyhow!("No playlist returned in create response")),
        }
    }

    /// Update an existing playlist with new songs
    pub fn update_playlist(
        &self,
        playlist_id: &str,
        name: &str,
        song_ids: &[String],
    ) -> Result<String> {
        let (salt, token) = self.generate_auth_params();

        // Fetch current playlist songs to remove all tracks
        let get_url = format!(
            "{}/rest/getPlaylist?u={}&t={}&s={}&v=1.16.1&c=playlist-generator&f=json&id={}",
            self.base_url,
            encode(&self.username),
            token,
            salt,
            encode(playlist_id)
        );
        let list_resp = self.agent.get(&get_url).call()?;
        let list_text = list_resp.into_string()?;
        let list_json: serde_json::Value = serde_json::from_str(&list_text)?;
        // Extract existing songs array
        let existing = list_json["subsonic-response"]["playlist"]
            .get("entry")
            .and_then(|s| s.as_array())
            .cloned()
            .unwrap_or_else(Vec::new);

        let mut url = format!(
            "{}/rest/updatePlaylist?u={}&t={}&s={}&v=1.16.1&c=playlist-generator&f=json&playlistId={}&name={}",
            self.base_url,
            encode(&self.username),
            token,
            salt,
            encode(playlist_id),
            encode(name)
        );
        println!(
            "Updating playlist '{}' (ID: {}) with {} existing songs to remove...",
            name,
            playlist_id,
            existing.len()
        );
        // Remove all existing tracks by index in descending order
        for idx in 0..existing.len() {
            url.push_str(&format!("&songIndexToRemove={idx}"));
        }
        // Append new songs
        for song_id in song_ids {
            url.push_str(&format!("&songIdToAdd={}", encode(song_id)));
        }
        println!(
            "Updating playlist '{}' (ID: {}) with {} songs...",
            name,
            playlist_id,
            song_ids.len()
        );
        println!("Update playlist URL: {url}");

        let response = self.agent.get(&url).call()?;
        let response_text = response.into_string()?;

        let parsed: serde_json::Value = serde_json::from_str(&response_text)?;
        if let Some(status) = parsed
            .get("subsonic-response")
            .and_then(|r| r.get("status"))
            .and_then(|s| s.as_str())
        {
            if status == "ok" {
                println!("✓ Successfully updated playlist '{name}' (ID: {playlist_id})");
                Ok(playlist_id.to_string())
            } else {
                Err(anyhow::anyhow!(
                    "API error: Update playlist status was not 'ok': {}",
                    status
                ))
            }
        } else {
            Err(anyhow::anyhow!(
                "Invalid response format from update playlist"
            ))
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

        println!("Deleting playlist ID: {playlist_id}");

        let response = self.agent.get(&url).call()?;
        let response_text = response.into_string()?;

        // println!("Delete playlist response: {}", response_text);

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
                Err(anyhow::anyhow!(
                    "API error: Delete playlist status was not 'ok'"
                ))
            }
        } else {
            Err(anyhow::anyhow!(
                "Invalid response format from delete playlist"
            ))
        }
    }
}
