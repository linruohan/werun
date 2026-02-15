/// WeRun - Windows 启动器
///
/// 使用 GPUI 开发的高性能启动器
/// 灵感来源于 Vicinae
mod app;
mod core;
mod platform;
mod plugins;
mod ui;
mod utils;
use app::LauncherApp;
use gpui::{
    px, size, App,
    AppContext, Application, Bounds, Styled, TitlebarOptions,
    WindowBackgroundAppearance, WindowBounds, WindowKind, WindowOptions,
};
use gpui_component_assets::Assets;

fn main() {
    // 初始化日志
    env_logger::init();
    let app = Application::new().with_assets(Assets);
    log::info!("WeRun 启动器初始化...");

    // 启动 GPUI 应用
    app.run(move |cx| {
        // 打开启动器窗口
        open_launcher_window(cx);
    });
}

/// 打开启动器主窗口
fn open_launcher_window(cx: &mut App) {
    // 窗口选项配置
    let mut window_size = size(px(800.0), px(500.0));
    if let Some(display) = cx.primary_display() {
        let display_size = display.bounds().size;
        window_size.width = window_size.width.min(display_size.width * 0.85);
        window_size.height = window_size.height.min(display_size.height * 0.85);
    }
    let window_bounds = Bounds::centered(None, window_size, cx);
    let window_options = WindowOptions {
        window_bounds: Some(WindowBounds::Windowed(window_bounds)),
        titlebar: Some(TitlebarOptions {
            title: None,
            appears_transparent: true,
            traffic_light_position: None,
        }),
        window_background: WindowBackgroundAppearance::Transparent,
        kind: WindowKind::PopUp,
        display_id: None,
        window_min_size: Some(size(px(600.0), px(400.0))),
        focus: true,
        show: true,
        is_movable: false,
        app_id: Some("werun".to_string()),
        ..Default::default()
    };

    // 创建窗口
    let window_handle = cx.open_window(window_options, |window, cx| {
        cx.new(|cx| LauncherApp::new(window, cx))
    });

    match window_handle {
        Ok(_handle) => {
            log::info!("启动器窗口已打开");
        }
        Err(e) => {
            log::error!("打开窗口失败: {:?}", e);
        }
    }
}
