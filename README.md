# Playlist Generator

A Rust application that generates playlists for OpenSubsonic servers using available music metadata. Creates playlists based on genre, BPM, and user listening data, with basic heuristics for song ordering and artist diversity.

**Note**: This is a weekend project, using LLM code generation. Please use with caution! 

## Features

- **Playlist Generation**: Creates playlists based on genre, BPM, and user preferences
- **OpenSubsonic API Integration**: Works with OpenSubsonic-compatible servers (Navidrome, Airsonic, etc.)
- **Content Filtering**: Excludes tracks that appear to be interludes, sketches, or non-musical content
- **Basic Quality Heuristics**: Avoids consecutive songs by the same artist and manages BPM transitions
- **Automation Support**: Suitable for cron job automation with logging
- **JSON Configuration**: Configurable playlists with genre filters and preferences
- **Playlist Management**: Replaces existing playlists to prevent accumulation

## Installation

### Prerequisites
- Rust 1.70+ and Cargo
- Access to an OpenSubsonic-compatible music server (Navidrome, Airsonic, etc.)

### Setup
1. **Clone the repository**
   ```bash
   git clone <repository-url>
   cd playlist-generator-rust
   ```

2. **Configure environment variables**
   ```bash
   cp .env.example .env
   ```
   Edit `.env` with your server details:
   ```env
   BASE_URL=https://your-music-server.com
   USERNAME=your-username
   PASSWORD=your-password
   ```

3. **Build the application**
   ```bash
   cargo build --release
   ```

## Usage

### Basic Usage

**Generate playlists:**
```bash
cargo run
```

**Debug mode (preview without creating playlists):**
```bash
cargo run -- --debug
```

**Custom configuration file:**
```bash
cargo run -- --config my-playlists.json
```

**Quiet mode (reduced output):**
```bash
cargo run -- --quiet
```

### Command Line Options

- `-c, --config <FILE>`: Specify playlist configuration file (default: `playlists.json`)
- `-d, --debug`: Debug mode - show playlist details without uploading to server
- `-q, --quiet`: Reduce output verbosity
- `-h, --help`: Show help information
- `-V, --version`: Show version information

## Configuration

### Playlist Configuration

The application uses a JSON configuration file to define playlists. The configuration format is documented in [`PLAYLIST_CONFIG.md`](PLAYLIST_CONFIG.md), with complete examples available in [`playlists-example.json`](playlists-example.json).

**Basic structure:**
```json
[
  {
    "name": "Morning Chill",
    "target_length": 25,
    "acceptable_genres": ["Jazz", "Chillout", "Indie Folk"],
    "bpm_thresholds": {
      "min_bpm": 60,
      "max_bpm": 95
    },
    "preference_weights": {
      "starred_boost": 40.0,
      "discovery_mode": false
    }
  }
]
```

**Key configuration options:**
- `name`: Playlist name (dominant genres may be appended automatically)
- `target_length`: Target number of songs
- `acceptable_genres`: List of genres to include
- `bpm_thresholds`: BPM range filters
- `preference_weights`: Boost starred tracks, enable discovery mode
- `quality_weights`: Control artist diversity, BPM transitions, etc.

See [`PLAYLIST_CONFIG.md`](PLAYLIST_CONFIG.md) for detailed configuration documentation.

### Environment Variables

Configure your OpenSubsonic server connection in `.env`:

```env
BASE_URL=https://your-music-server.com
USERNAME=your-username
PASSWORD=your-password
```

**Required variables:**
- `BASE_URL`: Your OpenSubsonic server URL
- `USERNAME`: API username
- `PASSWORD`: API password

## Automation & Deployment

### Cron Job Setup

For automated daily playlist generation:

1. **Build release version:**
   ```bash
   cargo build --release
   ```

2. **Set up cron job:**
   ```bash
   # Run daily at 6:00 AM
   0 6 * * * cd /path/to/playlist-generator-rust && ./target/release/playlist-generator >> /var/log/playlist-generator.log 2>&1
   ```

3. **Using the provided script:**
   ```bash
   chmod +x generate-playlists.sh
   0 6 * * * /path/to/playlist-generator-rust/generate-playlists.sh
   ```

### Monitoring

The application provides logging for monitoring automated runs:
- Success/failure indicators for each operation
- Error messages with details
- Playlist generation statistics
- Creation summaries

Exit codes:
- `0`: Success - all playlists created
- `1`: Failure - configuration or API errors

## How It Works

### Playlist Generation Process

The application generates playlists using the metadata available from OpenSubsonic APIs:

