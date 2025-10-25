# Playlist Configuration Guide

This document explains how to configure playlists using the `playlists.json` file.

## Configuration Structure

Each playlist configuration is a JSON object with the following structure:


### Basic Properties

- **`name`** (string): The name of the playlist that will be created
- **`target_length`** (number): Target number of songs for the playlist

### Genre Filtering

- **`acceptable_genres`** (array of strings): List of genres that are acceptable for this playlist. Songs must match at least one of these genres to be included. If undefined, all genres are acceptable.

- **`unacceptable_genres`** (array of strings): List of genres that should be excluded from this playlist. Songs matching any of these genres will be filtered out, even if they match acceptable_genres. If undefined, no genres are specifically excluded.

**Note**: Both filters work together. A song must pass both filters to be included:
1. Must match at least one acceptable genre (if acceptable_genres is defined)
2. Must NOT match any unacceptable genre (if unacceptable_genres is defined) 

### BPM Thresholds (optional)

- **`bpm_thresholds.max_bpm`** (number): Maximum BPM for songs.
- **`bpm_thresholds.min_bpm`** (number): Minimum BPM for songs.

### Minimum Days Since Last Play (optional)

- **`min_days_since_last_play`** (number): Minimum number of days since a song was last played for it to be included in the playlist. If undefined, this rule is not enforced.

### Playlist Preferences (0.0 to 1.0)

These weights determine how much you want each characteristic in your playlist. Each value expresses your preference:

- **`genre_coherence`**: How consistent you want the genre selection
  - 0.0 = Maximize genre variety (eclectic, discovery-focused)
  - 0.5 = Balanced genre mixing (some variety within theme)
  - 1.0 = Prioritize genre consistency (focused, coherent mood)

- **`popularity_balance`**: How balanced you want popular vs. obscure tracks
  - 0.0 = Allow extreme popularity differences (mix of hits and deep cuts)
  - 0.5 = Moderate popularity variety OR no play count data
  - 1.0 = Prefer balanced mix of moderately popular tracks

- **`era_cohesion`**: How much you want songs from the same time period
  - 0.0 = Maximize era variety (time-traveling playlist)
  - 0.5 = Allow some era mixing (decades can blend)
  - 1.0 = Prioritize same era (nostalgic, historically cohesive)

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
- **`play_count_filter`** (object, optional): Filter songs by play count (see Play Count Filtering section)

## Play Count Filtering

The `play_count_filter` option allows you to create highly targeted playlists based on how often songs have been played. This is perfect for creating discovery playlists, high-rotation favorites, or anything in between.

### Filter Types

#### 1. Exact Play Count
Filter for songs with an exact number of plays:
```json
"play_count_filter": {
  "type": "Exact",
  "count": null        // null = zero plays (never played)
}
```
```json
"play_count_filter": {
  "type": "Exact", 
  "count": 5          // exactly 5 plays
}
```

#### 2. Play Count Range
Filter for songs within a range of play counts:
```json
"play_count_filter": {
  "type": "Range",
  "min": 1,           // minimum plays (optional)
  "max": 10           // maximum plays (optional)
}
```

#### 3. Percentile-Based
Filter for top or bottom percentile of most/least played songs:
```json
"play_count_filter": {
  "type": "Percentile",
  "direction": "top",   // "top" for most played, "bottom" for least played
  "percent": 0.2        // 0.2 = top/bottom 20%
}
```

#### 4. Threshold-Based
Filter songs above/below a play count threshold:
```json
"play_count_filter": {
  "type": "Threshold",
  "operator": "above",  // "above", "below", "at_least", "at_most"
  "count": 15
}
```

### Play Count Filter Examples

#### Zero Plays Discovery
Perfect for finding completely unplayed music:
```json
"play_count_filter": {
  "type": "Exact",
  "count": null
}
```

#### High Rotation Favorites
Your most played songs (top 10%):
```json
"play_count_filter": {
  "type": "Percentile",
  "direction": "top",
  "percent": 0.1
}
```

#### Medium Rotation Mix
Songs you've played a moderate amount (5-20 times):
```json
"play_count_filter": {
  "type": "Range",
  "min": 5,
  "max": 20
}
```

#### Rarely Played Gems
Songs you've barely explored (1-3 plays):
```json
"play_count_filter": {
  "type": "Threshold",
  "operator": "at_most",
  "count": 3
}
```

## Example Configurations

### Zero Plays Discovery Playlist
Perfect for exploring completely unplayed music:
```json
{
  "name": "🔍 Zero Plays",
  "target_length": 30,
  "quality_weights": {
    "artist_diversity": 0.8,
    "bpm_transition_smoothness": 0.3,
    "genre_coherence": 0.4,
    "popularity_balance": 0,
    "era_cohesion": 0.2
  },
  "transition_rules": {
    "max_bpm_jump": 80,
    "preferred_bpm_change": 0,
    "avoid_artist_repeats_within": 8,
    "avoid_album_repeats_within": 10
  },
  "preference_weights": {
    "starred_boost": 0,
    "play_count_weight": 50.0,
    "recency_penalty_weight": 0,
    "randomness_factor": 0.8,
    "discovery_mode": true,
    "play_count_filter": {
      "type": "Exact",
      "count": null
    }
  }
}
```

