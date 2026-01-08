use crate::services::{ConfigService, GitHubService};
use crate::state::{AppScreen, AppState};
use gpui::prelude::FluentBuilder;
use gpui::*;

pub struct SetupView {
    token_input: String,
    error: Option<String>,
    validating: bool,
    focus_handle: FocusHandle,
}

impl SetupView {
    pub fn new(cx: &mut Context<Self>) -> Self {
        Self {
            token_input: String::new(),
            error: None,
            validating: false,
            focus_handle: cx.focus_handle(),
        }
    }

    fn handle_key_down(&mut self, event: &KeyDownEvent, cx: &mut Context<Self>) {
        if self.validating {
            return;
        }

        let key = &event.keystroke.key;
        let key_char = &event.keystroke.key_char;

        // Handle backspace
        if key == "backspace" {
            self.token_input.pop();
            cx.notify();
            return;
        }

        // Handle Enter - submit the form
        if key == "enter" {
            self.submit_token(cx);
            return;
        }

        // Handle paste (Cmd+V / Ctrl+V)
        if (event.keystroke.modifiers.platform || event.keystroke.modifiers.control)
            && key == "v"
        {
            // Paste from clipboard
            if let Some(clipboard) = cx.read_from_clipboard() {
                if let Some(text) = clipboard.text() {
                    // Filter to only allow valid token characters
                    let filtered: String = text
                        .chars()
                        .filter(|c| c.is_ascii_alphanumeric() || *c == '_')
                        .collect();
                    self.token_input.push_str(&filtered);
                    cx.notify();
                }
            }
            return;
        }

        // Handle regular character input
        if let Some(ch) = key_char {
            // Only allow alphanumeric and underscore (valid for GitHub tokens)
            let filtered: String = ch
                .chars()
                .filter(|c| c.is_ascii_alphanumeric() || *c == '_')
                .collect();
            if !filtered.is_empty() {
                self.token_input.push_str(&filtered);
                cx.notify();
            }
        }
    }
}

impl Focusable for SetupView {
    fn focus_handle(&self, _cx: &App) -> FocusHandle {
        self.focus_handle.clone()
    }
}

impl Render for SetupView {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let error = self.error.clone();
        let validating = self.validating;
        let has_token = !self.token_input.is_empty();

        // Auto-focus on first render
        if !self.focus_handle.is_focused(window) {
            self.focus_handle.focus(window);
        }

        div()
            .id("setup-view")
            .size_full()
            .flex()
            .items_center()
            .justify_center()
            .bg(rgb(0x1e1e2e))
            .track_focus(&self.focus_handle)
            .on_key_down(cx.listener(|this, event, _window, cx| {
                this.handle_key_down(event, cx);
            }))
            .child(
                div()
                    .w(px(420.))
                    .p_8()
                    .bg(rgb(0x313244))
                    .rounded_lg()
                    .border_1()
                    .border_color(rgb(0x45475a))
                    .child(
                        div()
                            .flex()
                            .flex_col()
                            .gap_6()
                            // Title
                            .child(
                                div()
                                    .flex()
                                    .flex_col()
                                    .gap_2()
                                    .child(
                                        div()
                                            .text_xl()
                                            .font_weight(FontWeight::BOLD)
                                            .text_color(rgb(0xcdd6f4))
                                            .child("GitHub StarCleaner"),
                                    )
                                    .child(
                                        div()
                                            .text_sm()
                                            .text_color(rgb(0xa6adc8))
                                            .child("Enter your GitHub Personal Access Token to manage your starred repositories."),
                                    ),
                            )
                            // Input section
                            .child(
                                div()
                                    .flex()
                                    .flex_col()
                                    .gap_2()
                                    .child(
                                        div()
                                            .text_sm()
                                            .font_weight(FontWeight::MEDIUM)
                                            .text_color(rgb(0xcdd6f4))
                                            .child("Personal Access Token"),
                                    )
                                    .child(self.render_input(window, cx))
                                    .when_some(error, |this, err| {
                                        this.child(
                                            div()
                                                .text_sm()
                                                .text_color(rgb(0xf38ba8))
                                                .child(err),
                                        )
                                    }),
                            )
                            // Button
                            .child(self.render_button(validating, has_token, cx))
                            // Help text
                            .child(
                                div()
                                    .text_xs()
                                    .text_color(rgb(0x6c7086))
                                    .child("Token requires 'repo' or 'public_repo' scope for starring/unstarring."),
                            )
                            // Instructions
                            .child(
                                div()
                                    .text_xs()
                                    .text_color(rgb(0x6c7086))
                                    .mt_2()
                                    .child("Type your token or paste with Cmd+V. Press Enter to connect."),
                            ),
                    ),
            )
    }
}

