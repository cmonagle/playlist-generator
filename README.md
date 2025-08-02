# Playlist Generator

This is a Rust project that uses the opensubsonic API to generate playlists using heuristics and metadata.

## Features
- Connects to opensubsonic API
- Generates playlists based on heuristics and metadata
- **Automatically creates/updates playlists via API**
- **Filters out non-songs (interludes, sketches, etc.)**
- **Suitable for cron job automation**
- **Idempotent operations (safe to run repeatedly)**
- **Comprehensive logging for monitoring**

## Getting Started
1. Install Rust and Cargo
2. Copy the environment configuration: `cp .env.example .env`
3. Edit `.env` file with your opensubsonic server details
4. Build the project: `cargo build`
5. Run the project: `cargo run`

## Cron Job Setup

This application is designed to run as a daily cron job to automatically generate fresh playlists. Here's how to set it up:

### 1. Build Release Version
```bash
cargo build --release
```

### 2. Set Up Cron Job
The easiest way is to use the provided script:

```bash
# Make the script executable
chmod +x generate-playlists.sh

# Add to crontab (run daily at 6:00 AM)
0 6 * * * /path/to/playlist-generator-rust/generate-playlists.sh
```

Or run the binary directly:
```bash
# Run daily at 6:00 AM to generate fresh playlists
0 6 * * * cd /path/to/playlist-generator-rust && ./target/release/playlist-generator >> /var/log/playlist-generator.log 2>&1
```

### 3. Monitor Logs
The application provides detailed logging suitable for monitoring cron job execution:
- ✓ Success indicators for each step
- ✗ Error messages with details
- Summary of playlist creation results
- Suitable exit codes for monitoring systems

### 4. Environment Variables
Ensure your `.env` file is properly configured with:
- `BASE_URL`: Your opensubsonic server URL
- `USERNAME`: Your API username  
- `PASSWORD`: Your API password

The application will automatically:
- Delete existing playlists with the same names
- Create fresh "Softer Genres Playlist" and "Upbeat Genres Playlist"
- Log all operations for monitoring
- Return appropriate exit codes for automation

## Project Structure
- `src/`: Rust source code
- `generate-playlists.sh`: Cron job script for automated execution
- `.github/copilot-instructions.md`: Copilot custom instructions
- `.vscode/tasks.json`: VS Code tasks

## License
MIT

## Environment Variables
- Create an `.env` file in the project root. You can copy from the provided `.env.example`:
  ```sh
  cp .env.example .env
  ```
- Define the following variables in `.env`:
  - `BASE_URL`: The base URL for the opensubsonic API.
  - `USERNAME`: The username for API authentication.
  - `PASSWORD`: The password for API authentication.

## Tasks

### Playlist Generation

#### Task 1: Define, Set Up, and Document Environment Variables
- Create an `.env` file to store configuration variables.
- Define the following variables:
  - `BASE_URL`: The base URL for the opensubsonic API.
  - `USERNAME`: The username for API authentication.
  - `PASSWORD`: The password for API authentication.
- Update the documentation to include instructions for setting up the `.env` file.

#### Task 2: Create a Client to Consume Environment Variables
- Implement a function to read the `.env` file and load the configuration variables.
- Use these variables to initialize an HTTP client for interacting with the opensubsonic API.

#### Task 3: Fetch Songs from the API
- Step 1: Define the API endpoint for fetching songs.
- Step 2: Implement a function to send a GET request to the API endpoint.
- Step 3: Parse the API response to extract song metadata, including:
  - Title
  - Artist
  - Album
  - Genre
  - BPM
- Step 4: Handle errors gracefully, such as network issues or invalid responses.

#### Task 4: Categorize Songs by Genre and BPM
- Step 1: Define a list of "softer" genres (e.g., "acoustic", "ambient", "classical").
- Step 2: Define a list of "upbeat" genres (e.g., "pop", "rock", "dance").
- Step 3: Implement a function to filter songs into these two categories based on their genre.
- Step 4: Add additional filtering logic to exclude songs with missing or invalid BPM values.
- Step 5: Sort each category by BPM in ascending order.

#### Task 5: Generate Playlists
- Step 1: Create a playlist structure to hold categorized songs.
- Step 2: Populate the "Softer Genres Playlist" with songs from the softer genres category.
- Step 3: Populate the "Upbeat Genres Playlist" with songs from the upbeat genres category.
- Step 4: Ensure each playlist is formatted for compatibility with the Subsonic API.

#### Task 6: Create and Update Playlists via API
- Step 1: Implement a function to create playlists using the opensubsonic API endpoint.
- Step 2: Create or update the "Softer Genres Playlist" at the API endpoint, overwriting any existing playlist with the same base name pattern.
- Step 3: Create or update the "Upbeat Genres Playlist" at the API endpoint, overwriting any existing playlist with the same base name pattern.
- Step 4: Handle API errors gracefully, such as authentication failures or network issues.
- Step 5: Log playlist creation results for monitoring (suitable for cron job execution).
- Step 6: Implement idempotent playlist operations to safely replace previous day's playlists using pattern matching.

### Playlist Quality Criteria and Heuristics

To create high-quality playlists that resemble Spotify's Daily Mixes, we should implement the following criteria and heuristics based on our available metadata:

#### Artist Diversity (Available Metadata: `artist`, `artist_id`, `album_id`)
- **No consecutive songs by the same artist**: Avoid playing two songs by the same artist back-to-back
- **Artist spacing**: Maintain at least 3-5 songs between tracks by the same artist
- **Artist frequency limit**: Limit any single artist to maximum 10-15% of the total playlist
- **Album diversity**: Avoid too many songs from the same album consecutively

#### Genre and Mood Flow (Available Metadata: `genre`, `bpm`)
- **Smooth BPM transitions**: Ensure BPM doesn't jump drastically between consecutive songs (max ±20 BPM difference)
- **Genre coherence**: Keep related subgenres together while maintaining overall category consistency
- **BPM progression**: Create logical tempo progressions within playlist sections
- **Energy curve**: Build natural energy flows using BPM as a proxy for energy level

#### Temporal Considerations (Available Metadata: `year`)
- **Era mixing**: Blend different decades appropriately - don't cluster all old or new music together
- **Release year spacing**: Avoid too many songs from the same year consecutively
- **Decade balance**: Include variety across different decades when possible

#### User Preference Integration (Available Metadata: `play_count`, `starred`, `played`)
- **Play count weighting**: Favor songs with higher play counts (user preference indicator)
- **Starred track priority**: Include starred/favorited tracks more prominently
- **Recent play consideration**: Factor in when songs were last played to avoid immediate repetition
- **Discovery balance**: Mix popular (high play count) with less-played tracks

#### Technical Audio Quality (Available Metadata: `duration`, `bit_rate`)
- **Duration variety**: Mix song lengths - avoid too many very short or very long tracks in sequence
- **Audio quality consistency**: Maintain consistent bit rates when possible
- **Track positioning**: Consider track numbers for natural album flow when including multiple songs from same album

#### Playlist Structure Optimization
- **Playlist length**: Target 30-50 songs for daily mixes (calculated from duration metadata)
- **Frontload favorites**: Place starred tracks and high play count songs earlier in the playlist
- **Natural album flow**: When including multiple tracks from the same album, respect original track order when possible

#### Implementation Priority (Based on Available Data)
1. **Phase 1**: Artist diversity, basic BPM transitions, genre categorization
2. **Phase 2**: Year/era mixing, play count weighting, duration balancing  
3. **Phase 3**: Starred track integration, recent play avoidance, album flow optimization
4. **Phase 4**: Advanced BPM curve optimization, multi-criteria scoring system

#### Available Metadata Summary
Our OpenSubsonic API provides the following useful fields for playlist generation:
- **Core Info**: `title`, `artist`, `album`, `genre`, `year`, `duration`, `bpm`
- **User Data**: `play_count`, `starred`, `played` (last played timestamp)
- **Organization**: `track`, `disc_number`, `album_id`, `artist_id`
- **Technical**: `bit_rate`, `content_type`

#### Non-Song Filtering
The application automatically filters out non-musical content using heuristics based on track titles and metadata:

- **Interludes & Transitions**: "interlude", "intro", "outro", "bridge", "transition"
- **Sketches & Fragments**: "sketch", "fragment", "snippet", "bits"
- **Spoken Content**: "monologue", "dialogue", "speech", "interview"
- **Ambient Non-Songs**: "atmosphere", "soundscape", "field recording", "rain", "ocean"
- **Duration-Based**: Tracks shorter than 30 seconds or longer than 15 minutes
- **Instrumental Markers**: Short tracks with "(instrumental)" in parentheses
- **Other Indicators**: "silence", "test", "announcement", numeric-only titles

This ensures playlists contain only actual songs rather than interludes, sketches, or other non-musical content.

#### Smart Playlist Management
The application uses intelligent playlist management to handle daily updates:

- **Pattern-Based Cleanup**: Instead of exact name matching, it uses base name patterns to find existing playlists
- **Dynamic Genre Handling**: Playlist names include dominant genres (e.g., "Chill Vibes · Rock"), which can change daily
- **Automatic Cleanup**: All existing playlists matching the base pattern (e.g., "Chill Vibes") are removed before creating new ones
- **Example**: 
  - Day 1: Creates "Chill Vibes · Jazz"
  - Day 2: Removes "Chill Vibes · Jazz" and creates "Chill Vibes · Folk"
  - This ensures only one playlist of each type exists at any time

#### Limitations and Future Enhancements
- **Missing Audio Analysis**: No key signature, acousticness, energy, or valence data
- **Limited User Context**: No listening history patterns or collaborative filtering data  
- **No Seasonal Data**: No holiday or seasonal relevance information
- **Future API Enhancement**: Consider integrating with additional music analysis APIs for advanced features

### Future Enhancements
- ~~Add support for saving playlists to a file.~~ ✅ **IMPLEMENTED: API playlist creation**
- Allow users to specify custom genres or BPM ranges for playlist generation.
- Implement the playlist quality heuristics listed above.
- Add machine learning models for better song recommendations.
- Create user preference learning and feedback systems.
- Add configuration file support for playlist names and criteria.
- Implement more sophisticated error handling and retry logic.
- Add support for playlist descriptions and metadata in API calls.
