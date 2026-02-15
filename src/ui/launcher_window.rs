use std::sync::Arc;

/// 启动器主窗口
///
/// 包含搜索栏、结果列表和预览面板的完整界面
use gpui::prelude::FluentBuilder;
use gpui::*;
use gpui_component::{
    input::{Input, InputEvent, InputState},
    ActiveTheme, Icon, IconName,
};

use crate::{
    core::{
        plugin::PluginManager,
        search::{ActionData, ResultType, SearchResult},
    },
    plugins::{
        app_launcher::AppLauncherPlugin, calculator::CalculatorPlugin, clipboard::ClipboardPlugin,
        file_search::FileSearchPlugin, web_search::WebSearchPlugin,
    },
    utils::clipboard::ClipboardManager,
};

/// 启动器窗口状态
pub struct LauncherWindow {
    /// 搜索输入状态
    search_state: Entity<InputState>,
    /// 搜索结果
    results: Vec<SearchResult>,
    /// 当前选中索引
    selected_index: usize,
    /// 插件管理器
    plugin_manager: Arc<PluginManager>,
    /// 剪贴板管理器
    clipboard_manager: ClipboardManager,
    /// 输入事件订阅
    _subscription: Subscription,
}

impl LauncherWindow {
    /// 创建新的启动器窗口
    pub fn new(window: &mut Window, cx: &mut Context<Self>) -> Self {
        // 初始化插件管理器
        let mut plugin_manager = Self::init_plugins();

        // 初始化所有插件
        if let Err(e) = plugin_manager.initialize_all() {
            log::error!("初始化插件失败: {:?}", e);
        }

        // 创建搜索输入状态
        let search_state =
            cx.new(|cx| InputState::new(window, cx).placeholder("搜索应用、文件、命令..."));

        // 订阅输入事件
        let subscription = cx.subscribe_in(
            &search_state,
            window,
            |this, _state, event: &InputEvent, _window, cx| {
                this.on_input_event(event, cx);
            },
        );

        Self {
            search_state,
            results: Vec::new(),
            selected_index: 0,
            plugin_manager: Arc::new(plugin_manager),
            clipboard_manager: ClipboardManager::new(),
            _subscription: subscription,
        }
    }

    /// 初始化插件
    fn init_plugins() -> PluginManager {
        let mut manager = PluginManager::new();

        // 注册应用启动插件
        manager.register(AppLauncherPlugin::new());

        // 注册计算器插件
        manager.register(CalculatorPlugin::new());

        // 注册剪贴板历史插件
        manager.register(ClipboardPlugin::new());

        // 注册文件搜索插件
        manager.register(FileSearchPlugin::new());

        // 注册网页搜索插件
        manager.register(WebSearchPlugin::new());

        log::info!("已注册 {} 个插件", manager.plugin_count());

        manager
    }

