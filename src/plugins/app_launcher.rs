use std::{
    process::Command,
    sync::{Arc, Mutex},
};

use anyhow::Result;

/// 应用启动插件
///
/// 扫描并启动 Windows 应用程序
use crate::core::plugin::Plugin;
use crate::core::search::{ActionData, ResultType, SearchResult};

/// 应用信息
#[derive(Clone, Debug)]
pub struct AppInfo {
    /// 应用名称
    pub name: String,
    /// 应用路径
    pub path: String,
    /// 应用描述
    pub description: String,
    /// 图标路径
    pub icon: Option<String>,
}

/// 应用启动插件
pub struct AppLauncherPlugin {
    /// 是否启用
    enabled: bool,
    /// 已索引的应用列表
    apps: Arc<Mutex<Vec<AppInfo>>>,
}

impl AppLauncherPlugin {
    /// 创建新的应用启动插件
    pub fn new() -> Self {
        Self { enabled: true, apps: Arc::new(Mutex::new(Vec::new())) }
    }

    /// 扫描开始菜单中的应用
    fn scan_start_menu(&self) -> Result<Vec<AppInfo>> {
        let mut apps = Vec::new();

        // 获取开始菜单路径
        let start_menu_paths = [
            dirs::data_dir()
                .map(|p| p.join("Microsoft\\Windows\\Start Menu\\Programs"))
                .unwrap_or_default(),
            std::path::PathBuf::from("C:\\ProgramData\\Microsoft\\Windows\\Start Menu\\Programs"),
        ];

        for path in &start_menu_paths {
            if path.exists() {
                self.scan_directory(path, &mut apps)?;
            }
        }

        Ok(apps)
    }

    /// 递归扫描目录
    fn scan_directory(&self, path: &std::path::Path, apps: &mut Vec<AppInfo>) -> Result<()> {
        if let Ok(entries) = std::fs::read_dir(path) {
            for entry in entries.flatten() {
                let path = entry.path();

                if path.is_dir() {
                    // 递归扫描子目录
                    let _ = self.scan_directory(&path, apps);
                } else if path.extension().map(|e| e == "lnk").unwrap_or(false) {
                    // 解析快捷方式
                    if let Some(app) = self.parse_shortcut(&path) {
                        apps.push(app);
                    }
                } else if path.extension().map(|e| e == "exe").unwrap_or(false) {
                    // 可执行文件
                    let name = path
                        .file_stem()
                        .map(|s| s.to_string_lossy().to_string())
                        .unwrap_or_default();

                    apps.push(AppInfo {
                        name,
                        path: path.to_string_lossy().to_string(),
                        description: "应用程序".to_string(),
                        icon: None,
                    });
                }
            }
        }

        Ok(())
    }

    /// 解析快捷方式文件
    fn parse_shortcut(&self, path: &std::path::Path) -> Option<AppInfo> {
        // TODO: 使用 lnk crate 解析快捷方式
        // 目前简化处理，仅提取文件名
        let name = path.file_stem().map(|s| s.to_string_lossy().to_string()).unwrap_or_default();

        Some(AppInfo {
            name,
            path: path.to_string_lossy().to_string(),
            description: "快捷方式".to_string(),
            icon: None,
        })
    }

    /// 启动应用
    fn launch_app(&self, path: &str) -> Result<()> {
        // 解析快捷方式获取实际目标
        let target_path = if path.ends_with(".lnk") {
            // TODO: 解析 .lnk 文件获取目标路径
            path.to_string()
        } else {
            path.to_string()
        };

        // 启动应用
        Command::new("cmd").args(["/c", "start", "", &target_path]).spawn()?;

        Ok(())
    }
}

impl Plugin for AppLauncherPlugin {
    fn id(&self) -> &str {
        "app_launcher"
    }

    fn name(&self) -> &str {
        "应用启动器"
    }

    fn description(&self) -> &str {
        "扫描并启动 Windows 应用程序"
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
        log::info!("初始化应用启动插件...");

        // 扫描应用
        let apps = self.scan_start_menu()?;

        // 存储应用列表
        if let Ok(mut guard) = self.apps.lock() {
            *guard = apps;
            log::info!("已索引 {} 个应用", guard.len());
        }

        Ok(())
    }

    fn search(&self, query: &str, limit: usize) -> Result<Vec<SearchResult>> {
        let apps = self.apps.lock().unwrap();
        let mut results = Vec::new();

        for app in apps.iter() {
            // 简单的模糊匹配
            if app.name.to_lowercase().contains(&query.to_lowercase()) {
                results.push(SearchResult {
                    id: format!("app:{}", app.path),
                    title: app.name.clone(),
                    description: app.description.clone(),
                    icon: app.icon.clone(),
                    result_type: ResultType::Application,
                    score: 100, // TODO: 实现更好的评分算法
                    action: ActionData::LaunchApp { path: app.path.clone(), args: Vec::new() },
                });

                if results.len() >= limit {
                    break;
                }
            }
        }

        Ok(results)
    }

    fn execute(&self, result: &SearchResult) -> Result<()> {
        if let ActionData::LaunchApp { path, .. } = &result.action {
            self.launch_app(path)?;
        }
        Ok(())
    }

    fn refresh(&mut self) -> Result<()> {
        self.initialize()
    }
}

impl Default for AppLauncherPlugin {
    fn default() -> Self {
        Self::new()
    }
}
