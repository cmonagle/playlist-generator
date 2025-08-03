use serde::{Deserialize, Serialize};

/// Our Song structure with the fields available from the OpenSubsonic API
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Song {
    pub id: String,
    pub title: String,
    pub artist: String,
    pub album: String,
    pub genre: Option<String>,      // Single genre field (legacy)
    pub genres: Option<Vec<Genre>>, // Multiple genres array (OpenSubsonic extension)
    pub bpm: Option<u32>,
    pub duration: Option<u32>,
    pub year: Option<u32>,
    pub track: Option<u32>,
    #[serde(rename = "playCount")]
    pub play_count: Option<u32>,
    #[serde(rename = "discNumber")]
    pub disc_number: Option<u32>,
    #[serde(rename = "albumId")]
    pub album_id: Option<String>,
    #[serde(rename = "artistId")]
    pub artist_id: Option<String>,
    pub played: Option<String>,  // Last played timestamp
    pub starred: Option<String>, // Starred timestamp (if favorited)
    #[serde(rename = "bitRate")]
    pub bit_rate: Option<u32>,
    #[serde(rename = "contentType")]
    pub content_type: Option<String>,
}

/// Genre structure for multiple genres support
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Genre {
    pub name: String,
}

/// Response structure for getRandomSongs API call
#[derive(Debug, Deserialize)]
pub struct RandomSongsResponse {
    #[serde(rename = "subsonic-response")]
    pub subsonic_response: SubsonicResponse,
}

#[derive(Debug, Deserialize)]
#[allow(dead_code)]
pub struct SubsonicResponse {
    pub status: String,
    /// allow undefined version for flexibility
    pub version: String,
    #[serde(rename = "randomSongs")]
    pub random_songs: Option<RandomSongs>,
}

#[derive(Debug, Deserialize)]
pub struct RandomSongs {
    pub song: Vec<Song>,
}

impl Song {
    /// Get all genres for this song, combining both single genre and genres array
    pub fn get_all_genres(&self) -> Vec<String> {
        let mut all_genres = Vec::new();

        // Add the single genre if it exists
        if let Some(ref genre) = self.genre {
            all_genres.push(genre.to_lowercase());
        }

        // Add all genres from the genres array if it exists
        if let Some(ref genres) = self.genres {
            for genre in genres {
                all_genres.push(genre.name.to_lowercase());
            }
        }

        // Remove duplicates and return
        all_genres.sort();
        all_genres.dedup();
        all_genres
    }

    /// Check if this song matches any of the given genre patterns (String version)
    pub fn matches_genre_patterns_string(&self, patterns: &[String]) -> bool {
        let all_genres = self.get_all_genres();

        patterns.iter().any(|pattern| {
            all_genres
                .iter()
                .any(|genre| genre.to_lowercase().contains(&pattern.to_lowercase()))
        })
    }
}

impl Default for Song {
    fn default() -> Self {
        Song {
            id: String::new(),
            title: "Unknown".to_string(),
            artist: "Unknown".to_string(),
            album: "Unknown".to_string(),
            genre: None,
            genres: None,
            bpm: None,
            duration: None,
            year: None,
            track: None,
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
}

/// Response structure for createPlaylist API call
#[derive(Debug, Deserialize)]
pub struct CreatePlaylistResponse {
    #[serde(rename = "subsonic-response")]
    pub subsonic_response: CreatePlaylistSubsonicResponse,
}

#[derive(Debug, Deserialize)]
pub struct CreatePlaylistSubsonicResponse {
    pub status: String,
    pub version: String,
    pub playlist: Option<CreatedPlaylist>,
}

#[derive(Debug, Deserialize)]
pub struct CreatedPlaylist {
    pub id: String,
    pub name: String,
    #[serde(rename = "songCount")]
    pub song_count: Option<u32>,
    pub duration: Option<u32>,
    pub public: Option<bool>,
    pub created: Option<String>,
    pub changed: Option<String>,
}

/// Response structure for getPlaylists API call
#[derive(Debug, Deserialize)]
pub struct GetPlaylistsResponse {
    #[serde(rename = "subsonic-response")]
    pub subsonic_response: GetPlaylistsSubsonicResponse,
}

#[derive(Debug, Deserialize)]
pub struct GetPlaylistsSubsonicResponse {
    pub status: String,
    pub version: String,
    pub playlists: Option<PlaylistsContainer>,
}

#[derive(Debug, Deserialize)]
pub struct PlaylistsContainer {
    pub playlist: Vec<PlaylistInfo>,
}

#[derive(Debug, Deserialize)]
pub struct PlaylistInfo {
    pub id: String,
    pub name: String,
    #[serde(rename = "songCount")]
    pub song_count: Option<u32>,
    pub duration: Option<u32>,
    pub public: Option<bool>,
    pub created: Option<String>,
    pub changed: Option<String>,
}