    /// 处理搜索输入变化
    fn on_search_change(&mut self, query: String, cx: &mut Context<Self>) {
        log::debug!("搜索查询: {}", query);

        // 执行搜索
        if !query.is_empty() {
            let mut results = self.plugin_manager.search_all(&query, 50);

            // 为结果添加高亮
            for result in &mut results {
                let highlighted_title =
                    crate::utils::fuzzy::highlight_matches(&query, &result.title);
                result.highlighted_title = Some(highlighted_title);

                let highlighted_desc =
                    crate::utils::fuzzy::highlight_matches(&query, &result.description);
                result.highlighted_description = Some(highlighted_desc);
            }

            self.results = results;
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
            },
            "arrowup" => {
                // 向上导航
                if self.selected_index > 0 {
                    self.selected_index -= 1;
                    cx.notify();
                }
            },
            "enter" => {
                // 确认执行
                if let Some(result) = self.results.get(self.selected_index) {
                    log::info!("执行: {:?}", result);
                    self.execute_result(result);
                    // 执行后关闭窗口
                    cx.emit(DismissEvent);
                }
            },
            "escape" => {
                // 关闭窗口
                cx.emit(DismissEvent);
            },
            _ => {},
        }
    }

    /// 执行搜索结果
    fn execute_result(&self, result: &SearchResult) {
        // 尝试通过插件管理器执行
        if let Err(e) = self.plugin_manager.execute(result) {
            log::error!("通过插件执行失败: {:?}", e);

            // 如果插件执行失败，尝试根据类型执行
            match &result.action {
                ActionData::LaunchApp { path, .. } => {
                    log::info!("启动应用: {}", path);
                    let _ =
                        std::process::Command::new("cmd").args(["/c", "start", "", path]).spawn();
                },
                ActionData::OpenFile { path } => {
                    log::info!("打开文件: {}", path);
                    let _ = std::process::Command::new("explorer").arg(path).spawn();
                },
                ActionData::ExecuteCommand { command } => {
                    log::info!("执行命令: {}", command);
                    let _ = std::process::Command::new("cmd").args(["/c", command]).spawn();
                },
                ActionData::CopyToClipboard { text } => {
                    log::info!("复制到剪贴板: {}", text);
                    if let Err(e) = self.clipboard_manager.set_text(text) {
                        log::error!("复制到剪贴板失败: {:?}", e);
                    }
                },
                ActionData::OpenUrl { url } => {
                    log::info!("打开 URL: {}", url);
                    let _ =
                        std::process::Command::new("cmd").args(["/c", "start", "", url]).spawn();
                },
                _ => {
                    log::warn!("未知的动作类型");
                },
            }
        }
    }

    /// 处理输入事件
    fn on_input_event(&mut self, event: &InputEvent, cx: &mut Context<Self>) {
        match event {
            InputEvent::Change => {
                let query = self.search_state.read(cx).value().to_string();
                self.on_search_change(query, cx);
            },
            InputEvent::PressEnter { .. } => {
                // 执行选中的结果
                if let Some(result) = self.results.get(self.selected_index) {
                    log::info!("执行: {:?}", result);
                    self.execute_result(result);
                    cx.emit(DismissEvent);
                }
            },
            _ => {},
        }
    }
}

impl Render for LauncherWindow {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let theme = cx.theme();

