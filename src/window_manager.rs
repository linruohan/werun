use std::sync::{Arc, Mutex};

/// 窗口管理器
///
/// 管理启动器窗口的显示、隐藏和状态
use gpui::*;

use crate::app::LauncherApp;

/// 窗口状态
#[derive(Clone, Debug, PartialEq)]
pub enum WindowVisibility {
    /// 窗口可见
    Visible,
    /// 窗口隐藏
    Hidden,
    /// 窗口最小化
    Minimized,
}

/// 窗口管理器
pub struct WindowManager {
    /// 窗口句柄
    window_handle: Arc<Mutex<Option<WindowHandle<LauncherApp>>>>,
    /// 窗口可见性状态
    visibility: Arc<Mutex<WindowVisibility>>,
    /// 窗口位置
    position: Arc<Mutex<Option<Point<Pixels>>>>,
}

impl WindowManager {
    /// 创建新的窗口管理器
    pub fn new() -> Self {
        Self {
            window_handle: Arc::new(Mutex::new(None)),
            visibility: Arc::new(Mutex::new(WindowVisibility::Hidden)),
            position: Arc::new(Mutex::new(None)),
        }
    }

    /// 设置窗口句柄
    pub fn set_window_handle(&self, handle: WindowHandle<LauncherApp>) {
        let mut guard = self.window_handle.lock().unwrap();
        *guard = Some(handle);
        *self.visibility.lock().unwrap() = WindowVisibility::Visible;
    }

    /// 切换窗口显示/隐藏
    pub fn toggle_window(&self, _cx: &mut App) {
        let visibility = self.visibility.lock().unwrap().clone();

        match visibility {
            WindowVisibility::Visible => {
                self.hide_window();
            },
            WindowVisibility::Hidden | WindowVisibility::Minimized => {
                self.show_window();
            },
        }
    }

    /// 显示窗口
    pub fn show_window(&self) {
        if self.window_handle.lock().unwrap().is_some() {
            // 更新可见性状态
            *self.visibility.lock().unwrap() = WindowVisibility::Visible;
            log::info!("窗口已显示");
        }
    }

    /// 隐藏窗口
    pub fn hide_window(&self) {
        if self.window_handle.lock().unwrap().is_some() {
            // 更新可见性状态
            *self.visibility.lock().unwrap() = WindowVisibility::Hidden;
            log::info!("窗口已隐藏");
        }
    }

    /// 最小化窗口
    pub fn minimize_window(&self) {
        if self.window_handle.lock().unwrap().is_some() {
            *self.visibility.lock().unwrap() = WindowVisibility::Minimized;
            log::info!("窗口已最小化");
        }
    }

    /// 关闭窗口
    pub fn close_window(&self) {
        if self.window_handle.lock().unwrap().is_some() {
            *self.window_handle.lock().unwrap() = None;
            *self.visibility.lock().unwrap() = WindowVisibility::Hidden;
            log::info!("窗口已关闭");
        }
    }

    /// 获取窗口可见性状态
    pub fn get_visibility(&self) -> WindowVisibility {
        self.visibility.lock().unwrap().clone()
    }

    /// 检查窗口是否可见
    pub fn is_visible(&self) -> bool {
        matches!(self.get_visibility(), WindowVisibility::Visible)
    }

    /// 保存窗口位置
    pub fn save_position(&self, pos: Point<Pixels>) {
        *self.position.lock().unwrap() = Some(pos);
    }

    /// 获取窗口位置
    pub fn get_position(&self) -> Option<Point<Pixels>> {
        *self.position.lock().unwrap()
    }

    /// 窗口失焦时自动隐藏
    pub fn on_blur(&self) {
        // 检查配置是否启用失焦隐藏
        let hide_on_blur =
            crate::core::config_manager::global_config().get_config().window.hide_on_blur;

        if hide_on_blur {
            self.hide_window();
        }
    }
}

impl Default for WindowManager {
    fn default() -> Self {
        Self::new()
    }
}

// 全局窗口管理器实例
use once_cell::sync::Lazy;

static GLOBAL_WINDOW_MANAGER: Lazy<WindowManager> = Lazy::new(WindowManager::new);

/// 获取全局窗口管理器
pub fn global_window_manager() -> &'static WindowManager {
    &GLOBAL_WINDOW_MANAGER
}
