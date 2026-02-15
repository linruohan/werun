/// 应用主模块
///
/// 管理启动器的整体状态和生命周期
use gpui::*;

use crate::ui::launcher_window::LauncherWindow;

/// 启动器应用状态
pub struct LauncherApp {
    /// 主窗口视图
    window_view: Entity<LauncherWindow>,
}

impl LauncherApp {
    /// 创建新的应用实例
    pub fn new(window: &mut Window, cx: &mut Context<Self>) -> Self {
        // 创建主窗口视图
        let window_view = cx.new(|cx| LauncherWindow::new(window, cx));

        log::info!("LauncherApp 初始化完成");

        Self { window_view }
    }
}

impl Render for LauncherApp {
    fn render(&mut self, _window: &mut Window, _cx: &mut Context<Self>) -> impl IntoElement {
        div()
            .size_full()
            .flex()
            .flex_col()
            .child(self.window_view.clone())
    }
}