        div()
            .size_full()
            .flex()
            .flex_col()
            .gap_2()
            .p_4()
            .bg(theme.background)
            .rounded_xl()
            .border_1()
            .border_color(theme.border)
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
                    .border_color(theme.border)
                    .bg(theme.secondary)
                    .child(Icon::new(IconName::Search).text_color(theme.muted_foreground))
                    .child(Input::new(&self.search_state).cleanable(true).flex_1()),
            )
            // 分隔线
            .child(div().h_px().w_full().bg(theme.border))
            // 结果列表
            .child(
                div()
                    .flex()
                    .flex_col()
                    .gap_1()
                    .children(self.results.iter().enumerate().map(|(index, result)| {
                        let is_selected = index == self.selected_index;
                        render_result_item(result, is_selected, &theme)
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
                    .text_color(theme.muted_foreground)
                    .child(format!("{} 个结果", self.results.len()))
                    .child("↑↓ 选择 · ↵ 执行 · Esc 关闭"),
            )
    }
}

/// 获取结果类型的图标
fn get_result_icon(result_type: &ResultType) -> IconName {
    match result_type {
        ResultType::Application => IconName::AppWindow,
        ResultType::File => IconName::File,
        ResultType::Folder => IconName::Folder,
        ResultType::Command => IconName::Terminal,
        ResultType::Calculator => IconName::Calculator,
        ResultType::Clipboard => IconName::Clipboard,
        ResultType::Settings => IconName::Settings2,
        ResultType::Custom(_) => IconName::FileBox,
    }
}

/// 解析高亮文本，返回普通文本和高亮文本的片段
fn parse_highlighted_text(text: &str) -> Vec<(String, bool)> {
    let mut fragments = Vec::new();
    let mut current_text = String::new();
    let mut in_bracket = false;

    for ch in text.chars() {
        match ch {
            '[' => {
                if !current_text.is_empty() {
                    fragments.push((current_text.clone(), false));
                    current_text.clear();
                }
                in_bracket = true;
            },
            ']' => {
                if !current_text.is_empty() {
                    fragments.push((current_text.clone(), true));
                    current_text.clear();
                }
                in_bracket = false;
            },
            _ => {
                current_text.push(ch);
            },
        }
    }

    // 添加剩余的文本
    if !current_text.is_empty() {
        fragments.push((current_text, in_bracket));
    }

    fragments
}

/// 渲染高亮文本
///
/// 样式规则：
/// - 未选中：匹配字符橙色 + 粗体
/// - 选中：匹配字符橙色 + 浅蓝边框 + 粗体
fn render_highlighted_text(
    text: &str,
    theme: &gpui_component::Theme,
    is_selected: bool,
    is_title: bool,
) -> impl IntoElement {
    let fragments = parse_highlighted_text(text);

    // 橙色 - 使用主题中的 warning 颜色（通常是橙色/黄色）
    let orange_color = theme.warning;

    // 基础颜色
    let base_color = if is_selected {
        theme.accent_foreground
    } else if is_title {
        theme.foreground
    } else {
        theme.muted_foreground
    };

    div().flex().flex_row().children(fragments.into_iter().map(move |(text, is_highlighted)| {
        let mut div_element = div()
            .text_color(if is_highlighted { orange_color } else { base_color })
            .font_weight(if is_highlighted { FontWeight::BOLD } else { FontWeight::NORMAL });

        if is_highlighted {
            if is_selected {
                // 选中状态：橙色 + 浅蓝边框 + 粗体
                div_element = div_element
                    .border_1()
                    .border_color(theme.primary.opacity(0.5))
                    .rounded_sm()
                    .px_1()
                    .py_0();
            } else {
                // 未选中状态：橙色 + 粗体（无边框）
                div_element = div_element.px_1();
            }
        }

        div_element.child(text)
    }))
}

/// 渲染结果项
fn render_result_item(
    result: &SearchResult,
    is_selected: bool,
    theme: &gpui_component::Theme,
) -> impl IntoElement {
    let bg_color = if is_selected { theme.accent } else { theme.background };

    let text_color = if is_selected { theme.accent_foreground } else { theme.foreground };

    let type_name = match &result.result_type {
        ResultType::Application => "应用",
        ResultType::File => "文件",
        ResultType::Folder => "文件夹",
        ResultType::Command => "命令",
        ResultType::Calculator => "计算",
        ResultType::Clipboard => "剪贴板",
        ResultType::Settings => "设置",
        ResultType::Custom(_) => "其他",
    };

    let icon = get_result_icon(&result.result_type);

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
                .bg(if is_selected { theme.accent_foreground } else { theme.secondary })
                .child(Icon::new(icon).text_color(text_color)),
        )
        .child(
            div()
                .flex()
                .flex_col()
                .flex_1()
                .gap_1()
                .child(div().text_sm().child(render_highlighted_text(
                    result.display_title(),
                    theme,
                    is_selected,
                    true, // 是标题
                )))
                .child(div().text_xs().child(render_highlighted_text(
                    result.display_description(),
                    theme,
                    is_selected,
                    false, // 是描述
                ))),
        )
        .child(
            div()
                .px_2()
                .py_0()
                .rounded_full()
                .text_xs()
                .bg(if is_selected { theme.accent_foreground } else { theme.secondary })
                .text_color(if is_selected { theme.accent } else { theme.muted_foreground })
                .child(type_name),
        )
}

/// 关闭窗口事件
pub struct DismissEvent;

impl EventEmitter<DismissEvent> for LauncherWindow {}
