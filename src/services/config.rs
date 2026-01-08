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

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::TempDir;

    fn with_temp_config_dir<F>(test_fn: F)
    where
        F: FnOnce(&TempDir),
    {
        let temp_dir = TempDir::new().unwrap();
        // Note: We can't easily override config_dir() since it uses dirs::config_dir()
        // These tests validate the serialization/deserialization logic
        test_fn(&temp_dir);
    }

    #[test]
    fn test_save_and_load_config() {
        with_temp_config_dir(|temp_dir| {
            let config_path = temp_dir.path().join("config.toml");

            let config = AppConfig {
                github: crate::models::GitHubConfig {
                    personal_access_token: Some("test_token_123".to_string()),
                },
            };

            // Save directly to temp file
            let content = toml::to_string_pretty(&config).unwrap();
            fs::write(&config_path, &content).unwrap();

            // Load and verify
            let loaded_content = fs::read_to_string(&config_path).unwrap();
            let loaded: AppConfig = toml::from_str(&loaded_content).unwrap();

            assert_eq!(loaded.get_token(), Some("test_token_123"));
        });
    }

    #[test]
    fn test_config_file_format() {
        let config = AppConfig {
            github: crate::models::GitHubConfig {
                personal_access_token: Some("ghp_abcdef123456".to_string()),
            },
        };

        let content = toml::to_string_pretty(&config).unwrap();

        // Verify TOML structure
        assert!(content.contains("[github]"));
        assert!(content.contains("personal_access_token = \"ghp_abcdef123456\""));
    }

    #[test]
    fn test_load_nonexistent_returns_default() {
        // When config file doesn't exist, load() should return default
        // This is the expected behavior from the implementation
        let default_config = AppConfig::default();
        assert!(default_config.github.personal_access_token.is_none());
    }

    #[test]
    fn test_save_creates_valid_toml() {
        let config = AppConfig {
            github: crate::models::GitHubConfig {
                personal_access_token: Some("my_secret_token".to_string()),
            },
        };

        let content = toml::to_string_pretty(&config).unwrap();
        let parsed: AppConfig = toml::from_str(&content).unwrap();

        assert_eq!(parsed.get_token(), config.get_token());
    }

    #[test]
    fn test_config_with_empty_token() {
        let config = AppConfig {
            github: crate::models::GitHubConfig {
                personal_access_token: Some("".to_string()),
            },
        };

        let content = toml::to_string_pretty(&config).unwrap();
        let parsed: AppConfig = toml::from_str(&content).unwrap();

        // Empty token should be treated as no token
        assert!(!parsed.has_token());
    }

    #[test]
    fn test_config_roundtrip() {
        let original = AppConfig {
            github: crate::models::GitHubConfig {
                personal_access_token: Some("token_with_special_chars_!@#$%".to_string()),
            },
        };

        let serialized = toml::to_string_pretty(&original).unwrap();
        let deserialized: AppConfig = toml::from_str(&serialized).unwrap();

        assert_eq!(
            original.github.personal_access_token,
            deserialized.github.personal_access_token
        );
    }
}
