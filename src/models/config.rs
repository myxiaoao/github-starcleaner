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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_app_config_default() {
        let config = AppConfig::default();
        assert!(config.github.personal_access_token.is_none());
        assert!(!config.has_token());
        assert!(config.get_token().is_none());
    }

    #[test]
    fn test_has_token_with_valid_token() {
        let config = AppConfig {
            github: GitHubConfig {
                personal_access_token: Some("ghp_test_token".to_string()),
            },
        };
        assert!(config.has_token());
    }

    #[test]
    fn test_has_token_with_empty_token() {
        let config = AppConfig {
            github: GitHubConfig {
                personal_access_token: Some("".to_string()),
            },
        };
        assert!(!config.has_token());
    }

    #[test]
    fn test_has_token_with_none() {
        let config = AppConfig {
            github: GitHubConfig {
                personal_access_token: None,
            },
        };
        assert!(!config.has_token());
    }

    #[test]
    fn test_get_token_returns_valid_token() {
        let config = AppConfig {
            github: GitHubConfig {
                personal_access_token: Some("ghp_test_token".to_string()),
            },
        };
        assert_eq!(config.get_token(), Some("ghp_test_token"));
    }

    #[test]
    fn test_get_token_returns_none_for_empty() {
        let config = AppConfig {
            github: GitHubConfig {
                personal_access_token: Some("".to_string()),
            },
        };
        assert!(config.get_token().is_none());
    }

    #[test]
    fn test_config_dir_ends_with_app_name() {
        let dir = AppConfig::config_dir();
        assert!(dir.ends_with("github-starcleaner"));
    }

    #[test]
    fn test_config_path_ends_with_toml() {
        let path = AppConfig::config_path();
        assert!(path.ends_with("config.toml"));
    }

    #[test]
    fn test_config_serialization() {
        let config = AppConfig {
            github: GitHubConfig {
                personal_access_token: Some("test_token".to_string()),
            },
        };
        let serialized = toml::to_string(&config).unwrap();
        assert!(serialized.contains("personal_access_token"));
        assert!(serialized.contains("test_token"));
    }

    #[test]
    fn test_config_deserialization() {
        let toml_str = r#"
[github]
personal_access_token = "my_token"
"#;
        let config: AppConfig = toml::from_str(toml_str).unwrap();
        assert_eq!(config.get_token(), Some("my_token"));
    }
}
