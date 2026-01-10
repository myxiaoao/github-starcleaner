use crate::models::{AppConfig, Repository, RepositorySelection};
use crate::services::{is_token_expired_error, ConfigService, GitHubService};
use gpui::Global;

/// Current view/screen in the application
#[derive(Debug, Clone, PartialEq, Default)]
pub enum AppScreen {
    #[default]
    Setup,
    Loading,
    RepositoryList,
}

/// Pending confirmation action
#[derive(Debug, Clone)]
pub enum PendingAction {
    /// Unstar a single repo: (repo_id, owner, name, full_name)
    UnstarSingle(u64, String, String, String),
    /// Unstar multiple selected repos: count
    UnstarSelected(usize),
    /// Logout
    Logout,
}

/// Sort field for repositories (API-supported options only)
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub enum SortField {
    /// When the repository was starred (API: created)
    Starred,
    /// When the repository was last pushed to (API: updated)
    #[default]
    Pushed,
}

impl SortField {
    pub fn label(&self) -> &'static str {
        match self {
            SortField::Starred => "Starred",
            SortField::Pushed => "Pushed",
        }
    }

    /// API parameter value
    pub fn api_value(&self) -> &'static str {
        match self {
            SortField::Starred => "created",
            SortField::Pushed => "updated",
        }
    }

    pub fn all() -> &'static [SortField] {
        &[
            SortField::Starred,
            SortField::Pushed,
        ]
    }
}

/// Sort direction
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub enum SortDirection {
    #[default]
    Asc,
    Desc,
}

impl SortDirection {
    pub fn toggle(&self) -> Self {
        match self {
            SortDirection::Asc => SortDirection::Desc,
            SortDirection::Desc => SortDirection::Asc,
        }
    }

    pub fn label(&self) -> &'static str {
        match self {
            SortDirection::Asc => "↑",
            SortDirection::Desc => "↓",
        }
    }

    /// API parameter value
    pub fn api_value(&self) -> &'static str {
        match self {
            SortDirection::Asc => "asc",
            SortDirection::Desc => "desc",
        }
    }
}

/// Global application state
#[derive(Default)]
pub struct AppState {
    pub screen: AppScreen,
    pub config: AppConfig,
    pub github_service: Option<GitHubService>,
    pub repositories: Vec<Repository>,
    pub selection: RepositorySelection,
    pub loading: bool,
    pub loading_more: bool,
    pub error: Option<String>,
    pub username: Option<String>,
    pub current_page: u32,
    pub has_more: bool,
    pub pending_action: Option<PendingAction>,
    pub sort_field: SortField,
    pub sort_direction: SortDirection,
}

impl AppState {
    /// Initialize state from saved config
    pub fn from_config(config: AppConfig) -> Self {
        let screen = if config.has_token() {
            AppScreen::Loading
        } else {
            AppScreen::Setup
        };

        Self {
            screen,
            config,
            current_page: 1,
            has_more: true,
            ..Default::default()
        }
    }

    /// Set PAT and create GitHub service
    pub fn set_token(&mut self, token: String) -> anyhow::Result<()> {
        self.config.github.personal_access_token = Some(token.clone());
        self.github_service = Some(GitHubService::new(&token)?);
        ConfigService::save(&self.config)?;
        Ok(())
    }

    /// Get selected repositories for unstar (owner, repo) pairs
    pub fn get_selected_repos(&self) -> Vec<(String, String)> {
        self.repositories
            .iter()
            .filter(|r| self.selection.is_selected(r.id))
            .map(|r| (r.owner.clone(), r.name.clone()))
            .collect()
    }

    /// Get selected repository IDs
    pub fn get_selected_ids(&self) -> Vec<u64> {
        self.repositories
            .iter()
            .filter(|r| self.selection.is_selected(r.id))
            .map(|r| r.id)
            .collect()
    }

    /// Remove repositories by IDs (after unstar)
    pub fn remove_repos(&mut self, ids: &[u64]) {
        self.repositories.retain(|r| !ids.contains(&r.id));
        self.selection.remove_ids(ids);
    }

    /// Clear error message
    pub fn clear_error(&mut self) {
        self.error = None;
    }

    /// Set error message
    pub fn set_error(&mut self, error: String) {
        self.error = Some(error);
    }

    /// Logout and clear token
    pub fn logout(&mut self) -> anyhow::Result<()> {
        self.github_service = None;
        self.username = None;
        self.repositories.clear();
        self.selection.clear();
        self.screen = AppScreen::Setup;
        ConfigService::clear_token()?;
        self.config.github.personal_access_token = None;
        Ok(())
    }

    /// Handle API errors, with special handling for token expiration
    pub fn handle_api_error(&mut self, err: anyhow::Error, context: &str) {
        if is_token_expired_error(&err) {
            let _ = self.logout();
            self.error = Some("Token expired. Please login again.".to_string());
        } else {
            self.error = Some(format!("{}: {}", context, err));
        }
    }
}

