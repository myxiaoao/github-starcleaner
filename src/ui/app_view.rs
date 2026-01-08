use crate::services::GitHubService;
use crate::state::{AppScreen, AppState, SortDirection, SortField};
use crate::ui::{RepositoryListView, SetupView};
use gpui::*;

pub struct AppView {
    setup_view: Entity<SetupView>,
    repo_list_view: Entity<RepositoryListView>,
}

impl AppView {
    pub fn new(cx: &mut Context<Self>) -> Self {
        let setup_view = cx.new(|cx| SetupView::new(cx));
        let repo_list_view = cx.new(|cx| RepositoryListView::new(cx));

        // If we have a token, trigger loading
        let state = cx.global::<AppState>();
        if state.screen == AppScreen::Loading {
            Self::trigger_load_repos(cx);
        }

        Self {
            setup_view,
            repo_list_view,
        }
    }

    fn trigger_load_repos(cx: &mut Context<Self>) {
        cx.spawn(async |_view, cx| {
            // Get token and sort options
            let (token, sort_field, sort_direction): (Option<String>, SortField, SortDirection) = cx
                .update(|cx| {
                    let state = cx.global::<AppState>();
                    (
                        state.config.github.personal_access_token.clone(),
                        state.sort_field,
                        state.sort_direction,
                    )
                })
                .unwrap_or((None, SortField::default(), SortDirection::default()));

            let Some(token) = token else {
                cx.update(|cx| {
                    cx.update_global::<AppState, _>(|state, _cx| {
                        state.screen = AppScreen::Setup;
                        state.error = Some("No token found".to_string());
                    });
                })
                .ok();
                return;
            };

            // Create service and validate, then load first page
            let result = async {
                let service = GitHubService::new(&token)?;
                let (username, _) = service.validate_token().await?;
                let (repos, has_more) = service
                    .fetch_starred_repos_page(1, 100, sort_field.api_value(), sort_direction.api_value())
                    .await?;
                Ok::<_, anyhow::Error>((service, username, repos, has_more))
            }
            .await;

            cx.update(|cx| {
                cx.update_global::<AppState, _>(|state, _cx| match result {
                    Ok((service, username, repos, has_more)) => {
                        state.github_service = Some(service);
                        state.username = Some(username);
                        state.repositories = repos;
                        state.loading = false;
                        state.current_page = 1;
                        state.has_more = has_more;
                        state.screen = AppScreen::RepositoryList;
                    }
                    Err(e) => {
                        state.error = Some(format!("Failed to load: {}", e));
                        state.screen = AppScreen::Setup;
                        state.loading = false;
                    }
                });
            })
            .ok();
        })
        .detach();
    }

    fn render_loading(&self) -> impl IntoElement {
        div()
            .size_full()
            .flex()
            .items_center()
            .justify_center()
            .bg(rgb(0x1e1e2e))
            .child(
                div()
                    .flex()
                    .flex_col()
                    .gap_4()
                    .items_center()
                    .child(
                        div()
                            .text_lg()
                            .text_color(rgb(0xcdd6f4))
                            .child("Loading your starred repositories..."),
                    )
                    .child(
                        div()
                            .text_sm()
                            .text_color(rgb(0x6c7086))
                            .child("This may take a moment if you have many stars."),
                    ),
            )
    }
}

impl Render for AppView {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let state = cx.global::<AppState>();
        let screen = state.screen.clone();

        // Check if we need to transition to loading
        if screen == AppScreen::Loading && !state.loading {
            cx.update_global::<AppState, _>(|state, _cx| {
                state.loading = true;
            });
            Self::trigger_load_repos(cx);
        }

        match screen {
            AppScreen::Setup => div().size_full().child(self.setup_view.clone()).into_any_element(),
            AppScreen::Loading => self.render_loading().into_any_element(),
            AppScreen::RepositoryList => div().size_full().child(self.repo_list_view.clone()).into_any_element(),
        }
    }
}
