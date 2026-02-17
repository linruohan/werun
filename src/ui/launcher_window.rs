use std::sync::Arc;

use gpui::prelude::FluentBuilder;
use gpui::*;
use gpui_component::{
    list::{List, ListEvent, ListState},
    ActiveTheme, Icon, IconName,
};

use crate::{
    core::{
        plugin::PluginManager,
        search::{ActionData, ResultType, SearchResult},
    },
    plugins::{
        app_launcher::AppLauncherPlugin, calculator::CalculatorPlugin, clipboard::ClipboardPlugin,
        color_picker::ColorPickerPlugin, custom_commands::CustomCommandsPlugin,
        file_search::FileSearchPlugin, system_commands::SystemCommandsPlugin,
        web_search::WebSearchPlugin, window_switcher::WindowSwitcherPlugin,
    },
    ui::result_list::ResultListDelegate,
    utils::clipboard::ClipboardManager,
};

/// 启动器窗口状态
pub struct LauncherWindow {
    /// 列表状态
    list_state: Entity<ListState<ResultListDelegate>>,
    /// 插件管理器
    plugin_manager: Arc<PluginManager>,
    /// 剪贴板管理器
    clipboard_manager: ClipboardManager,
    /// 当前激活的插件ID
    active_plugin_id: Option<String>,
    /// 列表事件订阅
    _list_subscription: Subscription,
    /// 快捷键配置
    keybindings: crate::core::config::KeybindingsConfig,
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

        // 创建列表委托和状态（使用 List 内置搜索）
        let plugin_manager = Arc::new(plugin_manager);
        let delegate =
            ResultListDelegate::new(Vec::new()).with_plugin_manager(plugin_manager.clone());
        let list_state = cx.new(|cx| ListState::new(delegate, window, cx).searchable(true));

        // 订阅列表事件
        let list_subscription =
            cx.subscribe_in(&list_state, window, |this, _state, event: &ListEvent, window, cx| {
                this.on_list_event(event, window, cx);
            });

        // 加载快捷键配置
        let keybindings = crate::core::config_manager::global_config().get_config().keybindings;

        Self {
            list_state,
            plugin_manager,
            clipboard_manager: ClipboardManager::new(),
            active_plugin_id: None,
            _list_subscription: list_subscription,
            keybindings,
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

        // 注册系统命令插件
        manager.register(SystemCommandsPlugin::new());

        // 注册自定义命令插件
        manager.register(CustomCommandsPlugin::new());

        // 注册颜色选择器插件
        manager.register(ColorPickerPlugin::new());

        // 注册窗口切换器插件
        manager.register(WindowSwitcherPlugin::new());

        log::info!("已注册 {} 个插件", manager.plugin_count());

        manager
    }

    /// 处理搜索输入变化
    fn on_search_change(&mut self, query: String, cx: &mut Context<Self>) {
        log::debug!("搜索查询: {}", query);

        // 执行搜索
        let results = if !query.is_empty() {
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

            results
        } else {
            Vec::new()
        };

        // 更新列表状态
        self.list_state.update(cx, |state, cx| {
            state.delegate_mut().update_from_search(results);
            cx.notify();
        });
    }

    /// 处理列表事件
    fn on_list_event(&mut self, event: &ListEvent, window: &mut Window, cx: &mut Context<Self>) {
        match event {
            ListEvent::Confirm(ix) => {
                let result_opt = {
                    let delegate = self.list_state.read(cx).delegate();
                    delegate.get_item(ix.row).cloned()
                };

                if let Some(result) = result_opt {
                    // 检查是否是插件选择
                    if result.id.starts_with("__plugin__:") {
                        if let ActionData::Custom { plugin: _, data } = &result.action {
                            // 选择插件：设置活动插件
                            let plugin_id = data.clone();
                            self.active_plugin_id = Some(plugin_id.clone());

                            // 更新 delegate 的活动插件
                            self.list_state.update(cx, |state, cx| {
                                state.delegate_mut().set_active_plugin(Some(plugin_id.clone()));
                            });

                            log::info!("切换到插件: {}", plugin_id);
                            return;
                        }
                    }

                    log::info!("确认执行: {:?}", result);
                    self.execute_result(&result);
                    cx.emit(DismissEvent);
                }
            },
            ListEvent::Cancel => {
                cx.emit(DismissEvent);
            },
            _ => {},
        }
    }

