use std::sync::{Arc, Mutex};

use anyhow::Result;

use crate::core::plugin::Plugin;
use crate::core::search::{ActionData, ResultType, SearchResult};

#[derive(Clone, Debug)]
pub struct WindowInfo {
    pub hwnd: isize,
    pub title: String,
    pub process_name: String,
}

pub struct WindowSwitcherPlugin {
    enabled: bool,
    windows: Arc<Mutex<Vec<WindowInfo>>>,
}

impl WindowSwitcherPlugin {
    pub fn new() -> Self {
        Self { enabled: true, windows: Arc::new(Mutex::new(Vec::new())) }
    }

    fn get_windows(&self) -> Vec<WindowInfo> {
        #[cfg(target_os = "windows")]
        {
            self.enumerate_windows()
        }
        #[cfg(not(target_os = "windows"))]
        {
            Vec::new()
        }
    }

    #[cfg(target_os = "windows")]
    fn enumerate_windows(&self) -> Vec<WindowInfo> {
        use std::ffi::OsString;
        use std::os::windows::ffi::OsStringExt;
        use windows::Win32::Foundation::{BOOL, HWND, LPARAM};
        use windows::Win32::UI::WindowsAndMessaging::{
            EnumWindows, GetWindowTextW, GetWindowThreadProcessId, IsWindowVisible,
        };

        let _windows: Vec<WindowInfo> = Vec::new();

        unsafe extern "system" fn enum_windows_callback(hwnd: HWND, lparam: LPARAM) -> BOOL {
            let windows = &mut *(lparam.0 as *mut Vec<WindowInfo>);

            if IsWindowVisible(hwnd).as_bool() {
                let mut title_buf = [0u16; 512];
                let len = GetWindowTextW(hwnd, &mut title_buf);

                if len > 0 {
                    let title = OsString::from_wide(&title_buf[..len as usize])
                        .to_string_lossy()
                        .to_string();

                    if !title.is_empty() && title != "Program Manager" {
                        let mut process_id: u32 = 0;
                        GetWindowThreadProcessId(hwnd, Some(&mut process_id));

                        let process_name = if let Ok(process) =
                            std::process::Command::new("tasklist")
                                .args([
                                    "/FI",
                                    &format!("PID eq {}", process_id),
                                    "/FO",
                                    "CSV",
                                    "/NH",
                                ])
                                .output()
                        {
                            let output = String::from_utf8_lossy(&process.stdout);
                            output
                                .split(',')
                                .next()
                                .map(|s| s.trim_matches('"').to_string())
                                .unwrap_or_else(|| "Unknown".to_string())
                        } else {
                            "Unknown".to_string()
                        };

                        windows.push(WindowInfo { hwnd: hwnd.0 as isize, title, process_name });
                    }
                }
            }

            BOOL(1)
        }

        unsafe {
            let mut windows_vec: Vec<WindowInfo> = Vec::new();
            let ptr = LPARAM(&mut windows_vec as *mut _ as isize);

            let _ = EnumWindows(Some(enum_windows_callback), ptr);

            windows_vec
        }
    }

    fn switch_to_window(&self, hwnd: isize) -> Result<()> {
        #[cfg(target_os = "windows")]
        {
            use windows::Win32::Foundation::HWND;
            use windows::Win32::UI::WindowsAndMessaging::{
                SetForegroundWindow, ShowWindow, SW_RESTORE,
            };

            unsafe {
                let _ = ShowWindow(HWND(hwnd as *mut _), SW_RESTORE);
                let _ = SetForegroundWindow(HWND(hwnd as *mut _));
            }
        }
        Ok(())
    }

    fn close_window(&self, hwnd: isize) -> Result<()> {
        #[cfg(target_os = "windows")]
        {
            use windows::Win32::Foundation::HWND;
            use windows::Win32::UI::WindowsAndMessaging::PostMessageW;
            use windows::Win32::UI::WindowsAndMessaging::WM_CLOSE;

            unsafe {
                let _ = PostMessageW(HWND(hwnd as *mut _), WM_CLOSE, None, None);
            }
        }
        Ok(())
    }

