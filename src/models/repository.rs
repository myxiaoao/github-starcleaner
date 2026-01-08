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

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Utc;

    fn create_test_repo(id: u64, name: &str) -> Repository {
        Repository {
            id,
            name: name.to_string(),
            full_name: format!("owner/{}", name),
            owner: "owner".to_string(),
            description: Some("Test description".to_string()),
            language: Some("Rust".to_string()),
            stargazers_count: 100,
            forks_count: 10,
            open_issues_count: 5,
            license: Some("MIT".to_string()),
            topics: vec!["rust".to_string(), "cli".to_string()],
            updated_at: Utc::now(),
            pushed_at: Some(Utc::now()),
            html_url: format!("https://github.com/owner/{}", name),
            starred_order: 0,
        }
    }

    #[test]
    fn test_repository_selection_new() {
        let selection = RepositorySelection::new();
        assert_eq!(selection.count(), 0);
    }

    #[test]
    fn test_repository_selection_toggle() {
        let mut selection = RepositorySelection::new();

        // Toggle on
        selection.toggle(1);
        assert!(selection.is_selected(1));
        assert_eq!(selection.count(), 1);

        // Toggle off
        selection.toggle(1);
        assert!(!selection.is_selected(1));
        assert_eq!(selection.count(), 0);
    }

    #[test]
    fn test_repository_selection_multiple() {
        let mut selection = RepositorySelection::new();

        selection.toggle(1);
        selection.toggle(2);
        selection.toggle(3);

        assert!(selection.is_selected(1));
        assert!(selection.is_selected(2));
        assert!(selection.is_selected(3));
        assert!(!selection.is_selected(4));
        assert_eq!(selection.count(), 3);
    }

    #[test]
    fn test_repository_selection_select_all() {
        let mut selection = RepositorySelection::new();
        let repos = vec![
            create_test_repo(1, "repo1"),
            create_test_repo(2, "repo2"),
            create_test_repo(3, "repo3"),
        ];

        selection.select_all(&repos);

        assert_eq!(selection.count(), 3);
        assert!(selection.is_selected(1));
        assert!(selection.is_selected(2));
        assert!(selection.is_selected(3));
    }

    #[test]
    fn test_repository_selection_clear() {
        let mut selection = RepositorySelection::new();

        selection.toggle(1);
        selection.toggle(2);
        assert_eq!(selection.count(), 2);

        selection.clear();
        assert_eq!(selection.count(), 0);
        assert!(!selection.is_selected(1));
        assert!(!selection.is_selected(2));
    }

    #[test]
    fn test_repository_selection_remove_ids() {
        let mut selection = RepositorySelection::new();

        selection.toggle(1);
        selection.toggle(2);
        selection.toggle(3);
        selection.toggle(4);

        selection.remove_ids(&[2, 4]);

        assert_eq!(selection.count(), 2);
        assert!(selection.is_selected(1));
        assert!(!selection.is_selected(2));
        assert!(selection.is_selected(3));
        assert!(!selection.is_selected(4));
    }

    #[test]
    fn test_repository_serialization() {
        let repo = create_test_repo(123, "test-repo");
        let json = serde_json::to_string(&repo).unwrap();

        assert!(json.contains("\"id\":123"));
        assert!(json.contains("\"name\":\"test-repo\""));
        assert!(json.contains("\"owner\":\"owner\""));
    }

    #[test]
    fn test_repository_deserialization() {
        let json = r#"{
            "id": 456,
            "name": "my-repo",
            "full_name": "user/my-repo",
            "owner": "user",
            "description": "A test repo",
            "language": "Python",
            "stargazers_count": 50,
            "forks_count": 5,
            "open_issues_count": 2,
            "license": "Apache-2.0",
            "topics": ["python", "test"],
            "updated_at": "2024-01-01T00:00:00Z",
            "pushed_at": "2024-01-01T00:00:00Z",
            "html_url": "https://github.com/user/my-repo",
            "starred_order": 10
        }"#;

        let repo: Repository = serde_json::from_str(json).unwrap();
        assert_eq!(repo.id, 456);
        assert_eq!(repo.name, "my-repo");
        assert_eq!(repo.owner, "user");
        assert_eq!(repo.language, Some("Python".to_string()));
        assert_eq!(repo.starred_order, 10);
    }

    #[test]
    fn test_repository_optional_fields() {
        let json = r#"{
            "id": 789,
            "name": "minimal-repo",
            "full_name": "user/minimal-repo",
            "owner": "user",
            "description": null,
            "language": null,
            "stargazers_count": 0,
            "forks_count": 0,
            "open_issues_count": 0,
            "license": null,
            "topics": [],
            "updated_at": "2024-01-01T00:00:00Z",
            "pushed_at": null,
            "html_url": "https://github.com/user/minimal-repo"
        }"#;

        let repo: Repository = serde_json::from_str(json).unwrap();
        assert_eq!(repo.id, 789);
        assert!(repo.description.is_none());
        assert!(repo.language.is_none());
        assert!(repo.license.is_none());
        assert!(repo.pushed_at.is_none());
        assert_eq!(repo.starred_order, 0); // default value
    }
}
