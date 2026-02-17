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

/// 枚举窗口数据结构
struct EnumData {
    found_hwnd: Option<windows::Win32::Foundation::HWND>,
}

/// 全局快捷键管理器（使用 Box 和 leak 使其永久存活）
static mut HOTKEY_MANAGER: Option<Box<platform::windows::GlobalHotkeyManager>> = None;

fn main() {
    // 初始化日志（默认设置为 info 级别）
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("info")).init();

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

/// 注册全局快捷键 Alt+Space
fn register_global_hotkey() {
    use platform::windows::GlobalHotkeyManager;

    // 从配置中读取快捷键
    let toggle_key = global_config().get_config().keybindings.toggle_launcher;
    log::info!("注册全局快捷键: {}", toggle_key);

    std::thread::spawn(move || {
        // 等待窗口创建完成
        std::thread::sleep(std::time::Duration::from_millis(2000));

        log::info!("开始注册全局快捷键...");

        match GlobalHotkeyManager::new() {
            Ok(mut manager) => {
                log::info!("快捷键管理器创建成功");
                if let Err(e) = manager.register_alt_space(|| {
                    log::info!("Alt+Space 快捷键被触发");
                    // 切换窗口显示/隐藏
                    toggle_launcher_window();
                }) {
                    log::error!("注册全局快捷键失败: {:?}", e);
                } else {
                    // 将 manager 放入全局变量，防止被 Drop
                    unsafe {
                        HOTKEY_MANAGER = Some(Box::new(manager));
                        log::info!("全局快捷键管理器已保存");
                    }
                }
            },
            Err(e) => {
                log::error!("创建快捷键管理器失败: {:?}", e);
            },
        }
    });
}

/// 切换窗口显示/隐藏（供快捷键调用）
fn toggle_launcher_window() {
    log::info!("请求切换窗口状态");

    // 使用 Windows API 直接操作窗口
    use windows::Win32::{
        Foundation::LPARAM,
        UI::WindowsAndMessaging::{EnumWindows, FindWindowW},
    };

    unsafe {
        // 尝试多种方式查找窗口

        // 方式1：通过窗口标题查找
        let window_name: Vec<u16> = "WeRun".encode_utf16().chain(std::iter::once(0)).collect();
        log::info!("尝试查找窗口标题: WeRun");

        match FindWindowW(None, windows::core::PCWSTR(window_name.as_ptr())) {
            Ok(hwnd) => {
                log::info!("找到窗口 (通过标题): {:?}", hwnd);
                toggle_window_visibility(hwnd);
                return;
            },
            Err(e) => {
                log::warn!("通过标题查找窗口失败: {:?}", e);
            },
        }

        // 方式2：枚举所有窗口，查找标题包含 "WeRun" 的窗口
        log::info!("尝试枚举窗口查找...");

        let mut enum_data = EnumData { found_hwnd: None };

        let _ = EnumWindows(Some(enum_windows_callback), LPARAM(&mut enum_data as *mut _ as isize));

        if let Some(hwnd) = enum_data.found_hwnd {
            log::info!("找到窗口 (通过枚举): {:?}", hwnd);
            toggle_window_visibility(hwnd);
            return;
        }

        log::warn!("未找到 WeRun 窗口");
    }
}

/// 切换窗口可见性
unsafe fn toggle_window_visibility(hwnd: windows::Win32::Foundation::HWND) {
    use windows::Win32::UI::WindowsAndMessaging::{
        IsWindowVisible, SetForegroundWindow, ShowWindow, SW_HIDE, SW_SHOW,
    };

    // 检查窗口是否可见
    if IsWindowVisible(hwnd).as_bool() {
        log::info!("窗口当前可见，执行隐藏");
        let _ = ShowWindow(hwnd, SW_HIDE);
    } else {
        log::info!("窗口当前隐藏，执行显示");
        let _ = ShowWindow(hwnd, SW_SHOW);
        // 激活窗口
        let _ = SetForegroundWindow(hwnd);
    }
}

/// 枚举窗口回调函数
unsafe extern "system" fn enum_windows_callback(
    hwnd: windows::Win32::Foundation::HWND,
    lparam: windows::Win32::Foundation::LPARAM,
) -> windows::Win32::Foundation::BOOL {
    use windows::Win32::UI::WindowsAndMessaging::GetWindowTextW;

    let data = &mut *(lparam.0 as *mut EnumData);

    // 获取窗口文本
    let mut text: [u16; 256] = [0; 256];
    let len = GetWindowTextW(hwnd, &mut text);

    if len > 0 {
        let window_text = String::from_utf16_lossy(&text[..len as usize]);

        // 检查窗口标题是否包含 "WeRun"
        if window_text.contains("WeRun") {
            log::info!("找到匹配的窗口: {}", window_text);
            data.found_hwnd = Some(hwnd);
            return windows::Win32::Foundation::BOOL(0); // 停止枚举
        }
    }

    windows::Win32::Foundation::BOOL(1) // 继续枚举
}
