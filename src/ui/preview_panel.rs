/// 预览面板组件
///
/// 显示选中结果的详细信息
use gpui::*;
use gpui_component::prelude::*;
use gpui_component::theme::ActiveTheme;
use gpui_component::IconName;

use crate::core::search::{ResultType, SearchResult};

/// 预览面板视图
pub struct PreviewPanelView {
    /// 当前选中的结果
    result: Option<SearchResult>,
}

impl PreviewPanelView {
    /// 创建新的预览面板
    pub fn new(result: Option<SearchResult>) -> Self {
        Self { result }
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
            ResultType::Settings => IconName::Settings,
            ResultType::Custom(_) => IconName::Box,
        }
    }
}

impl RenderOnce for PreviewPanelView {
    fn render(self, _window: &mut Window, cx: &mut AppContext) -> impl IntoElement {
        let theme = cx.theme();

        let content = if let Some(result) = self.result {
            div()
                .flex()
                .flex_col()
                .gap_4()
                // 标题区域
                .child(
                    div()
                        .flex()
                        .flex_row()
                        .items_center()
                        .gap_3()
                        .child(
                            div()
                                .flex()
                                .items_center()
                                .justify_center()
                                .w_12()
                                .h_12()
                                .rounded_lg()
                                .bg(theme.secondary)
                                .child(
                                    gpui_component::Icon::new(Self::get_result_icon(
                                        &result.result_type,
                                    ))
                                    .large()
                                    .text_color(theme.foreground),
                                ),
                        )
                        .child(
                            div()
                                .flex()
                                .flex_col()
                                .gap_1()
                                .child(
                                    div()
                                        .text_lg()
                                        .font_weight(FontWeight::SEMIBOLD)
                                        .text_color(theme.foreground)
                                        .child(result.title),
                                )
                                .child(
                                    div()
                                        .text_sm()
                                        .text_color(theme.muted_foreground)
                                        .child(result.description),
                                ),
                        ),
                )
                // 分隔线
                .child(div().h_px().w_full().bg(theme.border))
                // 详情信息
                .child(
                    div()
                        .flex()
                        .flex_col()
                        .gap_2()
                        .child(
                            div()
                                .text_sm()
                                .font_weight(FontWeight::MEDIUM)
                                .text_color(theme.foreground)
                                .child("详细信息"),
                        )
                        .child(
                            div()
                                .text_sm()
                                .text_color(theme.muted_foreground)
                                .child(format!("ID: {}", result.id)),
                        )
                        .child(
                            div()
                                .text_sm()
                                .text_color(theme.muted_foreground)
                                .child(format!("类型: {:?}", result.result_type)),
                        ),
                )
        } else {
            div()
                .flex()
                .flex_col()
                .items_center()
                .justify_center()
                .h_full()
                .gap_2()
                .child(
                    gpui_component::Icon::new(IconName::Info)
                        .large()
                        .text_color(theme.muted_foreground),
                )
                .child(
                    div()
                        .text_sm()
                        .text_color(theme.muted_foreground)
                        .child("选择一个项目查看详情"),
                )
        };

        div()
            .w(px(280.0))
            .h_full()
            .p_4()
            .bg(theme.secondary)
            .border_l_1()
            .border_color(theme.border)
            .child(content)
    }
}
