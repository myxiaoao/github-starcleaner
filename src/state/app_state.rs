use crate::models::{AppConfig, Repository, RepositorySelection};
use crate::services::{ConfigService, GitHubService};
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
}

impl Global for AppState {}
