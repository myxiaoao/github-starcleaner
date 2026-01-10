use crate::services::is_token_expired_error;
use crate::state::{AppState, PendingAction, SortDirection, SortField};
use crate::ui::{catppuccin, render_repository_row};
use gpui::prelude::FluentBuilder;
use gpui::*;

pub struct RepositoryListView;

impl RepositoryListView {
    pub fn new(cx: &mut Context<Self>) -> Self {
        // Observe global state changes to refresh UI
        cx.observe_global::<AppState>(|_this, cx| {
            tracing::info!("Global state changed, notifying view");
            cx.notify();
        }).detach();

        Self
    }
}

impl Render for RepositoryListView {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        // Clone all needed data upfront to avoid borrow issues
        let (
            selection_count,
            total_count,
            all_selected,
            username,
            pending_action,
            has_more,
            loading_more,
            loading,
            sort_field,
            sort_direction,
            repos_for_render,
        ) = {
            let state = cx.global::<AppState>();
            let repos = &state.repositories;
            let selection_count = state.selection.count();
            let total_count = repos.len();
            let all_selected = selection_count == total_count && total_count > 0;

            let repos_for_render: Vec<_> = repos
                .iter()
                .map(|r| {
                    let is_selected = state.selection.is_selected(r.id);
                    (r.clone(), is_selected)
                })
                .collect();

            (
                selection_count,
                total_count,
                all_selected,
                state.username.clone().unwrap_or_default(),
                state.pending_action.clone(),
                state.has_more,
                state.loading_more,
                state.loading,
                state.sort_field,
                state.sort_direction,
                repos_for_render,
            )
        };