### High Rotation Favorites
Your most played songs (top 15%):
```json
{
  "name": "🔥 High Rotation",
  "target_length": 25,
  "acceptable_genres": ["Pop", "Rock", "Electronic", "Hip Hop", "Indie Pop"],
  "quality_weights": {
    "artist_diversity": 0.3,
    "bpm_transition_smoothness": 0.4,
    "genre_coherence": 0.6,
    "popularity_balance": 0.2,
    "era_cohesion": 0.3
  },
  "preference_weights": {
    "starred_boost": 100.0,
    "play_count_weight": 30.0,
    "recency_penalty_weight": 10.0,
    "randomness_factor": 0.2,
    "discovery_mode": false,
    "play_count_filter": {
      "type": "Percentile",
      "direction": "top",
      "percent": 0.15
    }
  }
}
```

### Morning Chill Playlist (Coherent & Smooth)
```json
{
  "name": "Morning Chill",
  "target_length": 25,
  "acceptable_genres": ["Jazz", "Chillout", "Indie Folk", "Lo-Fi"],
  "quality_weights": {

    "genre_coherence": 0.8,
    "popularity_balance": 0.6,
    "era_cohesion": 0.4
  },
  "preference_weights": {
    "starred_boost": 40.0,
    "discovery_mode": false
  }
}
```

### Discovery Playlist (Diverse & Exploratory)
```json
{
  "name": "Discovery",
  "target_length": 20,
  "acceptable_genres": ["Experimental", "Indie", "Alternative", "World Music"],
  "quality_weights": {

    "genre_coherence": 0.1,
    "popularity_balance": 0.3,
    "era_cohesion": 0.2
  },
  "preference_weights": {
    "starred_boost": 10.0,
    "play_count_weight": 5.0,
    "discovery_mode": true
  }
}
```

### Chill Playlist (Excluding Harsh Genres)
```json
{
  "name": "Chill Mix",
  "target_length": 30,
  "acceptable_genres": ["Jazz", "Chillout", "Indie Folk", "Lo-Fi", "Ambient", "Downtempo"],
  "unacceptable_genres": ["Metal", "Hardcore", "Punk", "Noise"],
  "quality_weights": {

    "genre_coherence": 0.7,
    "popularity_balance": 0.5,
    "era_cohesion": 0.4
  },
  "preference_weights": {
    "starred_boost": 40.0,
    "discovery_mode": false
  }
}
```

### High Energy Workout (Energetic & Dynamic)
```json
{
  "name": "Workout",
  "target_length": 40,
  "acceptable_genres": ["Electronic", "Rock", "Metal", "Hip Hop"],
  "quality_weights": {

    "genre_coherence": 0.7,
    "popularity_balance": 0.8,
    "era_cohesion": 0.1
  },
  "preference_weights": {
    "starred_boost": 80.0,
    "discovery_mode": false
  }
}
```

### Nostalgic 90s Mix (Era-Focused)
```json
{
  "name": "90s Throwback",
  "target_length": 30,
  "acceptable_genres": ["Alternative Rock", "Grunge", "Hip Hop", "R&B"],
  "quality_weights": {

    "genre_coherence": 0.6,
    "popularity_balance": 0.7,
    "era_cohesion": 0.9
  }
}
```

## Tips for Configuration

1. **Genre Selection**: Start broad and narrow down if playlists become too eclectic
2. **Preference Weights**: Think about what you want, not what's "good" or "bad"
   - Discovery playlist: Low genre_coherence (0.1-0.3) for variety
   - Workout playlist: Low bpm_transition_smoothness (0.2-0.4) for energy changes
   - Study playlist: High genre_coherence (0.8-1.0) and bpm_transition_smoothness (0.8-1.0)
3. **Discovery Mode**: Great for finding new music but can feel random
4. **Target Length**: Consider listening context (commute = 20-30, workout = 40+)
5. **Balancing Preferences**: Most playlists work well with values between 0.3-0.7 for most preferences

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

## Tips for Configuration

1. **Genre Selection**: Start broad and narrow down if playlists become too eclectic
2. **Genre Filtering Logic**: 
   - Songs must match at least one `acceptable_genres` (if specified)
   - Songs must NOT match any `unacceptable_genres` (if specified)
   - If a song matches both acceptable and unacceptable genres, it will be excluded
   - Use `unacceptable_genres` to fine-tune playlists by removing unwanted sub-genres
3. **Preference Weights**:
   - Discovery playlist: Low genre_coherence (0.1-0.3) for variety
   - Workout playlist: Low bpm_transition_smoothness (0.2-0.4) for energy changes
   - Study playlist: High genre_coherence (0.8-1.0) and bpm_transition_smoothness (0.8-1.0)
4. **Discovery Mode**: Great for finding new music but can feel random
5. **Target Length**: Consider listening context (commute = 20-30, workout = 40+)
6. **Balancing Preferences**: Most playlists work well with values between 0.3-0.7 for most preferences
7. **Play Count Filtering Tips**:
   - **Zero plays discovery**: Combine with high `artist_diversity` (0.8) and `randomness_factor` (0.8) for maximum exploration
   - **High rotation favorites**: Use lower `artist_diversity` (0.3) to allow favorite artists to repeat more
   - **Percentile filters**: Work best with large music libraries (500+ songs)
   - **Range filters**: Great for targeting specific listening habits (e.g., 1-5 plays for "songs I liked but forgot about")
   - **Threshold filters**: Useful for broad categories like "rarely played" (at_most 3) or "heavy rotation" (above 20)
   - **Combine with discovery_mode**: For zero/low play count filters, set `discovery_mode: true` and positive `play_count_weight`
