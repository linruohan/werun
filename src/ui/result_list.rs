/// 结果列表组件
///
/// 显示搜索结果列表，支持键盘导航
use gpui::*;
use gpui_component::prelude::*;
use gpui_component::theme::ActiveTheme;
use gpui_component::IconName;

use crate::core::search::SearchResult;
use crate::ui::result_item::ResultItemView;

/// 结果列表视图
pub struct ResultListView {
    /// 搜索结果列表
    results: Vec<SearchResult>,
    /// 当前选中索引
    selected_index: usize,
}

impl ResultListView {
    /// 创建新的结果列表视图
    pub fn new(results: &[SearchResult], selected_index: usize) -> Self {
        Self {
            results: results.to_vec(),
            selected_index,
        }
    }
}

impl RenderOnce for ResultListView {
    fn render(self, _window: &mut Window, cx: &mut AppContext) -> impl IntoElement {
        let theme = cx.theme();

        if self.results.is_empty() {
            // 空状态显示
            return div()
                .flex()
                .flex_col()
                .items_center()
                .justify_center()
                .flex_1()
                .gap_2()
                .child(
                    gpui_component::Icon::new(IconName::Search)
                        .large()
                        .text_color(theme.muted_foreground),
                )
                .child(
                    div()
                        .text_sm()
                        .text_color(theme.muted_foreground)
                        .child("输入关键词开始搜索..."),
                )
                .into_any_element();
        }

        div()
            .flex()
            .flex_col()
            .gap_1()
            .overflow_y_scroll()
            .children(self.results.into_iter().enumerate().map(|(index, result)| {
                let is_selected = index == self.selected_index;
                ResultItemView::new(result, is_selected)
            }))
            .into_any_element()
    }
}