        div()
            .size_full()
            .relative()
            .flex()
            .flex_col()
            .bg(rgb(catppuccin::BASE))
            // Header
            .child(
                div()
                    .w_full()
                    .px_4()
                    .py_3()
                    .flex()
                    .items_center()
                    .gap_4()
                    .border_b_1()
                    .border_color(rgb(catppuccin::SURFACE1))
                    .bg(rgb(catppuccin::MANTLE))
                    // Title
                    .child(
                        div()
                            .flex()
                            .items_center()
                            .gap_2()
                            .child(
                                div()
                                    .text_lg()
                                    .font_weight(FontWeight::BOLD)
                                    .text_color(rgb(catppuccin::TEXT))
                                    .child(format!("Starred Repositories ({})", total_count)),
                            )
                            .when(!username.is_empty(), |this| {
                                this.child(
                                    div()
                                        .text_sm()
                                        .text_color(rgb(catppuccin::OVERLAY0))
                                        .child(format!("@{}", username)),
                                )
                            }),
                    )
                    // Spacer
                    .child(div().flex_1())
                    // Unstar Selected button
                    .when(selection_count > 0, |this| {
                        let count = selection_count;
                        this.child(
                            div()
                                .id("unstar-selected-btn")
                                .px_4()
                                .py_2()
                                .rounded_md()
                                .bg(rgb(catppuccin::RED))
                                .text_sm()
                                .text_color(rgb(catppuccin::BASE))
                                .font_weight(FontWeight::MEDIUM)
                                .cursor_pointer()
                                .hover(|style| style.opacity(0.9))
                                .child(format!("Unstar Selected ({})", selection_count))
                                .on_click(cx.listener(move |_this, _event, _window, cx| {
                                    cx.update_global::<AppState, _>(|state, _cx| {
                                        state.pending_action = Some(PendingAction::UnstarSelected(count));
                                    });
                                })),
                        )
                    })
                    // Logout button
                    .child(
                        div()
                            .id("logout-btn")
                            .px_3()
                            .py_2()
                            .rounded_md()
                            .bg(rgb(catppuccin::SURFACE1))
                            .text_sm()
                            .text_color(rgb(catppuccin::SUBTEXT0))
                            .cursor_pointer()
                            .hover(|style| style.bg(rgb(catppuccin::SURFACE2)))
                            .child("Logout")
                            .on_click(cx.listener(|_this, _event, _window, cx| {
                                cx.update_global::<AppState, _>(|state, _cx| {
                                    state.pending_action = Some(PendingAction::Logout);
                                });
                            })),
                    ),
            )
            // Toolbar
            .child(
                div()
                    .w_full()
                    .px_4()
                    .py_2()
                    .flex()
                    .items_center()
                    .gap_4()
                    .border_b_1()
                    .border_color(rgb(catppuccin::SURFACE1))
                    .bg(rgb(catppuccin::SURFACE0))
                    // Select All checkbox
                    .child(
                        div()
                            .id("select-all-checkbox")
                            .flex()
                            .items_center()
                            .gap_2()
                            .cursor_pointer()
                            .child(
                                div()
                                    .w(px(18.))
                                    .h(px(18.))
                                    .flex()
                                    .items_center()
                                    .justify_center()
                                    .rounded_sm()
                                    .border_1()
                                    .border_color(if all_selected {
                                        rgb(catppuccin::BLUE)
                                    } else {
                                        rgb(catppuccin::SURFACE1)
                                    })
                                    .bg(if all_selected {
                                        rgb(catppuccin::BLUE)
                                    } else {
                                        rgb(catppuccin::BASE)
                                    })
                                    .child(if all_selected {
                                        div().text_xs().text_color(rgb(catppuccin::BASE)).child("✓")
                                    } else {
                                        div()
                                    }),
                            )
                            .child(
                                div()
                                    .text_sm()
                                    .text_color(rgb(catppuccin::TEXT))
                                    .child("Select All"),
                            )
                            .on_click(cx.listener(|this, _event, _window, cx| {
                                this.toggle_select_all(cx);
                            })),
                    )
                    // Sort controls
                    .child(
                        div()
                            .flex()
                            .items_center()
                            .gap_2()
                            .child(
                                div()
                                    .text_sm()
                                    .text_color(rgb(catppuccin::OVERLAY0))
                                    .child("Sort:"),
                            )
                            // Sort field buttons
                            .children(SortField::all().iter().map(|field| {
                                let is_active = *field == sort_field;
                                let field_copy = *field;
                                div()
                                    .id(ElementId::Name(format!("sort-{}", field.label()).into()))
                                    .px_2()
                                    .py_1()
                                    .rounded_sm()
                                    .text_xs()
                                    .cursor_pointer()
                                    .bg(if is_active {
                                        rgb(catppuccin::BLUE)
                                    } else {
                                        rgb(catppuccin::SURFACE1)
                                    })
                                    .text_color(if is_active {
                                        rgb(catppuccin::BASE)
                                    } else {
                                        rgb(catppuccin::SUBTEXT0)
                                    })
                                    .hover(|style| {
                                        if is_active {
                                            style
                                        } else {
                                            style.bg(rgb(catppuccin::SURFACE2))
                                        }
                                    })
                                    .child(field.label())
                                    .on_click(cx.listener(move |this, _event, _window, cx| {
                                        let needs_reload = cx.update_global::<AppState, _>(|state, _cx| {
                                            if state.sort_field == field_copy {
                                                // Toggle direction if same field
                                                state.sort_direction = state.sort_direction.toggle();
                                            } else {
                                                // Change field, reset to ascending
                                                state.sort_field = field_copy;
                                                state.sort_direction = SortDirection::Asc;
                                            }
                                            true
                                        });
                                        if needs_reload {
                                            this.reload_repos(cx);
                                        }
                                    }))
                            }))
                            // Direction indicator
                            .child(
                                div()
                                    .id("sort-direction")
                                    .px_2()
                                    .py_1()
                                    .rounded_sm()
                                    .text_xs()
                                    .cursor_pointer()
                                    .bg(rgb(catppuccin::SURFACE1))
                                    .text_color(rgb(catppuccin::TEXT))
                                    .hover(|style| style.bg(rgb(catppuccin::SURFACE2)))
                                    .child(sort_direction.label())
                                    .on_click(cx.listener(|this, _event, _window, cx| {
                                        cx.update_global::<AppState, _>(|state, _cx| {
                                            state.sort_direction = state.sort_direction.toggle();
                                        });
                                        this.reload_repos(cx);
                                    })),
                            ),
                    )
                    // Spacer
                    .child(div().flex_1())
                    // Filter info
                    .child(
                        div()
                            .text_sm()
                            .text_color(rgb(catppuccin::OVERLAY0))
                            .child(format!("{} repositories", total_count)),
                    ),
            )
            // Repository list
            .child(
                div()
                    .id("repo-list-scroll")
                    .flex_1()
                    .overflow_y_scroll()
                    .child(if loading {
                        // Loading indicator
                        div()
                            .size_full()
                            .flex()
                            .items_center()
                            .justify_center()
                            .py_8()
                            .child(
                                div()
                                    .text_color(rgb(catppuccin::OVERLAY0))
                                    .child("Loading...")
                            )
                            .into_any_element()
                    } else {
                        div()
                            .flex()
                            .flex_col()
                            .children(
                                repos_for_render
                                    .into_iter()
                                    .map(|(repo, is_selected)| {
                                        let owner = repo.owner.clone();
                                        let name = repo.name.clone();
                                        let full_name = repo.full_name.clone();
                                        render_repository_row(repo, is_selected, move |repo_id, cx| {
                                            cx.update_global::<AppState, _>(|state, _cx| {
                                                state.pending_action = Some(PendingAction::UnstarSingle(
                                                    repo_id,
                                                    owner.clone(),
                                                    name.clone(),
                                                    full_name.clone(),
                                                ));
                                            });
                                        })
                                    }),
                            )
                            // Load More button
                            .when(has_more, |this| {
                                this.child(
                                    div()
                                        .w_full()
                                        .py_4()
                                        .flex()
                                        .justify_center()
                                        .child(
                                            div()
                                                .id("load-more-btn")
                                                .px_6()
                                                .py_2()
                                                .rounded_md()
                                                .bg(if loading_more {
                                                    rgb(catppuccin::SURFACE1)
                                                } else {
                                                    rgb(catppuccin::BLUE)
                                                })
                                                .text_sm()
                                                .text_color(rgb(catppuccin::BASE))
                                                .font_weight(FontWeight::MEDIUM)
                                                .cursor_pointer()
                                                .when(!loading_more, |this| {
                                                    this.hover(|style| style.bg(rgb(catppuccin::SAPPHIRE)))
                                                })
                                                .child(if loading_more {
                                                    "Loading..."
                                                } else {
                                                    "Load More"
                                                })
                                                .when(!loading_more, |this| {
                                                    this.on_click(cx.listener(|this, _event, _window, cx| {
                                                        this.load_more(cx);
                                                    }))
                                                }),
                                        ),
                                )
                            })
                            .into_any_element()
                    }),
            )
            // Confirmation dialog overlay - must be last child to be on top
            .when_some(pending_action, |this, action| {
                this.child(Self::render_confirmation_dialog(action, cx))
            })
    }
}

