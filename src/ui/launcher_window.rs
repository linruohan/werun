/// 启动器主窗口
///
/// 包含搜索栏、结果列表和预览面板的完整界面
use gpui::*;
use std::sync::Arc;

use crate::core::plugin::PluginManager;
use crate::core::search::{SearchEngine, SearchResult};
use crate::plugins::app_launcher::AppLauncherPlugin;
use crate::plugins::calculator::CalculatorPlugin;
use crate::plugins::clipboard::ClipboardPlugin;
use crate::plugins::file_search::FileSearchPlugin;

/// 启动器窗口状态
pub struct LauncherWindow {
    /// 搜索查询
    search_query: SharedString,
    /// 搜索结果
    results: Vec<SearchResult>,
    /// 当前选中索引
    selected_index: usize,
    /// 搜索引擎
    search_engine: SearchEngine,
    /// 插件管理器
    plugin_manager: Arc<PluginManager>,
}

impl LauncherWindow {
    /// 创建新的启动器窗口
    pub fn new(_window: &mut Window, _cx: &mut Context<Self>) -> Self {
        // 初始化插件管理器
        let plugin_manager = Self::init_plugins();

        Self {
            search_query: SharedString::default(),
            results: Vec::new(),
            selected_index: 0,
            search_engine: SearchEngine::new(),
            plugin_manager: Arc::new(plugin_manager),
        }
    }

    /// 初始化插件
    fn init_plugins() -> PluginManager {
        let mut manager = PluginManager::new();

        // 注册应用启动插件
        let app_launcher = Arc::new(AppLauncherPlugin::new());
        manager.register(app_launcher);

        // 注册计算器插件
        let calculator = Arc::new(CalculatorPlugin::new());
        manager.register(calculator);

        // 注册剪贴板历史插件
        let clipboard = Arc::new(ClipboardPlugin::new());
        manager.register(clipboard);

        // 注册文件搜索插件
        let file_search = Arc::new(FileSearchPlugin::new());
        manager.register(file_search);

        log::info!("已注册 {} 个插件", manager.all_plugins().len());

        manager
    }

    /// 处理搜索输入变化
    fn on_search_change(&mut self, query: String, cx: &mut Context<Self>) {
        log::debug!("搜索查询: {}", query);

        self.search_query = query.clone().into();

        // 更新搜索引擎查询
        self.search_engine.set_query(&query);

        // 执行搜索
        if !query.is_empty() {
            let plugins: Vec<_> = self.plugin_manager.enabled_plugins();
            self.results = self.search_engine.search(&plugins);
        } else {
            self.results.clear();
        }

        // 清空选中索引
        self.selected_index = 0;

        // 通知 UI 更新
        cx.notify();
    }

    /// 处理键盘事件
    fn handle_key_event(&mut self, event: &KeyDownEvent, cx: &mut Context<Self>) {
        match event.keystroke.key.as_str() {
            "arrowdown" => {
                // 向下导航
                if !self.results.is_empty() {
                    self.selected_index = (self.selected_index + 1).min(self.results.len() - 1);
                    cx.notify();
                }
            }
            "arrowup" => {
                // 向上导航
                if self.selected_index > 0 {
                    self.selected_index -= 1;
                    cx.notify();
                }
            }
            "enter" => {
                // 确认执行
                if let Some(result) = self.results.get(self.selected_index) {
                    log::info!("执行: {:?}", result);
                    self.execute_result(result);
                }
            }
            "escape" => {
                // 关闭窗口
                cx.emit(DismissEvent);
            }
            _ => {}
        }
    }