impl Global for AppState {}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::GitHubConfig;
    use chrono::Utc;

    fn create_test_repo(id: u64, name: &str, owner: &str) -> Repository {
        Repository {
            id,
            name: name.to_string(),
            full_name: format!("{}/{}", owner, name),
            owner: owner.to_string(),
            description: None,
            language: None,
            stargazers_count: 0,
            forks_count: 0,
            open_issues_count: 0,
            license: None,
            topics: vec![],
            updated_at: Utc::now(),
            pushed_at: None,
            html_url: format!("https://github.com/{}/{}", owner, name),
            starred_order: 0,
        }
    }

    #[test]
    fn test_app_screen_default() {
        let screen = AppScreen::default();
        assert_eq!(screen, AppScreen::Setup);
    }

    #[test]
    fn test_sort_field_label() {
        assert_eq!(SortField::Starred.label(), "Starred");
        assert_eq!(SortField::Pushed.label(), "Pushed");
    }

    #[test]
    fn test_sort_field_api_value() {
        assert_eq!(SortField::Starred.api_value(), "created");
        assert_eq!(SortField::Pushed.api_value(), "updated");
    }

    #[test]
    fn test_sort_field_all() {
        let all = SortField::all();
        assert_eq!(all.len(), 2);
        assert!(all.contains(&SortField::Starred));
        assert!(all.contains(&SortField::Pushed));
    }

    #[test]
    fn test_sort_direction_toggle() {
        assert_eq!(SortDirection::Asc.toggle(), SortDirection::Desc);
        assert_eq!(SortDirection::Desc.toggle(), SortDirection::Asc);
    }

    #[test]
    fn test_sort_direction_label() {
        assert_eq!(SortDirection::Asc.label(), "↑");
        assert_eq!(SortDirection::Desc.label(), "↓");
    }

    #[test]
    fn test_sort_direction_api_value() {
        assert_eq!(SortDirection::Asc.api_value(), "asc");
        assert_eq!(SortDirection::Desc.api_value(), "desc");
    }

    #[test]
    fn test_app_state_default() {
        let state = AppState::default();
        assert_eq!(state.screen, AppScreen::Setup);
        assert!(state.repositories.is_empty());
        assert_eq!(state.selection.count(), 0);
        assert!(!state.loading);
        assert!(state.error.is_none());
        assert!(state.username.is_none());
        assert!(state.pending_action.is_none());
        assert_eq!(state.sort_field, SortField::Pushed);
        assert_eq!(state.sort_direction, SortDirection::Asc);
    }

    #[test]
    fn test_from_config_with_token() {
        let config = AppConfig {
            github: GitHubConfig {
                personal_access_token: Some("valid_token".to_string()),
            },
        };

        let state = AppState::from_config(config);
        assert_eq!(state.screen, AppScreen::Loading);
        assert_eq!(state.current_page, 1);
        assert!(state.has_more);
    }

    #[test]
    fn test_from_config_without_token() {
        let config = AppConfig {
            github: GitHubConfig {
                personal_access_token: None,
            },
        };

        let state = AppState::from_config(config);
        assert_eq!(state.screen, AppScreen::Setup);
    }

    #[test]
    fn test_from_config_with_empty_token() {
        let config = AppConfig {
            github: GitHubConfig {
                personal_access_token: Some("".to_string()),
            },
        };

        let state = AppState::from_config(config);
        assert_eq!(state.screen, AppScreen::Setup);
    }

    #[test]
    fn test_get_selected_repos() {
        let mut state = AppState::default();
        state.repositories = vec![
            create_test_repo(1, "repo1", "owner1"),
            create_test_repo(2, "repo2", "owner2"),
            create_test_repo(3, "repo3", "owner3"),
        ];

        state.selection.toggle(1);
        state.selection.toggle(3);

        let selected = state.get_selected_repos();
        assert_eq!(selected.len(), 2);
        assert!(selected.contains(&("owner1".to_string(), "repo1".to_string())));
        assert!(selected.contains(&("owner3".to_string(), "repo3".to_string())));
    }

    #[test]
    fn test_get_selected_ids() {
        let mut state = AppState::default();
        state.repositories = vec![
            create_test_repo(1, "repo1", "owner1"),
            create_test_repo(2, "repo2", "owner2"),
        ];

        state.selection.toggle(1);
        state.selection.toggle(2);

        let ids = state.get_selected_ids();
        assert_eq!(ids.len(), 2);
        assert!(ids.contains(&1));
        assert!(ids.contains(&2));
    }

    #[test]
    fn test_remove_repos() {
        let mut state = AppState::default();
        state.repositories = vec![
            create_test_repo(1, "repo1", "owner1"),
            create_test_repo(2, "repo2", "owner2"),
            create_test_repo(3, "repo3", "owner3"),
        ];

        state.selection.toggle(1);
        state.selection.toggle(2);
        state.selection.toggle(3);

        state.remove_repos(&[1, 3]);

        assert_eq!(state.repositories.len(), 1);
        assert_eq!(state.repositories[0].id, 2);
        assert_eq!(state.selection.count(), 1);
        assert!(state.selection.is_selected(2));
    }

    #[test]
    fn test_clear_error() {
        let mut state = AppState::default();
        state.error = Some("Test error".to_string());

        state.clear_error();
        assert!(state.error.is_none());
    }

    #[test]
    fn test_set_error() {
        let mut state = AppState::default();

        state.set_error("Something went wrong".to_string());
        assert_eq!(state.error, Some("Something went wrong".to_string()));
    }

    #[test]
    fn test_pending_action_variants() {
        let single = PendingAction::UnstarSingle(1, "owner".to_string(), "repo".to_string(), "owner/repo".to_string());
        let selected = PendingAction::UnstarSelected(5);
        let logout = PendingAction::Logout;

        // Just verify we can create these variants
        match single {
            PendingAction::UnstarSingle(id, owner, name, full_name) => {
                assert_eq!(id, 1);
                assert_eq!(owner, "owner");
                assert_eq!(name, "repo");
                assert_eq!(full_name, "owner/repo");
            }
            _ => panic!("Expected UnstarSingle"),
        }

        match selected {
            PendingAction::UnstarSelected(count) => assert_eq!(count, 5),
            _ => panic!("Expected UnstarSelected"),
        }

        match logout {
            PendingAction::Logout => {}
            _ => panic!("Expected Logout"),
        }
    }
}