impl RepositoryListView {
    /// Reload repositories from page 1 with current sort options
    fn reload_repos(&mut self, cx: &mut Context<Self>) {
        // Check if already loading
        let is_loading = {
            let state = cx.global::<AppState>();
            state.loading || state.loading_more
        };

        if is_loading {
            return;
        }

        cx.update_global::<AppState, _>(|state, _cx| {
            state.loading = true;
            state.repositories.clear();
            state.selection.clear();
            state.current_page = 1;
            state.has_more = true;
        });
        cx.notify();

        cx.spawn(async move |_view, cx| {
            let (service, sort_field, sort_direction) = {
                let result = cx.update(|cx| {
                    let state = cx.global::<AppState>();
                    (state.github_service.clone(), state.sort_field, state.sort_direction)
                });
                match result {
                    Ok(v) => v,
                    Err(_) => return,
                }
            };

            if let Some(service) = service {
                let result = service
                    .fetch_starred_repos_page(1, 100, sort_field.api_value(), sort_direction.api_value())
                    .await;

                cx.update(|cx| {
                    let state = cx.global_mut::<AppState>();
                    state.loading = false;
                    match result {
                        Ok((repos, has_more)) => {
                            state.repositories = repos;
                            state.current_page = 1;
                            state.has_more = has_more;
                        }
                        Err(e) => {
                            state.handle_api_error(e, "Failed to reload");
                        }
                    }
                })
                .ok();
            }
        })
        .detach();
    }

