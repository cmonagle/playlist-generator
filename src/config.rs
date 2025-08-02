use anyhow::Result;

/// Configuration loaded from environment variables
#[derive(Debug)]
pub struct Config {
    pub base_url: String,
    pub username: String,
    pub password: String,
}

/// Load configuration from `.env` and environment
pub fn load_config() -> Result<Config> {
    // Load `.env` file if present
    dotenv::dotenv().ok();
    // Read variables
    let base_url = std::env::var("BASE_URL")?;
    let username = std::env::var("USERNAME")?;
    let password = std::env::var("PASSWORD")?;
    Ok(Config {
        base_url,
        username,
        password,
    })
}
