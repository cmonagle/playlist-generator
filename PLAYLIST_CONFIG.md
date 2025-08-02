# Playlist Configuration Guide

This document explains how to configure playlists using the `playlists.json` file.

## Configuration Structure

Each playlist configuration is a JSON object with the following structure:

### Basic Properties

- **`name`** (string): The name of the playlist that will be created
- **`target_length`** (number): Target number of songs for the playlist

### Genre Filtering

- **`acceptable_genres`** (array of strings): List of genres that are acceptable for this playlist. Songs must match at least one of these genres to be included. If undefined, all 

### BPM Thresholds (optional)

- **`bpm_thresholds.max_bpm`** (number): Maximum BPM for songs considered "softer"
- **`bpm_thresholds.min_bpm`** (number): Minimum BPM for songs considered "upbeat"

### Quality Weights (0.0 to 1.0)

These weights determine how much each factor contributes to playlist quality:

- **`artist_diversity`**: How much to prioritize having different artists
- **`bpm_transition_smoothness`**: How much to prioritize smooth BPM transitions
- **`genre_coherence`**: How much to prioritize genre consistency
- **`popularity_balance`**: How much to balance popular vs. obscure tracks
- **`duration_consistency`**: How much to prioritize consistent song lengths
- **`era_cohesion`**: How much to group songs from similar time periods

### Transition Rules

- **`max_bpm_jump`** (number): Maximum allowed BPM difference between consecutive songs
- **`preferred_bpm_change`** (number): Preferred BPM change direction (negative = slower, positive = faster, 0 = neutral)
- **`avoid_artist_repeats_within`** (number): Number of songs to skip before repeating an artist

### Preference Weights

- **`starred_boost`** (number): How much to boost starred/favorited tracks
- **`play_count_weight`** (number): How much to weight play count in selection
- **`recency_penalty_weight`** (number): How much to penalize recently played tracks
- **`randomness_factor`** (0.0 to 1.0): Amount of randomness in selection
- **`discovery_mode`** (boolean): If true, prioritizes less-played tracks

## Example Configurations

### Morning Chill Playlist
```json
{
  "name": "Morning Chill",
  "target_length": 25,
  "acceptable_genres": ["Jazz", "Chillout", "Indie Folk", "Lo-Fi"],
  "bpm_thresholds": {
    "softer_max": 85,
    "upbeat_min": 95
  },
  "preference_weights": {
    "starred_boost": 40.0,
    "discovery_mode": false
  }
}
```

### High Energy Workout
```json
{
  "name": "Workout",
  "target_length": 40,
  "acceptable_genres": ["Electronic", "Rock", "Metal", "Hip Hop"],
  "bpm_thresholds": {
    "softer_max": 120,
    "upbeat_min": 140
  },
  "preference_weights": {
    "starred_boost": 80.0,
    "discovery_mode": false
  }
}
```

### Discovery Mode
```json
{
  "name": "Discovery",
  "target_length": 20,
  "acceptable_genres": ["Experimental", "Indie", "Alternative", "World Music"],
  "preference_weights": {
    "starred_boost": 10.0,
    "play_count_weight": 5.0,
    "discovery_mode": true
  }
}
```

## Tips for Configuration

1. **Genre Selection**: Start broad and narrow down if playlists become too eclectic
2. **BPM Thresholds**: Adjust based on your music collection's BPM distribution
3. **Discovery Mode**: Use sparingly - great for finding new music but can feel random
4. **Quality Weights**: Higher values = more emphasis on that factor
5. **Target Length**: Consider listening context (commute = 20-30, workout = 40+)

## Testing Your Configuration

Run the playlist generator in debug mode to see how your configurations perform:
```bash
cargo run -- --debug
```

This will show you:
- Which songs are selected for each playlist
- Quality scores and metadata
- BPM ranges and genre distributions
- Artist diversity statistics

Adjust your weights and genres based on the results until you get playlists that match your preferences.