    fn load_more(&mut self, cx: &mut Context<Self>) {
        // Check if already loading
        let can_load = {
            let state = cx.global::<AppState>();
            !state.loading_more && state.has_more
        };

        if !can_load {
            return;
        }

        cx.update_global::<AppState, _>(|state, _cx| {
            state.loading_more = true;
        });
        cx.notify();

        cx.spawn(async move |_view, cx| {
            let (service, next_page, sort_field, sort_direction) = {
                let result = cx.update(|cx| {
                    let state = cx.global::<AppState>();
                    (
                        state.github_service.clone(),
                        state.current_page + 1,
                        state.sort_field,
                        state.sort_direction,
                    )
                });
                match result {
                    Ok(v) => v,
                    Err(_) => return,
                }
            };

            if let Some(service) = service {
                let result = service
                    .fetch_starred_repos_page(next_page, 100, sort_field.api_value(), sort_direction.api_value())
                    .await;

                cx.update(|cx| {
                    let state = cx.global_mut::<AppState>();
                    state.loading_more = false;
                    match result {
                        Ok((repos, has_more)) => {
                            state.repositories.extend(repos);
                            state.current_page = next_page;
                            state.has_more = has_more;
                        }
                        Err(e) => {
                            state.handle_api_error(e, "Failed to load more");
                        }
                    }
                })
                .ok();
            }
        })
        .detach();
    }

    fn toggle_select_all(&mut self, cx: &mut Context<Self>) {
        cx.update_global::<AppState, _>(|state, _cx| {
            if state.selection.count() == state.repositories.len() {
                state.selection.clear();
            } else {
                state.selection.select_all(&state.repositories);
            }
        });
        cx.notify();
    }

    fn unstar_selected(&mut self, cx: &mut Context<Self>) {
        let (repos_to_unstar, ids_to_remove): (Vec<_>, Vec<_>) = {
            let state = cx.global::<AppState>();
            state
                .repositories
                .iter()
                .filter(|r| state.selection.is_selected(r.id))
                .map(|r| ((r.owner.clone(), r.name.clone()), r.id))
                .unzip()
        };

        if repos_to_unstar.is_empty() {
            return;
        }

        cx.spawn(async move |_view, cx| {
            let service = cx
                .update(|cx| cx.global::<AppState>().github_service.clone())
                .ok()
                .flatten();

            if let Some(service) = service {
                let results = service.unstar_repos(&repos_to_unstar).await;

                // Check for token expiration
                let token_expired = results
                    .iter()
                    .any(|(_, _, result)| result.as_ref().err().map(is_token_expired_error).unwrap_or(false));

                if token_expired {
                    cx.update(|cx| {
                        let state = cx.global_mut::<AppState>();
                        let _ = state.logout();
                        state.error = Some("Token expired. Please login again.".to_string());
                    })
                    .ok();
                    return;
                }

                let success_ids: Vec<u64> = results
                    .iter()
                    .zip(ids_to_remove.iter())
                    .filter(|((_, _, result), _)| result.is_ok())
                    .map(|(_, id)| *id)
                    .collect();

                cx.update(|cx| {
                    let state = cx.global_mut::<AppState>();
                    state.remove_repos(&success_ids);
                })
                .ok();
            }
        })
        .detach();
    }

    fn logout(&mut self, cx: &mut Context<Self>) {
        cx.update_global::<AppState, _>(|state, _cx| {
            let _ = state.logout();
        });
        cx.notify();
    }

