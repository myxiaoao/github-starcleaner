use crate::models::AppConfig;
use anyhow::{Context, Result};
use std::fs;

pub struct ConfigService;

impl ConfigService {
    /// Load config from file, returns default if not exists
    pub fn load() -> Result<AppConfig> {
        let path = AppConfig::config_path();

        if !path.exists() {
            return Ok(AppConfig::default());
        }

        let content =
            fs::read_to_string(&path).context("Failed to read config file")?;

        let config: AppConfig =
            toml::from_str(&content).context("Failed to parse config file")?;

        Ok(config)
    }

    /// Save config to file, creating directory if needed
    pub fn save(config: &AppConfig) -> Result<()> {
        let dir = AppConfig::config_dir();
        fs::create_dir_all(&dir).context("Failed to create config directory")?;

        let content =
            toml::to_string_pretty(config).context("Failed to serialize config")?;

        let path = AppConfig::config_path();
        fs::write(&path, content).context("Failed to write config file")?;

        Ok(())
    }

    /// Save PAT to config
    pub fn save_token(token: &str) -> Result<()> {
        let mut config = Self::load().unwrap_or_default();
        config.github.personal_access_token = Some(token.to_string());
        Self::save(&config)
    }

    /// Clear the saved token
    pub fn clear_token() -> Result<()> {
        let mut config = Self::load().unwrap_or_default();
        config.github.personal_access_token = None;
        Self::save(&config)
    }
}
