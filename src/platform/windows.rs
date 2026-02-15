/// Windows 平台特定功能
///
/// 提供全局快捷键、窗口管理等 Windows API 封装
use std::sync::Mutex;
use windows::Win32::Foundation::{HINSTANCE, HWND, LPARAM, LRESULT, WPARAM};
use windows::Win32::UI::Input::KeyboardAndMouse::{
    RegisterHotKey, UnregisterHotKey, HOT_KEY_MODIFIERS, MOD_ALT, VK_SPACE,
};
use windows::Win32::UI::WindowsAndMessaging::{
    CreateWindowExW, DefWindowProcW, DispatchMessageW, GetMessageW, RegisterClassW,
    TranslateMessage, CS_HREDRAW, CS_VREDRAW, CW_USEDEFAULT, MSG, WM_HOTKEY, WNDCLASSW,
    WS_EX_NOACTIVATE, WS_OVERLAPPED,
};

/// 全局快捷键管理器
pub struct GlobalHotkeyManager {
    /// 窗口句柄
    hwnd: HWND,
    /// 是否已注册
    registered: bool,
}

/// 热键 ID
const HOTKEY_ID: i32 = 1;

/// 全局窗口类名
const WINDOW_CLASS_NAME: &str = "WeRunHotkeyWindow";

/// 全局回调函数（使用 Mutex 包装以支持线程安全）
static HOTKEY_CALLBACK: Mutex<Option<Box<dyn Fn() + Send + Sync>>> = Mutex::new(None);

impl GlobalHotkeyManager {
    /// 创建新的全局快捷键管理器
    pub fn new() -> anyhow::Result<Self> {
        let hwnd = Self::create_message_window()?;

        Ok(Self {
            hwnd,
            registered: false,
        })
    }

    /// 注册 Alt+Space 全局快捷键
    pub fn register_alt_space<F>(&mut self, callback: F) -> anyhow::Result<()>
    where
        F: Fn() + Send + Sync + 'static,
    {
        if self.registered {
            return Ok(());
        }

        // 存储回调函数
        if let Ok(mut guard) = HOTKEY_CALLBACK.lock() {
            *guard = Some(Box::new(callback));
        }

        // 注册全局快捷键 Alt+Space
        unsafe {
            RegisterHotKey(
                self.hwnd,
                HOTKEY_ID,
                HOT_KEY_MODIFIERS(MOD_ALT.0),
                VK_SPACE.0 as u32,
            )?;
        }

        self.registered = true;
        log::info!("全局快捷键 Alt+Space 注册成功");

        // 启动消息循环（在单独线程中）
        std::thread::spawn(move || {
            Self::message_loop();
        });

        Ok(())
    }

    /// 注销快捷键
    pub fn unregister(&mut self) -> anyhow::Result<()> {
        if !self.registered {
            return Ok(());
        }

        unsafe {
            UnregisterHotKey(self.hwnd, HOTKEY_ID)?;
        }

        self.registered = false;
        log::info!("全局快捷键已注销");

        Ok(())
    }

    /// 创建消息窗口（用于接收快捷键消息）
    fn create_message_window() -> anyhow::Result<HWND> {
        unsafe {
            // 获取模块句柄
            let hinstance: HINSTANCE =
                windows::Win32::System::LibraryLoader::GetModuleHandleW(None)?.into();

            // 注册窗口类
            let class_name: Vec<u16> = WINDOW_CLASS_NAME
                .encode_utf16()
                .chain(std::iter::once(0))
                .collect();

            let window_title: Vec<u16> = "WeRun Hotkey Window"
                .encode_utf16()
                .chain(std::iter::once(0))
                .collect();

            let wnd_class = WNDCLASSW {
                lpfnWndProc: Some(Self::window_proc),
                hInstance: hinstance,
                lpszClassName: windows::core::PCWSTR(class_name.as_ptr()),
                style: CS_HREDRAW | CS_VREDRAW,
                ..Default::default()
            };

            RegisterClassW(&wnd_class);

            // 创建消息窗口
            let hwnd = CreateWindowExW(
                WS_EX_NOACTIVATE,
                windows::core::PCWSTR(class_name.as_ptr()),
                windows::core::PCWSTR(window_title.as_ptr()),
                WS_OVERLAPPED,
                CW_USEDEFAULT,
                CW_USEDEFAULT,
                CW_USEDEFAULT,
                CW_USEDEFAULT,
                None,
                None,
                hinstance,
                None,
            )?;

            Ok(hwnd)
        }
    }

    /// 窗口过程函数
    unsafe extern "system" fn window_proc(
        hwnd: HWND,
        msg: u32,
        wparam: WPARAM,
        lparam: LPARAM,
    ) -> LRESULT {
        match msg {
            WM_HOTKEY => {
                // 快捷键被触发
                if wparam.0 as i32 == HOTKEY_ID {
                    log::debug!("全局快捷键 Alt+Space 被触发");

                    // 调用回调函数
                    if let Ok(guard) = HOTKEY_CALLBACK.lock() {
                        if let Some(callback) = guard.as_ref() {
                            callback();
                        }
                    }
                }
                LRESULT(0)
            }
            _ => DefWindowProcW(hwnd, msg, wparam, lparam),
        }
    }

    /// 消息循环
    fn message_loop() {
        unsafe {
            let mut msg: MSG = std::mem::zeroed();

            while GetMessageW(&mut msg, None, 0, 0).into() {
                let _ = TranslateMessage(&msg);
                DispatchMessageW(&msg);
            }
        }
    }
}

impl Drop for GlobalHotkeyManager {
    fn drop(&mut self) {
        let _ = self.unregister();
    }
}
