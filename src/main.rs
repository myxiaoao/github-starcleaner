use github_starcleaner::services::ConfigService;
use github_starcleaner::state::AppState;
use github_starcleaner::ui::AppView;
use gpui::*;

fn main() {
    tracing_subscriber::fmt::init();

    let app = Application::new();
    app.on_reopen(|cx| {
        // When app is reopened (e.g., clicked in dock), open a new window if none exist
        if cx.windows().is_empty() {
            open_main_window(cx);
        } else {
            // Activate existing window
            if let Some(window_handle) = cx.windows().first() {
                window_handle.update(cx, |_view, window, _cx| {
                    window.activate_window();
                }).ok();
            }
        }
    });
    app.run(|cx: &mut App| {
            // Load config and initialize state
            let config = ConfigService::load().unwrap_or_default();
            let state = AppState::from_config(config);
            cx.set_global(state);

            // Open main window
            open_main_window(cx);
        });
}

fn open_main_window(cx: &mut App) {
    cx.open_window(
        WindowOptions {
            titlebar: Some(TitlebarOptions {
                title: Some("GitHub StarCleaner".into()),
                ..Default::default()
            }),
            window_bounds: Some(WindowBounds::Windowed(Bounds {
                origin: point(px(100.), px(100.)),
                size: size(px(1200.), px(800.)),
            })),
            focus: true,
            show: true,
            kind: WindowKind::Normal,
            is_movable: true,
            ..Default::default()
        },
        |_window, cx| cx.new(|cx| AppView::new(cx)),
    )
    .expect("Failed to open window");
}