    fn minimize_window(&self, hwnd: isize) -> Result<()> {
        #[cfg(target_os = "windows")]
        {
            use windows::Win32::Foundation::HWND;
            use windows::Win32::UI::WindowsAndMessaging::ShowWindow;
            use windows::Win32::UI::WindowsAndMessaging::SW_MINIMIZE;

            unsafe {
                let _ = ShowWindow(HWND(hwnd as *mut _), SW_MINIMIZE);
            }
        }
        Ok(())
    }

    fn maximize_window(&self, hwnd: isize) -> Result<()> {
        #[cfg(target_os = "windows")]
        {
            use windows::Win32::Foundation::HWND;
            use windows::Win32::UI::WindowsAndMessaging::ShowWindow;
            use windows::Win32::UI::WindowsAndMessaging::SW_MAXIMIZE;

            unsafe {
                let _ = ShowWindow(HWND(hwnd as *mut _), SW_MAXIMIZE);
            }
        }
        Ok(())
    }
}

impl Plugin for WindowSwitcherPlugin {
    fn id(&self) -> &str {
        "window_switcher"
    }

    fn name(&self) -> &str {
        "窗口切换"
    }

    fn description(&self) -> &str {
        "快速切换和管理打开的窗口"
    }

    fn version(&self) -> &str {
        "0.1.0"
    }

    fn is_enabled(&self) -> bool {
        self.enabled
    }

    fn set_enabled(&mut self, enabled: bool) {
        self.enabled = enabled;
    }

    fn initialize(&mut self) -> Result<()> {
        log::info!("初始化窗口切换器插件...");

        let windows = self.get_windows();

        if let Ok(mut guard) = self.windows.lock() {
            *guard = windows.clone();
            log::info!("已发现 {} 个窗口", guard.len());
        }

        Ok(())
    }

    fn search(&self, query: &str, limit: usize) -> Result<Vec<SearchResult>> {
        let mut results = Vec::new();
        let query_lower = query.to_lowercase();

        if query.is_empty() {
            let windows = self.get_windows();

            if let Ok(mut guard) = self.windows.lock() {
                *guard = windows.clone();
            }

            for window in windows.iter().take(limit) {
                results.push(SearchResult::new(
                    format!("window_switcher:{}", window.hwnd),
                    window.title.clone(),
                    format!("进程: {}", window.process_name),
                    ResultType::Custom("window".to_string()),
                    0,
                    ActionData::Custom {
                        plugin: "window_switcher".to_string(),
                        data: window.hwnd.to_string(),
                    },
                ));
            }
        } else {
            for window in self.windows.lock().unwrap().iter() {
                if window.title.to_lowercase().contains(&query_lower)
                    || window.process_name.to_lowercase().contains(&query_lower)
                {
                    results.push(SearchResult::new(
                        format!("window_switcher:{}", window.hwnd),
                        window.title.clone(),
                        format!("进程: {}", window.process_name),
                        ResultType::Custom("window".to_string()),
                        50,
                        ActionData::Custom {
                            plugin: "window_switcher".to_string(),
                            data: window.hwnd.to_string(),
                        },
                    ));

                    if results.len() >= limit {
                        break;
                    }
                }
            }
        }

        Ok(results)
    }

    fn execute(&self, result: &SearchResult) -> Result<()> {
        if let ActionData::Custom { data, .. } = &result.action {
            if let Ok(hwnd) = data.parse::<isize>() {
                self.switch_to_window(hwnd)?;
            }
        }
        Ok(())
    }

    fn refresh(&mut self) -> Result<()> {
        self.initialize()
    }
}

impl Default for WindowSwitcherPlugin {
    fn default() -> Self {
        Self::new()
    }
}