**Content Filtering:**
- Filters out tracks that appear to be interludes, sketches, or ambient non-songs
- Uses title patterns and duration to identify non-musical content
- Focuses on tracks that seem to be actual songs

**Basic Heuristics:**
- **Artist Variety**: Avoids consecutive songs by the same artist
- **BPM Transitions**: Manages tempo changes between songs (configurable jump limits)
- **Genre Grouping**: Keeps similar genres together while allowing some variety
- **User Preferences**: Gives weight to starred tracks and play counts
- **Temporal Distribution**: Balances songs across different years when possible

**Playlist Management:**
- Removes existing playlists matching base name patterns before creating new ones
- Handles dynamic playlist names that include dominant genres
- Prevents accumulation of old playlists

### Available Metadata & Limitations

The generator works with standard OpenSubsonic metadata:
- **Basic Info**: Title, artist, album, genre, year, duration
- **Audio Data**: BPM (when available), bit rate
- **User Data**: Play count, starred status, last played timestamp

**Important Limitations:**
- **No Audio Analysis**: No data on energy, mood, key signature, or acoustic features
- **Genre Dependency**: Relies entirely on existing genre tags in your music library
- **BPM Availability**: BPM-based features only work if your music has BPM metadata
- **Simple Heuristics**: Uses basic rules rather than machine learning or advanced analysis
- **No Collaborative Filtering**: Doesn't learn from listening patterns or similar users

## Project Structure

```
├── src/
│   ├── main.rs           # Main application entry point
│   ├── config.rs         # Configuration loading
│   ├── client.rs         # OpenSubsonic API client
│   ├── models.rs         # Data models
│   └── playlist/         # Playlist generation logic
│       ├── mod.rs
│       ├── config.rs     # Playlist configuration
│       ├── generator.rs  # Core generation algorithms
│       └── metadata.rs   # Metadata analysis
├── playlists.json        # Playlist configuration
├── playlists-example.json # Example configuration
└── generate-playlists.sh # Automation script
```

## Examples

### Sample Output

```
Testing API connection...
✓ API connection successful

Fetching songs for playlist generation...
Fetched 500 songs total.
Filtered out 23 non-songs (interludes, sketches, etc.)
Using 477 actual songs for playlist generation

=== GENERATION RESULTS ===
Generated 2 playlists

Chill Vibes · Jazz
==================
Quality Score: 87.3/100

📊 Playlist Details:
   Songs: 42 | Duration: 156m23s | Avg BPM: 89.2
   Unique Artists: 28 | BPM Range: 65-118
   Era: 1995 - 2023
   Top Genres: Jazz (18), Folk (12), Acoustic (8)

✓ Successfully created playlist 'Chill Vibes · Jazz' with ID: 12345

=== PLAYLIST CREATION SUMMARY ===
Successfully created 2/2 playlists
🎉 All playlists created successfully!
```

### Debug Mode Example

```bash
cargo run -- --debug
```

This shows detailed song information without uploading to your server, useful for testing configurations.

## Troubleshooting

### Common Issues

**Connection failed:**
- Verify your `.env` file has correct server URL and credentials
- Check if your server supports OpenSubsonic API
- Ensure server is accessible from your network

**No songs found:**
- Check your playlist configuration genre filters
- Verify your music library has songs with the required metadata
- Consider broadening BPM ranges or genre lists
- Note that BPM-based filtering requires your music to have BPM metadata

**Compilation errors:**
- Ensure you have Rust 1.70+ installed
- Run `cargo clean` and try building again

**Permission errors:**
- Ensure your user account has playlist creation permissions
- Check server logs for authentication issues

### Debug Tips

1. Use `--debug` mode to preview playlists without uploading
2. Use `--quiet` mode to reduce output for monitoring
3. Check logs for detailed error information
4. Test API connection before playlist generation
5. Verify your music library has the metadata (genres, BPM) your configuration expects

## Contributing

1. Fork the repository
2. Create a feature branch (`git checkout -b feature/amazing-feature`)
3. Commit your changes (`git commit -m 'Add amazing feature'`)
4. Push to the branch (`git push origin feature/amazing-feature`)
5. Open a Pull Request

## License

MIT License - see LICENSE file for details.

## Related Projects

- [Navidrome](https://github.com/navidrome/navidrome) - Modern Music Server and Streamer
- [Airsonic](https://github.com/airsonic/airsonic) - Free, web-based media streamer
- [OpenSubsonic API](http://www.subsonic.org/pages/api.jsp) - API specification
