use gpui::{actions, App, Global};

/// WeRun - Windows 启动器库
///
/// 提供启动器的核心功能和组件
pub mod app;
pub mod core;
pub mod platform;
pub mod plugins;
pub mod themes;
pub mod ui;
pub mod utils;

use gpui::KeyBinding;
use gpui_component::Root;

actions!(werun, [
    About,
    Open,
    Quit,
    ToggleSearch,
    TestAction,
    Tab,
    TabPrev,
    ShowPanelInfo,
    ToggleListActiveHighlight
]);

/// 版本信息
pub const VERSION: &str = env!("CARGO_PKG_VERSION");
pub const NAME: &str = env!("CARGO_PKG_NAME");

/// 应用状态
pub struct AppState {}

impl Global for AppState {}

impl AppState {
    fn init(cx: &mut App) {
        let state = Self {};
        cx.set_global::<AppState>(state);
    }
}

pub fn init(cx: &mut App) {
    // 只初始化 gpui_component，不初始化 tracing_subscriber
    // 因为 main.rs 中已经初始化了 env_logger
    gpui_component::init(cx);
    AppState::init(cx);
    themes::init(cx);

    cx.bind_keys([
        KeyBinding::new("/", ToggleSearch, None),
        #[cfg(target_os = "macos")]
        KeyBinding::new("cmd-o", Open, None),
        #[cfg(not(target_os = "macos"))]
        KeyBinding::new("ctrl-o", Open, None),
        #[cfg(target_os = "macos")]
        KeyBinding::new("cmd-q", Quit, None),
        #[cfg(not(target_os = "macos"))]
        KeyBinding::new("alt-f4", Quit, None),
    ]);

    cx.on_action(|_: &Quit, cx: &mut App| {
        cx.quit();
    });

    cx.on_action(|_: &About, cx: &mut App| {
        if let Some(window) = cx.active_window().and_then(|w| w.downcast::<Root>()) {
            cx.defer(move |cx| {
                window
                    .update(cx, |root, window, cx| {
                        root.push_notification("Werun \nVersion 0.1.0", window, cx);
                    })
                    .unwrap();
            });
        }
    });

    cx.activate(true);
}
