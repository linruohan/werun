/// WeRun - Windows 启动器
///
/// 使用 GPUI 开发的高性能启动器
/// 灵感来源于 Vicinae
use gpui::*;

mod app;
mod core;
mod platform;
mod plugins;
mod ui;
mod utils;

use app::LauncherApp;
use gpui_component_assets::Assets;

fn main() {
    // 初始化日志
    env_logger::init();

    log::info!("WeRun 启动器初始化...");

    // 启动 GPUI 应用
    Application::new().with_assets(Assets).run(move |cx: &mut App| {
        // 初始化 werun
        werun::init(cx);

        // 激活应用
        cx.activate(true);

        // 打开启动器窗口
        open_launcher_window(cx);

        // 注册全局快捷键
        register_global_hotkey();
    });
}

/// 打开启动器主窗口
fn open_launcher_window(cx: &mut App) {
    // 窗口选项配置
    let window_options = WindowOptions {
        window_bounds: Some(WindowBounds::Windowed(Bounds::centered(
            None,
            size(px(800.0), px(500.0)),
            cx,
        ))),
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
    match cx.open_window(window_options, |window, cx| cx.new(|cx| LauncherApp::new(window, cx))) {
        Ok(_handle) => {
            log::info!("启动器窗口已打开");
        },
        Err(e) => {
            log::error!("打开窗口失败: {:?}", e);
        },
    }
}

/// 注册全局快捷键 Alt+Space
fn register_global_hotkey() {
    use platform::windows::GlobalHotkeyManager;

    std::thread::spawn(|| {
        // 等待窗口创建完成
        std::thread::sleep(std::time::Duration::from_millis(500));

        match GlobalHotkeyManager::new() {
            Ok(mut manager) => {
                if let Err(e) = manager.register_alt_space(|| {
                    log::info!("Alt+Space 快捷键被触发");
                }) {
                    log::error!("注册全局快捷键失败: {:?}", e);
                }
            },
            Err(e) => {
                log::error!("创建快捷键管理器失败: {:?}", e);
            },
        }
    });
}
