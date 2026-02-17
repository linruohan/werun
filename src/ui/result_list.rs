use crate::core::plugin::PluginManager;
use crate::core::search::{ResultType, SearchResult};
use gpui::*;
use gpui_component::list::{ListDelegate, ListItem, ListState};
use gpui_component::theme::ActiveTheme;
use gpui_component::IconName;
use gpui_component::IndexPath;
use gpui_component::Sizable;
use std::sync::Arc;

pub struct ResultListDelegate {
    pub items: Vec<SearchResult>,
    pub selected_index: Option<usize>,
    search_query: String,
    plugin_manager: Option<Arc<PluginManager>>,
}

impl ResultListDelegate {
    pub fn new(items: Vec<SearchResult>) -> Self {
        Self { items, selected_index: None, search_query: String::new(), plugin_manager: None }
    }

    pub fn with_plugin_manager(mut self, manager: Arc<PluginManager>) -> Self {
        self.plugin_manager = Some(manager);
        self
    }

    pub fn set_items(&mut self, items: Vec<SearchResult>) {
        self.items = items;
        self.selected_index = None;
    }

    pub fn items_count(&self) -> usize {
        self.items.len()
    }

    pub fn get_item(&self, index: usize) -> Option<&SearchResult> {
        self.items.get(index)
    }

    pub fn update_from_search(&mut self, results: Vec<SearchResult>) {
        self.items = results;
        self.selected_index = None;
    }

    fn perform_search_internal(&mut self, query: &str) {
        if let Some(manager) = &self.plugin_manager {
            let mut results = manager.search_all(query, 50);

            for result in &mut results {
                let highlighted_title =
                    crate::utils::fuzzy::highlight_matches(query, &result.title);
                result.highlighted_title = Some(highlighted_title);

                let highlighted_desc =
                    crate::utils::fuzzy::highlight_matches(query, &result.description);
                result.highlighted_description = Some(highlighted_desc);
            }

            self.items = results;
            self.selected_index = None;
        }
    }
}

impl ListDelegate for ResultListDelegate {
    type Item = ListItem;

    fn items_count(&self, _section: usize, _cx: &App) -> usize {
        self.items.len()
    }

    fn render_item(
        &mut self,
        ix: IndexPath,
        _window: &mut Window,
        cx: &mut Context<ListState<Self>>,
    ) -> Option<Self::Item> {
        let is_selected = Some(ix.row) == self.selected_index;
        let theme = cx.theme().clone();

        self.items.get(ix.row).map(|item| {
            let type_name = match &item.result_type {
                ResultType::Application => "应用",
                ResultType::File => "文件",
                ResultType::Folder => "文件夹",
                ResultType::Command => "命令",
                ResultType::Calculator => "计算",
                ResultType::Clipboard => "剪贴板",
                ResultType::Settings => "设置",
                ResultType::Custom(_) => "其他",
            };

            let icon_name = match &item.result_type {
                ResultType::Application => IconName::AppWindow,
                ResultType::File => IconName::File,
                ResultType::Folder => IconName::Folder,
                ResultType::Command => IconName::Terminal,
                ResultType::Calculator => IconName::Calculator,
                ResultType::Clipboard => IconName::Clipboard,
                ResultType::Settings => IconName::Settings,
                ResultType::Custom(_) => IconName::Search,
            };

            let bg_color = if is_selected { theme.accent } else { theme.background };
            let text_color = if is_selected { theme.accent_foreground } else { theme.foreground };
            let muted_color = if is_selected {
                theme.accent_foreground.opacity(0.7)
            } else {
                theme.muted_foreground
            };

            ListItem::new(ix)
                .child(
                    div()
                        .flex()
                        .flex_row()
                        .items_center()
                        .gap_3()
                        .px_3()
                        .py_2()
                        .rounded_md()
                        .bg(bg_color)
                        .child(
                            div()
                                .flex()
                                .items_center()
                                .justify_center()
                                .w_8()
                                .h_8()
                                .rounded_md()
                                .bg(if is_selected {
                                    theme.accent_foreground.opacity(0.2)
                                } else {
                                    theme.secondary
                                })
                                .child(
                                    gpui_component::Icon::new(icon_name)
                                        .small()
                                        .text_color(text_color),
                                ),
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
                                        .child(item.title.clone()),
                                )
                                .child(
                                    div()
                                        .text_xs()
                                        .text_color(muted_color)
                                        .child(item.description.clone()),
                                ),
                        )
                        .child(
                            div()
                                .px_2()
                                .py_0()
                                .rounded_full()
                                .text_xs()
                                .bg(if is_selected {
                                    theme.accent_foreground.opacity(0.2)
                                } else {
                                    theme.secondary
                                })
                                .text_color(muted_color)
                                .child(type_name),
                        ),
                )
                .selected(is_selected)
        })
    }

    fn set_selected_index(
        &mut self,
        ix: Option<IndexPath>,
        _window: &mut Window,
        _cx: &mut Context<ListState<Self>>,
    ) {
        self.selected_index = ix.map(|i| i.row);
    }

    fn perform_search(
        &mut self,
        query: &str,
        _window: &mut Window,
        cx: &mut Context<ListState<Self>>,
    ) -> Task<()> {
        self.search_query = query.to_string();
        self.perform_search_internal(query);
        cx.notify();
        Task::ready(())
    }
}
