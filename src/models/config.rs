use serde::{Deserialize, Serialize};
use std::path::PathBuf;

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct AppConfig {
    pub github: GitHubConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct GitHubConfig {
    pub personal_access_token: Option<String>,
}

impl AppConfig {
    /// Get the config directory path
    pub fn config_dir() -> PathBuf {
        dirs::config_dir()
            .unwrap_or_else(|| PathBuf::from("."))
            .join("github-starcleaner")
    }

    /// Get the config file path
    pub fn config_path() -> PathBuf {
        Self::config_dir().join("config.toml")
    }

    /// Check if a valid token is configured
    pub fn has_token(&self) -> bool {
        self.github
            .personal_access_token
            .as_ref()
            .map(|t| !t.is_empty())
            .unwrap_or(false)
    }

    /// Get the token if available
    pub fn get_token(&self) -> Option<&str> {
        self.github
            .personal_access_token
            .as_ref()
            .filter(|t| !t.is_empty())
            .map(|s| s.as_str())
    }
}
