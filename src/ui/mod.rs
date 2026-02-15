/// UI 模块
///
/// 提供启动器的所有用户界面组件
pub mod launcher_window;
pub mod themes;
use gpui::{
    actions, div, px, size, Action, AnyView, App, AppContext, Bounds, Context, FocusHandle,
    Focusable, InteractiveElement, IntoElement, KeyBinding, ParentElement, Pixels, Render,
    SharedString, Size, Styled, TitlebarOptions, Window, WindowBackgroundAppearance, WindowBounds,
    WindowKind, WindowOptions,
};
use gpui_component::{notification::Notification, scroll::ScrollbarShow, v_flex, Root, WindowExt};
use serde::Deserialize;

use crate::core::config_manager::global_config;

/// 版本信息
pub const VERSION: &str = env!("CARGO_PKG_VERSION");
pub const NAME: &str = env!("CARGO_PKG_NAME");

actions!(ui, [
    About,
    Open,
    Quit,
    ToggleSearch,
    TestAction,
    Tab,
    TabPrev,
    ShowPanelInfo,
    ToggleListActiveHighlight
]);
#[derive(Action, Clone, PartialEq, Eq, Deserialize)]
#[action(namespace = ui, no_json)]
pub struct SelectScrollbarShow(ScrollbarShow);

#[derive(Action, Clone, PartialEq, Eq, Deserialize)]
#[action(namespace = ui, no_json)]
pub struct SelectLocale(SharedString);

#[derive(Action, Clone, PartialEq, Eq, Deserialize)]
#[action(namespace = ui, no_json)]
pub struct SelectFont(usize);

#[derive(Action, Clone, PartialEq, Eq, Deserialize)]
#[action(namespace = ui, no_json)]
pub struct SelectRadius(usize);

pub fn init(cx: &mut App) {
    gpui_component::init(cx);
    themes::init(cx);
    cx.bind_keys([
        KeyBinding::new("/", ToggleSearch, None),
        #[cfg(target_os = "macos")]
        KeyBinding::new("cmd-o", Open, None),
        #[cfg(not(target_os = "macos"))]
        KeyBinding::new("ctrl-o", Open, None),
        #[cfg(target_os = "macos")]
        KeyBinding::new("cmd-q", Quit, None),
        #[cfg(not(target_os = "macos"))]
        KeyBinding::new("alt-f4", Quit, None),
    ]);

    cx.on_action(|_: &Quit, cx: &mut App| {
        cx.quit();
    });

    cx.on_action(|_: &About, cx: &mut App| {
        if let Some(window) = cx.active_window().and_then(|w| w.downcast::<Root>()) {
            cx.defer(move |cx| {
                window
                    .update(cx, |root, window, cx| {
                        root.push_notification(
                            "GPUI Component Storybook\nVersion 0.1.0",
                            window,
                            cx,
                        );
                    })
                    .unwrap();
            });
        }
    });

    cx.activate(true);
}

pub fn create_new_window<F, E>(title: &str, crate_view_fn: F, cx: &mut App)
where
    E: Into<AnyView>,
    F: FnOnce(&mut Window, &mut App) -> E + Send + 'static,
{
    create_new_window_with_size(title, None, crate_view_fn, cx);
}

pub fn create_new_window_with_size<F, E>(
    title: &str,
    window_size: Option<Size<Pixels>>,
    crate_view_fn: F,
    cx: &mut App,
) where
    E: Into<AnyView>,
    F: FnOnce(&mut Window, &mut App) -> E + Send + 'static,
{
    let mut window_size = window_size.unwrap_or(size(px(600.0), px(400.0)));
    if let Some(display) = cx.primary_display() {
        let display_size = display.bounds().size;
        window_size.width = window_size.width.min(display_size.width * 0.85);
        window_size.height = window_size.height.min(display_size.height * 0.85);
    }
    let _window_bounds = Bounds::centered(None, window_size, cx);
    // 从配置中读取窗口大小
    let config = global_config().get_config();
    let window_width = px(config.window.width);
    let window_height = px(config.window.height);

    // 窗口选项配置
    let window_options = WindowOptions {
        window_bounds: Some(WindowBounds::Windowed(Bounds::centered(
            None,
            size(window_width, window_height),
            cx,
        ))),
        titlebar: Some(TitlebarOptions {
            title: None,
            appears_transparent: true,
            traffic_light_position: None,
        }),
        window_background: WindowBackgroundAppearance::Transparent,
        kind: WindowKind::PopUp,
        display_id: None,
        window_min_size: Some(size(px(600.0), px(400.0))),
        focus: true,
        show: true,
        is_movable: false,
        app_id: Some("werun".to_string()),
        ..Default::default()
    };
    let title = SharedString::from(title.to_string());

    cx.spawn(async move |cx| {
        let window = cx
            .open_window(window_options, |window, cx| {
                let view = crate_view_fn(window, cx);
                let story_root = cx.new(|cx| RootView::new(title.clone(), view, window, cx));

                // Set focus to the StoryRoot to enable its actions.
                let focus_handle = story_root.focus_handle(cx);
                window.defer(cx, move |window, cx| {
                    focus_handle.focus(window, cx);
                });

                cx.new(|cx| Root::new(story_root, window, cx))
            })
            .expect("failed to open window");

        window
            .update(cx, |_, window, _| {
                window.activate_window();
                window.set_window_title(&title);
            })
            .expect("failed to update window");

        Ok::<_, anyhow::Error>(())
    })
    .detach();
}

struct RootView {
    focus_handle: FocusHandle,
    view: AnyView,
}

impl RootView {
    pub fn new(
        _title: impl Into<SharedString>,
        view: impl Into<AnyView>,
        _window: &mut Window,
        cx: &mut Context<Self>,
    ) -> Self {
        Self { focus_handle: cx.focus_handle(), view: view.into() }
    }

    fn on_action_panel_info(
        &mut self,
        _: &ShowPanelInfo,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        struct Info;
        let note = Notification::new().message("You have clicked panel info.").id::<Info>();
        window.push_notification(note, cx);
    }

    fn on_action_toggle_search(
        &mut self,
        _: &ToggleSearch,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        cx.propagate();
        if window.has_focused_input(cx) {
            return;
        }

        struct Search;
        let note = Notification::new().message("You have toggled search.").id::<Search>();
        window.push_notification(note, cx);
    }
}

impl Focusable for RootView {
    fn focus_handle(&self, _: &App) -> FocusHandle {
        self.focus_handle.clone()
    }
}

impl Render for RootView {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let sheet_layer = Root::render_sheet_layer(window, cx);
        let dialog_layer = Root::render_dialog_layer(window, cx);
        let notification_layer = Root::render_notification_layer(window, cx);

        div()
            .id("ui-root")
            .on_action(cx.listener(Self::on_action_panel_info))
            .on_action(cx.listener(Self::on_action_toggle_search))
            .size_full()
            .child(
                v_flex()
                    .size_full()
                    .child(
                        div()
                            .track_focus(&self.focus_handle)
                            .flex_1()
                            .overflow_hidden()
                            .child(self.view.clone()),
                    )
                    .children(sheet_layer)
                    .children(dialog_layer)
                    .children(notification_layer),
            )
    }
}
