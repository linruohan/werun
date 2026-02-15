use std::path::PathBuf;

use gpui::App;
/// 管理启动器的所有配置项
use serde::{Deserialize, Serialize};

/// 应用配置
#[derive(Clone, Debug, Serialize, Deserialize, Default)]
pub struct AppConfig {
    /// 窗口配置
    pub window: WindowConfig,
    /// 主题配置
    pub theme: ThemeConfig,
    /// 搜索配置
    pub search: SearchConfig,
    /// 快捷键配置
    pub keybindings: KeybindingsConfig,
    /// 插件配置
    pub plugins: PluginsConfig,
}

impl AppConfig {
    /// 加载配置文件
    pub fn load() -> anyhow::Result<Self> {
        let config_path = Self::config_path();

        if config_path.exists() {
            let content = std::fs::read_to_string(&config_path)?;
            let config: AppConfig = serde_json::from_str(&content)?;
            Ok(config)
        } else {
            // 创建默认配置
            let config = Self::default();
            config.save()?;
            Ok(config)
        }
    }

    /// 保存配置文件
    pub fn save(&self) -> anyhow::Result<()> {
        let config_path = Self::config_path();

        // 确保配置目录存在
        if let Some(parent) = config_path.parent() {
            std::fs::create_dir_all(parent)?;
        }

        let content = serde_json::to_string_pretty(self)?;
        std::fs::write(&config_path, content)?;

        Ok(())
    }

    /// 获取配置文件路径
    fn config_path() -> PathBuf {
        let app_data = dirs::config_dir().unwrap_or_else(|| PathBuf::from("."));
        app_data.join("werun").join("config.json")
    }
}

/// 窗口配置
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct WindowConfig {
    /// 窗口宽度
    pub width: f32,
    /// 窗口高度
    pub height: f32,
    /// 窗口透明度
    pub opacity: f32,
    /// 圆角半径
    pub border_radius: f32,
    /// 是否置顶
    pub always_on_top: bool,
    /// 失焦时自动隐藏
    pub hide_on_blur: bool,
    /// 显示动画时长 (毫秒)
    pub animation_duration_ms: u64,
}

impl Default for WindowConfig {
    fn default() -> Self {
        Self {
            width: 800.0,
            height: 500.0,
            opacity: 0.98,
            border_radius: 12.0,
            always_on_top: true,
            hide_on_blur: true,
            animation_duration_ms: 150,
        }
    }
}

/// 主题配置
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ThemeConfig {
    /// 当前主题名称
    pub current_theme: String,
    /// 是否跟随系统主题
    pub follow_system: bool,
}

impl Default for ThemeConfig {
    fn default() -> Self {
        Self { current_theme: "dark".to_string(), follow_system: true }
    }
}

/// 搜索配置
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct SearchConfig {
    /// 最大结果数
    pub max_results: usize,
    /// 搜索延迟 (毫秒)
    pub debounce_ms: u64,
    /// 是否显示文件搜索
    pub enable_file_search: bool,
    /// 文件搜索路径
    pub file_search_paths: Vec<String>,
    /// 忽略的文件模式
    pub file_ignore_patterns: Vec<String>,
}

impl Default for SearchConfig {
    fn default() -> Self {
        Self {
            max_results: 50,
            debounce_ms: 50,
            enable_file_search: true,
            file_search_paths: vec![
                dirs::desktop_dir().map(|p| p.to_string_lossy().to_string()).unwrap_or_default(),
                dirs::document_dir().map(|p| p.to_string_lossy().to_string()).unwrap_or_default(),
            ],
            file_ignore_patterns: vec![
                "*.tmp".to_string(),
                "*.log".to_string(),
                "node_modules".to_string(),
                ".git".to_string(),
            ],
        }
    }
}

/// 快捷键配置
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct KeybindingsConfig {
    /// 显示/隐藏启动器
    pub toggle_launcher: String,
    /// 向上导航
    pub navigate_up: String,
    /// 向下导航
    pub navigate_down: String,
    /// 确认选择
    pub confirm: String,
    /// 关闭窗口
    pub close: String,
}

impl Default for KeybindingsConfig {
    fn default() -> Self {
        Self {
            toggle_launcher: "Alt+Space".to_string(),
            navigate_up: "ArrowUp".to_string(),
            navigate_down: "ArrowDown".to_string(),
            confirm: "Enter".to_string(),
            close: "Escape".to_string(),
        }
    }
}

/// 插件配置
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct PluginsConfig {
    /// 启用的插件列表
    pub enabled: Vec<String>,
    /// 插件特定配置
    pub settings: serde_json::Value,
}

impl Default for PluginsConfig {
    fn default() -> Self {
        Self {
            enabled: vec![
                "app_launcher".to_string(),
                "calculator".to_string(),
                "clipboard".to_string(),
            ],
            settings: serde_json::json!({}),
        }
    }
}