impl SetupView {
    fn render_input(&self, window: &Window, _cx: &mut Context<Self>) -> impl IntoElement {
        let input_len = self.token_input.len();
        let is_focused = self.focus_handle.is_focused(window);
        let focus_handle = self.focus_handle.clone();

        div()
            .id("token-input")
            .w_full()
            .h(px(40.))
            .px_3()
            .bg(rgb(0x1e1e2e))
            .border_1()
            .border_color(if is_focused {
                rgb(0x89b4fa)
            } else {
                rgb(0x45475a)
            })
            .rounded_md()
            .flex()
            .items_center()
            .cursor_pointer()
            .on_click(move |_event, window, _cx| {
                focus_handle.focus(window);
            })
            .child(
                div()
                    .flex_1()
                    .text_sm()
                    .text_color(if input_len == 0 {
                        rgb(0x6c7086)
                    } else {
                        rgb(0xcdd6f4)
                    })
                    .child(if input_len == 0 {
                        "ghp_xxxxxxxxxxxx".to_string()
                    } else {
                        format!("{}|", "*".repeat(input_len.min(39)))
                    }),
            )
    }

    fn render_button(
        &self,
        validating: bool,
        has_token: bool,
        cx: &mut Context<Self>,
    ) -> impl IntoElement {
        let is_disabled = validating || !has_token;

        div()
            .id("connect-btn")
            .w_full()
            .h(px(40.))
            .flex()
            .items_center()
            .justify_center()
            .rounded_md()
            .cursor_pointer()
            .bg(if is_disabled {
                rgb(0x45475a)
            } else {
                rgb(0x89b4fa)
            })
            .text_color(if is_disabled {
                rgb(0x6c7086)
            } else {
                rgb(0x1e1e2e)
            })
            .font_weight(FontWeight::MEDIUM)
            .child(if validating {
                "Validating..."
            } else {
                "Connect"
            })
            .when(!is_disabled, |this| {
                this.hover(|style| style.bg(rgb(0x74c7ec)))
                    .on_click(cx.listener(|this, _event, _window, cx| {
                        this.submit_token(cx);
                    }))
            })
    }

    fn submit_token(&mut self, cx: &mut Context<Self>) {
        let token = self.token_input.clone();

        if token.is_empty() {
            self.error = Some("Please enter a Personal Access Token".to_string());
            cx.notify();
            return;
        }

        self.validating = true;
        self.error = None;
        cx.notify();

        let token_clone = token.clone();
        cx.spawn(async move |view, cx| {
            let result = async {
                let service = GitHubService::new(&token_clone)?;
                let (username, _) = service.validate_token().await?;
                Ok::<_, anyhow::Error>((service, username))
            }
            .await;

            view.update(cx, |view, cx| match result {
                Ok((service, username)) => {
                    if let Err(e) = ConfigService::save_token(&token_clone) {
                        view.error = Some(format!("Failed to save token: {}", e));
                        view.validating = false;
                        cx.notify();
                        return;
                    }

                    cx.update_global::<AppState, _>(|state, _cx| {
                        state.config.github.personal_access_token = Some(token_clone);
                        state.github_service = Some(service);
                        state.username = Some(username);
                        state.screen = AppScreen::Loading;
                    });
                    cx.notify();
                }
                Err(e) => {
                    view.error = Some(format!("Invalid token: {}", e));
                    view.validating = false;
                    cx.notify();
                }
            })
            .ok();
        })
        .detach();
    }

    pub fn set_token(&mut self, token: String) {
        self.token_input = token;
    }
}