    /// 执行搜索
    fn perform_search(&mut self, query: &str, cx: &mut Context<Self>) {
        log::info!(
            "perform_search 被调用，query: {}, active_plugin: {:?}",
            query,
            self.active_plugin_id
        );

        let results = if let Some(ref plugin_id) = self.active_plugin_id {
            if query.is_empty() {
                Vec::new()
            } else {
                self.plugin_manager.search_plugin(plugin_id, query, 50)
            }
        } else {
            if query.starts_with('/') {
                self.handle_plugin_command(query)
            } else {
                self.plugin_manager.search_all(query, 50)
            }
        };

        log::info!("搜索结果数量: {}", results.len());

        // 添加高亮
        let mut results = results;
        for result in &mut results {
            let highlighted_title = crate::utils::fuzzy::highlight_matches(query, &result.title);
            result.highlighted_title = Some(highlighted_title);

            let highlighted_desc =
                crate::utils::fuzzy::highlight_matches(query, &result.description);
            result.highlighted_description = Some(highlighted_desc);
        }

        // 更新列表
        self.list_state.update(cx, |state, cx| {
            state.delegate_mut().update_from_search(results);
            cx.notify();
        });
    }

    /// 处理插件命令
    fn handle_plugin_command(&self, query: &str) -> Vec<SearchResult> {
        let query = query.trim_start_matches('/');

        if query.is_empty() {
            return Vec::new();
        }

        let parts: Vec<&str> = query.splitn(2, ' ').collect();
        let plugin_prefix = parts[0];

        let matches = self.plugin_manager.match_plugin_ids(plugin_prefix);
        matches
            .into_iter()
            .map(|id| {
                SearchResult::new(
                    format!("__plugin__:{}", id),
                    format!("/{}", id),
                    "按 Enter 选择此插件".to_string(),
                    ResultType::Custom("plugin".to_string()),
                    1000,
                    ActionData::Custom { plugin: "plugin_selector".to_string(), data: id },
                )
            })
            .collect()
    }

    /// 处理键盘事件
    fn handle_key_event(
        &mut self,
        event: &KeyDownEvent,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        let key = event.keystroke.key.as_str();

        if key == self.keybindings.close.to_lowercase().as_str() || key == "escape" {
            cx.emit(DismissEvent);
            return;
        }

        let items_count = self.list_state.read(cx).delegate().items_count();
        if items_count == 0 {
            return;
        }

        let current = self.list_state.read(cx).selected_index();

        if key == self.keybindings.navigate_up.to_lowercase().as_str() || key == "arrowup" {
            let new_index = if let Some(ix) = current {
                if ix.row > 0 {
                    Some(gpui_component::IndexPath::default().row(ix.row - 1))
                } else {
                    Some(gpui_component::IndexPath::default().row(items_count - 1))
                }
            } else {
                Some(gpui_component::IndexPath::default().row(0))
            };

            if let Some(ix) = new_index {
                self.list_state.update(cx, |state, cx| {
                    state.set_selected_index(Some(ix), window, cx);
                });
            }
            return;
        }

        if key == self.keybindings.navigate_down.to_lowercase().as_str() || key == "arrowdown" {
            let new_index = if let Some(ix) = current {
                if ix.row < items_count - 1 {
                    Some(gpui_component::IndexPath::default().row(ix.row + 1))
                } else {
                    Some(gpui_component::IndexPath::default().row(0))
                }
            } else {
                Some(gpui_component::IndexPath::default().row(0))
            };

            if let Some(ix) = new_index {
                self.list_state.update(cx, |state, cx| {
                    state.set_selected_index(Some(ix), window, cx);
                });
            }
            return;
        }

        if key == self.keybindings.confirm.to_lowercase().as_str() || key == "enter" {
            if let Some(ix) = current {
                let result_opt = {
                    let delegate = self.list_state.read(cx).delegate();
                    delegate.get_item(ix.row).cloned()
                };

                if let Some(result) = result_opt {
                    if result.id.starts_with("__plugin__:") {
                        if let ActionData::Custom { plugin: _, data } = &result.action {
                            let plugin_id = data.clone();
                            self.active_plugin_id = Some(plugin_id.clone());
                            self.list_state.update(cx, |state, cx| {
                                state.delegate_mut().set_active_plugin(Some(plugin_id.clone()));
                            });
                            log::info!("切换到插件: {}", plugin_id);
                            return;
                        }
                    }

                    log::info!("确认执行: {:?}", result);
                    self.execute_result(&result);
                    cx.emit(DismissEvent);
                }
            }
        }
    }

    /// 执行搜索结果
    fn execute_result(&self, result: &SearchResult) {
        // 处理插件选择器的特殊 case
        if result.id.starts_with("__plugin__:") {
            if let ActionData::Custom { plugin: _, data } = &result.action {
                log::info!("切换到插件: {}，请输入搜索内容", data);
                // 选择插件后不执行任何操作，让用户在搜索框中继续输入
                return;
            }
        }

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
}

impl Render for LauncherWindow {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let theme = cx.theme();

        // 获取列表中的结果数量
        let results_count = self.list_state.read(cx).delegate().items_count();

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
            .on_key_down(cx.listener(|this, event: &KeyDownEvent, window, cx| {
                this.handle_key_event(event, window, cx);
            }))
            // 列表（带搜索框）
            .child(List::new(&self.list_state).max_h(px(400.)).p_1())
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
                    .child(format!("{} 个结果", results_count))
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
