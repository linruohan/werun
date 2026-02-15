/// 搜索栏组件
/// 
/// 提供搜索输入框和搜索图标

use gpui::*;
use gpui_component::prelude::*;
use gpui_component::{input::InputState, theme::ActiveTheme};
use gpui_component::IconName;

/// 搜索栏视图
pub struct SearchBarView {
    /// 输入状态引用
    input_state: Entity<InputState>,
}

impl SearchBarView {
    /// 创建新的搜索栏视图
    pub fn new(input_state: &Entity<InputState>) -> Self {
        Self {
            input_state: input_state.clone(),
        }
    }
}

impl RenderOnce for SearchBarView {
    fn render(self, _window: &mut Window, cx: &mut AppContext) -> impl IntoElement {
        let theme = cx.theme();
        
        div()
            .flex()
            .flex_row()
            .items_center()
            .gap_2()
            .px_3()
            .py_2()
            .bg(theme.secondary)
            .rounded_lg()
            .border_1()
            .border_color(theme.border)
            // 搜索图标
            .child(
                gpui_component::Icon::new(IconName::Search)
                    .small()
                    .text_color(theme.muted_foreground)
            )
            // 输入框
            .child(
                gpui_component::input::Input::new(&self.input_state)
                    .size(gpui_component::input::InputSize::Large)
                    .cleanable()
            )
    }
}
