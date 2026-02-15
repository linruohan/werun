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
mod window_manager;

use core::config_manager::global_config;

use app::LauncherApp;
use gpui_component_assets::Assets;
use ui::create_new_window;
fn main() {
    // 初始化日志
    env_logger::init();

    log::info!("WeRun 启动器初始化...");
    log::info!("配置目录: {:?}", dirs::config_dir());

    // 启动 GPUI 应用
    Application::new().with_assets(Assets).run(move |cx: &mut App| {
        // 初始化 werun
        ui::init(cx);
        // 激活应用
        cx.activate(true);

        // 加载配置
        let config = global_config().get_config();
        log::info!("当前主题: {}", config.theme.current_theme);
        log::info!("窗口大小: {}x{}", config.window.width, config.window.height);

        // 打开启动器窗口
        create_new_window("WeRun", LauncherApp::view, cx);

        // 注册全局快捷键
        register_global_hotkey();
    });

    // 应用退出时保存配置
    if let Err(e) = global_config().save() {
        log::error!("保存配置失败: {:?}", e);
    }
}

// /// 打开启动器主窗口
// fn open_launcher_window(cx: &mut App) {
//     // 创建窗口
//     match cx.open_window(window_options, |window, cx| cx.new(|cx| LauncherApp::new(window, cx)))
// {         Ok(handle) => {
//             log::info!("启动器窗口已打开");
//             // 设置窗口句柄到窗口管理器
//             global_window_manager().set_window_handle(handle);
//         },
//         Err(e) => {
//             log::error!("打开窗口失败: {:?}", e);
//         },
//     }
// }

/// 注册全局快捷键 Alt+Space
fn register_global_hotkey() {
    use platform::windows::GlobalHotkeyManager;

    // 从配置中读取快捷键
    let toggle_key = global_config().get_config().keybindings.toggle_launcher;
    log::info!("注册全局快捷键: {}", toggle_key);

    std::thread::spawn(move || {
        // 等待窗口创建完成
        std::thread::sleep(std::time::Duration::from_millis(500));

        match GlobalHotkeyManager::new() {
            Ok(mut manager) => {
                if let Err(e) = manager.register_alt_space(|| {
                    log::info!("Alt+Space 快捷键被触发");
                    // 使用 GPUI 的 AppContext 来切换窗口
                    // 这里需要通过消息队列或其他线程安全的方式
                    // 暂时只记录日志，实际实现需要在主线程中执行
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

/// 切换窗口显示/隐藏（供快捷键调用）
pub fn toggle_launcher_window() {
    log::info!("请求切换窗口状态");
    // 这里需要通过 GPUI 的事件系统在主线程中执行
    // 暂时只记录日志
}
