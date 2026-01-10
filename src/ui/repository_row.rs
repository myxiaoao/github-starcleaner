use crate::models::Repository;
use crate::state::AppState;
use crate::ui::catppuccin;
use gpui::prelude::FluentBuilder;
use gpui::*;

pub fn render_repository_row(
    repo: Repository,
    is_selected: bool,
    on_unstar: impl Fn(u64, &mut App) + 'static,
) -> impl IntoElement {
    let Repository {
        id: repo_id,
        full_name,
        html_url,
        description,
        language,
        stargazers_count,
        forks_count,
        open_issues_count,
        license,
        topics,
        updated_at,
        pushed_at,
        ..
    } = repo;

    let updated_at = updated_at.format("%Y-%m-%d").to_string();
    let pushed_at = pushed_at.map(|dt| dt.format("%Y-%m-%d").to_string());

    div()
        .id(ElementId::Name(format!("repo-row-{}", repo_id).into()))
        .w_full()
        .px_4()
        .py_3()
        .flex()
        .gap_3()
        .items_start() // Align children to top
        .border_b_1()
        .border_color(rgb(catppuccin::SURFACE1))
        .hover(|style| style.bg(rgb(catppuccin::SURFACE0)))
        // Checkbox - fixed width, aligned to top
        .child(
            div()
                .id(ElementId::Name(format!("checkbox-{}", repo_id).into()))
                .flex_shrink_0()
                .w(px(20.))
                .h(px(20.))
                .mt(px(2.))
                .flex()
                .items_center()
                .justify_center()
                .rounded_sm()
                .border_1()
                .border_color(if is_selected {
                    rgb(catppuccin::BLUE)
                } else {
                    rgb(catppuccin::SURFACE1)
                })
                .bg(if is_selected {
                    rgb(catppuccin::BLUE)
                } else {
                    rgb(catppuccin::BASE)
                })
                .cursor_pointer()
                .child(if is_selected {
                    div().text_sm().text_color(rgb(catppuccin::BASE)).child("✓")
                } else {
                    div()
                })
                .on_click(move |_event, _window, cx| {
                    cx.update_global::<AppState, _>(|state, _cx| {
                        state.selection.toggle(repo_id);
                    });
                }),
        )
        // Middle: content area (flexible, will shrink)
        .child(
            div()
                .flex_1()
                .min_w(px(100.)) // Minimum width to prevent complete collapse
                .overflow_hidden()
                .flex()
                .flex_col()
                .gap_1()
                // First row: Name + Language
                .child(
                    div()
                        .flex()
                        .items_center()
                        .gap_3()
                        .overflow_hidden()
                        // Name
                        .child(
                            div()
                                .id(ElementId::Name(format!("repo-name-{}", repo_id).into()))
                                .overflow_hidden()
                                .whitespace_nowrap()
                                .text_base()
                                .font_weight(FontWeight::SEMIBOLD)
                                .text_color(rgb(catppuccin::BLUE))
                                .cursor_pointer()
                                .hover(|style| style.underline())
                                .child(full_name)
                                .on_click({
                                    let url = html_url.clone();
                                    move |_event, _window, _cx| {
                                        let _ = open::that(&url);
                                    }
                                }),
                        )
                        // Language tag
                        .when_some(language, |this, lang| {
                            this.child(
                                div()
                                    .flex_shrink_0()
                                    .px_2()
                                    .py(px(2.))
                                    .rounded_sm()
                                    .bg(rgb(catppuccin::SURFACE1))
                                    .text_xs()
                                    .text_color(rgb(catppuccin::SUBTEXT0))
                                    .child(lang),
                            )
                        }),
                )
                // Description
                .when_some(description, |this, desc| {
                    let truncated = if desc.chars().count() > 100 {
                        format!("{}...", desc.chars().take(100).collect::<String>())
                    } else {
                        desc
                    };
                    this.child(
                        div()
                            .text_sm()
                            .text_color(rgb(catppuccin::SUBTEXT0))
                            .overflow_hidden()
                            .whitespace_nowrap()
                            .child(truncated),
                    )
                })
                // Stats row
                .child(
                    div()
                        .flex()
                        .gap_4()
                        .text_xs()
                        .text_color(rgb(catppuccin::OVERLAY0))
                        .child(format!("★ {}", stargazers_count))
                        .child(format!("⑂ {}", forks_count))
                        .child(format!("⚠ {}", open_issues_count))
                        .when_some(license, |this, lic| this.child(lic))
                        .when_some(pushed_at, |this, pushed| this.child(format!("Pushed: {}", pushed)))
                        .child(format!("Updated: {}", updated_at)),
                )
                // Topics
                .when(!topics.is_empty(), |this| {
                    this.child(
                        div()
                            .flex()
                            .gap_2()
                            .flex_wrap()
                            .mt_1()
                            .children(topics.iter().take(5).map(|topic| {
                                div()
                                    .px_2()
                                    .py(px(2.))
                                    .rounded_full()
                                    .bg(rgb(catppuccin::SURFACE0))
                                    .text_xs()
                                    .text_color(rgb(catppuccin::SUBTEXT0))
                                    .child(topic.clone())
                            })),
                    )
                }),
        )
        // Right: Unstar button (fixed width, top aligned)
        .child(
            div()
                .id(ElementId::Name(format!("unstar-btn-{}", repo_id).into()))
                .flex_shrink_0()
                .whitespace_nowrap()
                .px_3()
                .py_1()
                .h_auto()
                .rounded_md()
                .bg(rgb(catppuccin::SURFACE1))
                .text_xs()
                .text_color(rgb(catppuccin::RED))
                .cursor_pointer()
                .hover(|style| style.bg(rgb(catppuccin::SURFACE2)))
                .child("Unstar")
                .on_click(move |_event, _window, cx| {
                    on_unstar(repo_id, cx);
                }),
        )
}
