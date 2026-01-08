use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashSet;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Repository {
    pub id: u64,
    pub name: String,
    pub full_name: String,
    pub owner: String,
    pub description: Option<String>,
    pub language: Option<String>,
    pub stargazers_count: u32,
    pub forks_count: u32,
    pub open_issues_count: u32,
    pub license: Option<String>,
    pub topics: Vec<String>,
    pub updated_at: DateTime<Utc>,
    pub pushed_at: Option<DateTime<Utc>>,
    pub html_url: String,
    /// Order in which the repo was starred (from API response order)
    #[serde(default)]
    pub starred_order: u32,
}

impl Repository {
    /// Convert from octocrab Repository model with starred order
    pub fn from_octocrab_with_order(repo: octocrab::models::Repository, starred_order: u32) -> Self {
        Self {
            id: repo.id.0,
            name: repo.name,
            full_name: repo.full_name.clone().unwrap_or_default(),
            owner: repo
                .owner
                .as_ref()
                .map(|o| o.login.clone())
                .unwrap_or_default(),
            description: repo.description.clone(),
            language: repo
                .language
                .as_ref()
                .and_then(|v| v.as_str().map(|s| s.to_string())),
            stargazers_count: repo.stargazers_count.unwrap_or(0) as u32,
            forks_count: repo.forks_count.unwrap_or(0) as u32,
            open_issues_count: repo.open_issues_count.unwrap_or(0) as u32,
            license: repo.license.as_ref().map(|l| l.name.clone()),
            topics: repo.topics.clone().unwrap_or_default(),
            updated_at: repo.updated_at.unwrap_or_else(Utc::now),
            pushed_at: repo.pushed_at,
            html_url: repo.html_url.map(|u| u.to_string()).unwrap_or_default(),
            starred_order,
        }
    }
}

/// Selection state for batch operations
#[derive(Debug, Clone, Default)]
pub struct RepositorySelection {
    pub selected_ids: HashSet<u64>,
}

impl RepositorySelection {
    pub fn new() -> Self {
        Self::default()
    }

    /// Toggle selection for a repository
    pub fn toggle(&mut self, id: u64) {
        if self.selected_ids.contains(&id) {
            self.selected_ids.remove(&id);
        } else {
            self.selected_ids.insert(id);
        }
    }

    /// Select all repositories
    pub fn select_all(&mut self, repos: &[Repository]) {
        self.selected_ids = repos.iter().map(|r| r.id).collect();
    }

    /// Clear all selections
    pub fn clear(&mut self) {
        self.selected_ids.clear();
    }

    /// Check if a repository is selected
    pub fn is_selected(&self, id: u64) -> bool {
        self.selected_ids.contains(&id)
    }

    /// Get the count of selected repositories
    pub fn count(&self) -> usize {
        self.selected_ids.len()
    }

    /// Remove specific IDs from selection
    pub fn remove_ids(&mut self, ids: &[u64]) {
        for id in ids {
            self.selected_ids.remove(id);
        }
    }
}
