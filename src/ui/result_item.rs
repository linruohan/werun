use crate::core::search::{ResultType, SearchResult};
/// 结果项组件
///
/// 显示单个搜索结果
use gpui::*;
use gpui_component::theme::ActiveTheme;
use gpui_component::IconName;
use gpui_component::Sizable;

/// 结果项视图
pub struct ResultItemView {
    /// 结果数据
    result: SearchResult,
    /// 是否被选中
    is_selected: bool,
}

impl ResultItemView {
    /// 创建新的结果项视图
    pub fn new(result: SearchResult, is_selected: bool) -> Self {
        Self { result, is_selected }
    }

    /// 获取结果类型的显示名称
    fn type_name(&self) -> &'static str {
        match &self.result.result_type {
            ResultType::Application => "应用",
            ResultType::File => "文件",
            ResultType::Folder => "文件夹",
            ResultType::Command => "命令",
            ResultType::Calculator => "计算",
            ResultType::Clipboard => "剪贴板",
            ResultType::Settings => "设置",
            ResultType::Custom(_) => "其他",
        }
    }

    /// 获取结果类型的图标
    fn type_icon(&self) -> IconName {
        match &self.result.result_type {
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

impl RenderOnce for ResultItemView {
    fn render(self, _window: &mut Window, cx: &mut App) -> impl IntoElement {
        let theme = cx.theme().clone();

        // 根据选中状态设置样式
        let bg_color = if self.is_selected { theme.accent } else { theme.background };

        let text_color = if self.is_selected { theme.accent_foreground } else { theme.foreground };

        let muted_color = if self.is_selected {
            theme.accent_foreground.opacity(0.7)
        } else {
            theme.muted_foreground
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
            // 图标
            .child(
                div()
                    .flex()
                    .items_center()
                    .justify_center()
                    .w_8()
                    .h_8()
                    .rounded_md()
                      .bg(if self.is_selected {
                            theme.accent_foreground.opacity(0.2)
                        } else {
                            theme.secondary
                        })
                    .child(
                        gpui_component::Icon::new(self.type_icon())
                            .small()
                            .text_color(text_color)
                    )
            )
            // 内容
            .child(
                div()
                    .flex()
                    .flex_col()
                    .flex_1()
                    .gap_1()
                    // 标题
                    .child(
                        div()
                            .text_sm()
                            .font_weight(FontWeight::MEDIUM)
                            .text_color(text_color)
                            .child(self.result.title.clone())
                    )
                    // 描述
                    .child(
                        div()
                            .text_xs()
                            .text_color(muted_color)
                            .child(self.result.description.clone())
                    )
            )
            // 类型标签
            .child(
                div()
                    .px_2()
                    .py_0()
                    .rounded_full()
                    .text_xs()
                    .bg(if self.is_selected {
                        theme.accent_foreground.opacity(0.2)
                    } else {
                        theme.secondary
                    })
                    .text_color(muted_color)
                    .child(self.type_name())
            )
    }
}
