use chrono::Local;
use rand::seq::SliceRandom;

/// Helper trait for string formatting
pub trait ToTitleCase {
    fn to_title_case(&self) -> String;
}

impl ToTitleCase for str {
    fn to_title_case(&self) -> String {
        self.split_whitespace()
            .map(|word| {
                let mut chars = word.chars();
                match chars.next() {
                    None => String::new(),
                    Some(first) => {
                        first.to_uppercase().collect::<String>() + &chars.as_str().to_lowercase()
                    }
                }
            })
            .collect::<Vec<_>>()
            .join(" ")
    }
}

/// Playlist naming utilities
pub struct PlaylistNaming;

impl PlaylistNaming {
    /// Generate a descriptive name for the playlist based on metadata
    pub fn generate_playlist_name(
        name: String,
        metadata: &crate::playlist::PlaylistMetadata,
    ) -> String {
        let day_of_week = Local::now().format("%A").to_string();
        let suffix = Self::generate_suffix(metadata);
        format!("{name} {day_of_week} {suffix}").to_lowercase()
    }

    /// Generate a less-repetitive suffix using simple heuristics:
    /// - tempo (based on average BPM)
    /// - era (based on era_span)
    /// - genre (if strong enough presence)
    /// The function composes up to 2 descriptors + a noun.
    fn generate_suffix(metadata: &crate::playlist::PlaylistMetadata) -> String {
        // expanded pools
        const NOUNS: &[&str] = &[
            "tunes",
            "vibes",
            "jams",
            "melodies",
            "grooves",
            "beats",
            "rhythms",
            "sounds",
            "tracks",
            "set",
            "mix",
            "selection",
            "session",
            "collection",
            "playlist",
            "rotation",
            "queue",
            "lineup",
            "compilation",
            "showcase",
            "edition",
            "medley",
            "blend",
            "fusion",
        ];

        let mut rng = rand::thread_rng();

        // genre descriptor - include if it has a strong presence (>35%)
        let genre = if let Some((genre, &count)) = metadata.genre_distribution.iter().max_by_key(|(_, c)| *c) {
            if metadata.total_songs > 0 && (count as f32 / metadata.total_songs as f32) >= 0.35 {
                Some(genre.to_lowercase())
            } else {
                None
            }
        } else {
            None
        };

        // tempo descriptor from BPM - pick randomly from tempo-appropriate words
        let tempo = if metadata.average_bpm > 130.0 {
            // high energy options
            let options = ["energetic", "upbeat", "pumped", "lively"];
            options.choose(&mut rng).copied()
        } else if metadata.average_bpm >= 100.0 {
            // mid-tempo options
            let options = ["steady", "cruising", "flowing", "smooth"];
            options.choose(&mut rng).copied()
        } else if metadata.average_bpm > 0.0 {
            // low tempo/chill options
            let options = ["mellow", "relaxed", "laid-back", "easy"];
            options.choose(&mut rng).copied()
        } else {
            None
        };

        // era descriptor from era_span - only include if span is within acceptable range
        // Allow wider spans for older decades, stricter for recent
        let era = match metadata.era_span {
            (Some(min_y), Some(max_y)) => {
                let span = max_y - min_y;
                let mid_year = (min_y + max_y) / 2;
                
                // Debug output
                println!("Era span: {} to {} (span: {} years, midpoint: {})", min_y, max_y, span, mid_year);
                
                // Determine acceptable span based on era - much more lenient
                let max_acceptable_span = match mid_year {
                    0..=1979 => 30,      // Very lenient for older music (50s-70s)
                    1980..=1999 => 25,   // Lenient for 80s-90s
                    2000..=2009 => 20,   // Moderate for Y2K
                    2010..=2019 => 15,   // Still fairly lenient for 2010s
                    _ => 10,             // Moderately strict for 2020s+
                };
                
                println!("Max acceptable span for this era: {} years", max_acceptable_span);
                
                if span <= max_acceptable_span {
                    let era_label = match mid_year {
                        0..=1949 => "classics",
                        1950..=1959 => "50s",
                        1960..=1969 => "60s",
                        1970..=1979 => "70s",
                        1980..=1989 => "80s",
                        1990..=1999 => "90s",
                        2000..=2009 => "Y2K",
                        2010..=2014 => "uni years",
                        2015..=2019 => "algo era",
                        2020..=2021 => "pandemic",
                        2022..=2023 => "post-lockdown",
                        2024 => "recent",
                        _ => "fresh",
                    };
                    println!("✓ Using era label: {}", era_label);
                    Some(era_label)
                } else {
                    println!("✗ Span too wide, skipping era descriptor");
                    None // Span too wide, skip era descriptor
                }
            }
            _ => None,
        };

        // Build candidate tokens
        let mut tokens: Vec<String> = Vec::new();
        if let Some(g) = genre {
            tokens.push(g);
        }
        if let Some(t) = tempo {
            tokens.push(t.to_string());
        }
        if let Some(e) = era {
            tokens.push(e.to_string());
        }

        // Shuffle and pick up to 2 descriptors
        tokens.shuffle(&mut rng);
        let descriptors_count = tokens.len().min(2);
        let descriptors: Vec<String> = tokens.into_iter().take(descriptors_count).collect();

        // Pick a noun
        let noun = NOUNS.choose(&mut rng).unwrap();

        // Build the suffix
        if descriptors.is_empty() {
            noun.to_string()
        } else {
            format!("{} {}", descriptors.join(" "), noun)
        }
    }
}