    /// 执行搜索结果
    fn execute_result(&self, result: &SearchResult) {
        // 根据结果类型找到对应的插件执行
        let plugins = self.plugin_manager.enabled_plugins();

        for plugin in plugins {
            // 简单匹配：根据 ID 前缀判断
            if result.id.starts_with(&format!("{}:", plugin.id())) {
                if let Err(e) = plugin.execute(result) {
                    log::error!("执行失败: {:?}", e);
                }
                return;
            }
        }

        // 如果没有匹配到特定插件，尝试根据类型执行
        match &result.action {
            crate::core::search::ActionData::LaunchApp { path, .. } => {
                log::info!("启动应用: {}", path);
                // 使用 cmd /c start 启动应用
                let _ = std::process::Command::new("cmd")
                    .args(["/c", "start", "", path])
                    .spawn();
            }
            crate::core::search::ActionData::OpenFile { path } => {
                log::info!("打开文件: {}", path);
                let _ = std::process::Command::new("explorer").arg(path).spawn();
            }
            crate::core::search::ActionData::ExecuteCommand { command } => {
                log::info!("执行命令: {}", command);
                let _ = std::process::Command::new("cmd")
                    .args(["/c", command])
                    .spawn();
            }
            crate::core::search::ActionData::CopyToClipboard { text } => {
                log::info!("复制到剪贴板: {}", text);
                // TODO: 实现剪贴板操作
            }
            crate::core::search::ActionData::OpenUrl { url } => {
                log::info!("打开 URL: {}", url);
                let _ = std::process::Command::new("cmd")
                    .args(["/c", "start", "", url])
                    .spawn();
            }
            _ => {
                log::warn!("未知的动作类型");
            }
        }
    }
}

impl Render for LauncherWindow {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        div()
            .size_full()
            .flex()
            .flex_col()
            .gap_2()
            .p_4()
            // 键盘事件处理
            .on_key_down(cx.listener(|this, event: &KeyDownEvent, _window, cx| {
                this.handle_key_event(event, cx);
            }))
            // 搜索输入
            .child(
                div()
                    .flex()
                    .flex_row()
                    .items_center()
                    .gap_2()
                    .px_3()
                    .py_2()
                    .rounded_lg()
                    .border_1()
                    .child("🔍")
                    .child(div().flex().flex_1().child("搜索输入框")),
            )
            // 分隔线
            .child(div().h_px().w_full())
            // 结果列表
            .child(
                div()
                    .flex()
                    .flex_col()
                    .gap_1()
                    .children(self.results.iter().enumerate().map(|(index, result)| {
                        let is_selected = index == self.selected_index;
                        render_result_item(result, is_selected)
                    })),
            )
            // 底部状态栏
            .child(
                div()
                    .flex()
                    .flex_row()
                    .items_center()
                    .justify_between()
                    .px_2()
                    .py_1()
                    .text_sm()
                    .child(format!("{} 个结果", self.results.len()))
                    .child("↑↓ 选择 · ↵ 执行 · Esc 关闭"),
            )
    }
}

/// 渲染结果项
fn render_result_item(result: &SearchResult, is_selected: bool) -> impl IntoElement {
    let bg_color = if is_selected {
        gpui::rgb(0x3b82f6) // 蓝色高亮
    } else {
        gpui::rgb(0x1e1e2e) // 默认背景
    };

    let text_color = if is_selected {
        gpui::rgb(0xffffff)
    } else {
        gpui::rgb(0xcdd6f4)
    };

    let type_name = match &result.result_type {
        crate::core::search::ResultType::Application => "应用",
        crate::core::search::ResultType::File => "文件",
        crate::core::search::ResultType::Folder => "文件夹",
        crate::core::search::ResultType::Command => "命令",
        crate::core::search::ResultType::Calculator => "计算",
        crate::core::search::ResultType::Clipboard => "剪贴板",
        crate::core::search::ResultType::Settings => "设置",
        crate::core::search::ResultType::Custom(_) => "其他",
    };

    div()
        .flex()
        .flex_row()
        .items_center()
        .gap_3()
        .px_3()
        .py_2()
        .rounded_md()
        .bg(bg_color)
        .cursor_pointer()
        .child(
            div()
                .flex()
                .items_center()
                .justify_center()
                .w_8()
                .h_8()
                .rounded_md()
                .bg(gpui::rgb(0x313244))
                .child("📦"),
        )
        .child(
            div()
                .flex()
                .flex_col()
                .flex_1()
                .gap_1()
                .child(
                    div()
                        .text_sm()
                        .font_weight(FontWeight::MEDIUM)
                        .text_color(text_color)
                        .child(result.title.clone()),
                )
                .child(
                    div()
                        .text_xs()
                        .text_color(gpui::rgb(0x6c7086))
                        .child(result.description.clone()),
                ),
        )
        .child(
            div()
                .px_2()
                .py_0()
                .rounded_full()
                .text_xs()
                .bg(gpui::rgb(0x313244))
                .text_color(gpui::rgb(0x6c7086))
                .child(type_name),
        )
}

/// 关闭窗口事件
pub struct DismissEvent;

impl EventEmitter<DismissEvent> for LauncherWindow {}