    fn render_confirmation_dialog(action: PendingAction, cx: &mut Context<Self>) -> impl IntoElement {
        let (title, message) = match &action {
            PendingAction::UnstarSingle(_, _, _, full_name) => (
                "Confirm Unstar".to_string(),
                format!("Are you sure you want to unstar '{}'?", full_name),
            ),
            PendingAction::UnstarSelected(count) => (
                "Confirm Unstar".to_string(),
                format!("Are you sure you want to unstar {} repositories?", count),
            ),
            PendingAction::Logout => (
                "Confirm Logout".to_string(),
                "Are you sure you want to logout?".to_string(),
            ),
        };

        let action_clone = action.clone();

        // Full-screen overlay
        div()
            .id("confirmation-overlay")
            .absolute()
            .inset_0()
            .flex()
            .items_center()
            .justify_center()
            // Semi-transparent backdrop
            .child(
                div()
                    .id("confirmation-backdrop")
                    .absolute()
                    .inset_0()
                    .bg(rgba(0x00000099))
                    .on_click(cx.listener(|_this, _event, _window, cx| {
                        cx.update_global::<AppState, _>(|state, _cx| {
                            state.pending_action = None;
                        });
                    })),
            )
            // Dialog box
            .child(
                div()
                    .w(px(400.))
                    .p_6()
                    .rounded_lg()
                    .bg(rgb(catppuccin::SURFACE0))
                    .border_1()
                    .border_color(rgb(catppuccin::SURFACE1))
                    .flex()
                    .flex_col()
                    .gap_4()
                    // Title
                    .child(
                        div()
                            .text_lg()
                            .font_weight(FontWeight::BOLD)
                            .text_color(rgb(catppuccin::TEXT))
                            .child(title),
                    )
                    // Message
                    .child(
                        div()
                            .text_sm()
                            .text_color(rgb(catppuccin::SUBTEXT0))
                            .child(message),
                    )
                    // Buttons
                    .child(
                        div()
                            .flex()
                            .gap_3()
                            .justify_end()
                            .mt_2()
                            // Cancel button
                            .child(
                                div()
                                    .id("cancel-btn")
                                    .px_4()
                                    .py_2()
                                    .rounded_md()
                                    .bg(rgb(catppuccin::SURFACE1))
                                    .text_sm()
                                    .text_color(rgb(catppuccin::TEXT))
                                    .cursor_pointer()
                                    .hover(|style| style.bg(rgb(catppuccin::SURFACE2)))
                                    .child("Cancel")
                                    .on_click(cx.listener(|_this, _event, _window, cx| {
                                        cx.update_global::<AppState, _>(|state, _cx| {
                                            state.pending_action = None;
                                        });
                                    })),
                            )
                            // Confirm button
                            .child(
                                div()
                                    .id("confirm-btn")
                                    .px_4()
                                    .py_2()
                                    .rounded_md()
                                    .bg(rgb(catppuccin::RED))
                                    .text_sm()
                                    .text_color(rgb(catppuccin::BASE))
                                    .font_weight(FontWeight::MEDIUM)
                                    .cursor_pointer()
                                    .hover(|style| style.opacity(0.9))
                                    .child("Confirm")
                                    .on_click(cx.listener(move |this, _event, _window, cx| {
                                        this.execute_action(action_clone.clone(), cx);
                                    })),
                            ),
                    ),
            )
    }

    fn execute_action(&mut self, action: PendingAction, cx: &mut Context<Self>) {
        // Clear pending action first
        cx.update_global::<AppState, _>(|state, _cx| {
            state.pending_action = None;
        });

        match action {
            PendingAction::UnstarSingle(repo_id, owner, name, _) => {
                Self::do_unstar_repo(repo_id, owner, name, cx);
            }
            PendingAction::UnstarSelected(_) => {
                self.unstar_selected(cx);
            }
            PendingAction::Logout => {
                self.logout(cx);
            }
        }
    }

    fn do_unstar_repo(repo_id: u64, owner: String, name: String, cx: &mut Context<Self>) {
        cx.spawn(async move |_view, cx| {
            let service = cx
                .update(|cx| cx.global::<AppState>().github_service.clone())
                .ok()
                .flatten();

            if let Some(service) = service {
                match service.unstar_repo(&owner, &name).await {
                    Ok(_) => {
                        cx.update(|cx| {
                            let state = cx.global_mut::<AppState>();
                            state.remove_repos(&[repo_id]);
                        }).ok();
                    }
                    Err(e) => {
                        tracing::error!("Unstar API error: {}", e);
                        cx.update(|cx| {
                            let state = cx.global_mut::<AppState>();
                            state.handle_api_error(e, "Failed to unstar");
                        }).ok();
                    }
                }
            }
        })
        .detach();
    }
}
