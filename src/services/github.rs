use crate::models::Repository;
use anyhow::{anyhow, Context, Result};
use octocrab::Octocrab;
use std::sync::OnceLock;
use tokio::runtime::Runtime;

/// Error indicating the token has expired or is invalid
#[derive(Debug, Clone)]
pub struct TokenExpiredError;

impl std::fmt::Display for TokenExpiredError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Token expired or invalid")
    }
}

impl std::error::Error for TokenExpiredError {}

// Global Tokio runtime for octocrab async operations
fn tokio_runtime() -> &'static Runtime {
    static RUNTIME: OnceLock<Runtime> = OnceLock::new();
    RUNTIME.get_or_init(|| {
        Runtime::new().expect("Failed to create Tokio runtime")
    })
}

#[derive(Clone)]
pub struct GitHubService {
    client: Octocrab,
}

impl GitHubService {
    /// Create new service with PAT
    pub fn new(token: &str) -> Result<Self> {
        // Octocrab needs Tokio runtime even for initialization
        let token = token.to_string();
        let client = tokio_runtime().block_on(async {
            Octocrab::builder()
                .personal_token(token)
                .build()
        }).context("Failed to build GitHub client")?;

        Ok(Self { client })
    }

    /// Validate token by fetching current user, returns (username, starred_count)
    pub async fn validate_token(&self) -> Result<(String, Option<u32>)> {
        let client = self.client.clone();
        let result = tokio_runtime().spawn(async move {
            client
                .current()
                .user()
                .await
                .context("Failed to validate token - please check your Personal Access Token")
        }).await.context("Task failed")??;

        // GitHub API doesn't directly return starred count in user object
        // We'll get the count from the first page response header
        Ok((result.login, None))
    }

    /// Get the total starred count from API
    pub async fn get_starred_count(&self) -> Result<u32> {
        let client = self.client.clone();
        let result = tokio_runtime().spawn(async move {
            // Fetch just 1 item to get the total from pagination
            let repos = client
                .current()
                .list_repos_starred_by_authenticated_user()
                .per_page(1)
                .page(1u8)
                .send()
                .await
                .context("Failed to get starred count")?;

            // The Page struct should have total_count or we count from all pages
            // Unfortunately octocrab doesn't expose Link headers easily
            // So we'll return 0 here and rely on fetched count
            Ok::<_, anyhow::Error>(repos.total_count.unwrap_or(0) as u32)
        }).await.context("Task failed")??;

        Ok(result)
    }

    /// Fetch a page of starred repositories with sort options
    pub async fn fetch_starred_repos_page(
        &self,
        page: u32,
        per_page: u8,
        sort: &str,
        direction: &str,
    ) -> Result<(Vec<Repository>, bool)> {
        let client = self.client.clone();
        let sort = sort.to_string();
        let direction = direction.to_string();
        let result = tokio_runtime().spawn(async move {
            let repos = client
                .current()
                .list_repos_starred_by_authenticated_user()
                .sort(&sort)
                .direction(&direction)
                .per_page(per_page)
                .page(page as u8)
                .send()
                .await
                .context("Failed to fetch starred repos")?;

            let items: Vec<_> = repos.items;
            let has_more = items.len() == per_page as usize;

            // Calculate base order: (page - 1) * per_page
            let base_order = ((page as u32) - 1) * (per_page as u32);
            let repos = items
                .into_iter()
                .enumerate()
                .map(|(i, repo)| Repository::from_octocrab_with_order(repo, base_order + (i as u32)))
                .collect();
            Ok::<_, anyhow::Error>((repos, has_more))
        }).await.context("Task failed")??;

        Ok(result)
    }

    /// Fetch all starred repositories (handles pagination) - for backward compatibility
    pub async fn fetch_starred_repos(&self) -> Result<Vec<Repository>> {
        let mut all_repos = Vec::new();
        let mut page = 1u32;

        loop {
            let (repos, has_more) = self.fetch_starred_repos_page(page, 100, "created", "desc").await?;

            if repos.is_empty() {
                break;
            }

            all_repos.extend(repos);

            if !has_more || page > 500 {
                break;
            }

            page += 1;
        }

        Ok(all_repos)
    }

    /// Unstar a single repository
    pub async fn unstar_repo(&self, owner: &str, repo: &str) -> Result<()> {
        let client = self.client.clone();
        let owner = owner.to_string();
        let repo = repo.to_string();
        let owner_for_err = owner.clone();
        let repo_for_err = repo.clone();

        let result: Result<u16, octocrab::Error> = tokio_runtime().spawn(async move {
            // GitHub returns 204 No Content on success, so we use _delete which returns raw response
            let url = format!("https://api.github.com/user/starred/{}/{}", owner, repo);
            let response = client._delete(url, None::<&()>).await?;
            Ok(response.status().as_u16())
        }).await.context("Task failed")?;

        match result {
            Ok(status) if status == 204 || status == 200 => Ok(()),
            Ok(401) => Err(anyhow!(TokenExpiredError)),
            Ok(status) => Err(anyhow!("Failed to unstar {}/{}: HTTP {}", owner_for_err, repo_for_err, status)),
            Err(e) => Err(anyhow!("Failed to unstar {}/{}: {}", owner_for_err, repo_for_err, e)),
        }
    }

    /// Unstar multiple repositories
    pub async fn unstar_repos(
        &self,
        repos: &[(String, String)],
    ) -> Vec<(String, String, Result<()>)> {
        let mut results = Vec::new();

        for (owner, repo) in repos {
            let result = self.unstar_repo(owner, repo).await;
            results.push((owner.clone(), repo.clone(), result));
        }

        results
    }
}
