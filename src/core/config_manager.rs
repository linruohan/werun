use std::sync::{Arc, Mutex};

/// 配置管理器
///
/// 管理应用配置的加载、保存和实时更新
use crate::core::config::AppConfig;

/// 全局配置管理器
pub struct ConfigManager {
    config: Arc<Mutex<AppConfig>>,
}

impl ConfigManager {
    /// 创建新的配置管理器并加载配置
    pub fn new() -> Self {
        let config = match AppConfig::load() {
            Ok(cfg) => {
                log::info!("配置加载成功");
                cfg
            },
            Err(e) => {
                log::warn!("加载配置失败: {:?}，使用默认配置", e);
                AppConfig::default()
            },
        };

        Self { config: Arc::new(Mutex::new(config)) }
    }

    /// 获取配置
    pub fn get_config(&self) -> AppConfig {
        self.config.lock().unwrap().clone()
    }

    /// 更新配置
    pub fn update_config<F>(&self, f: F) -> anyhow::Result<()>
    where
        F: FnOnce(&mut AppConfig),
    {
        let mut config = self.config.lock().unwrap();
        f(&mut config);
        config.save()?;
        log::info!("配置已保存");
        Ok(())
    }

    /// 获取窗口宽度
    pub fn window_width(&self) -> f32 {
        self.config.lock().unwrap().window.width
    }

    /// 获取窗口高度
    pub fn window_height(&self) -> f32 {
        self.config.lock().unwrap().window.height
    }

    /// 获取当前主题
    pub fn current_theme(&self) -> String {
        self.config.lock().unwrap().theme.current_theme.clone()
    }

    /// 设置当前主题
    pub fn set_theme(&self, theme: &str) -> anyhow::Result<()> {
        self.update_config(|config| {
            config.theme.current_theme = theme.to_string();
        })
    }

    /// 获取最大结果数
    pub fn max_results(&self) -> usize {
        self.config.lock().unwrap().search.max_results
    }

    /// 检查插件是否启用
    pub fn is_plugin_enabled(&self, plugin_id: &str) -> bool {
        self.config.lock().unwrap().plugins.enabled.contains(&plugin_id.to_string())
    }

    /// 启用插件
    pub fn enable_plugin(&self, plugin_id: &str) -> anyhow::Result<()> {
        self.update_config(|config| {
            if !config.plugins.enabled.contains(&plugin_id.to_string()) {
                config.plugins.enabled.push(plugin_id.to_string());
            }
        })
    }

    /// 禁用插件
    pub fn disable_plugin(&self, plugin_id: &str) -> anyhow::Result<()> {
        self.update_config(|config| {
            config.plugins.enabled.retain(|id| id != plugin_id);
        })
    }

    /// 获取文件搜索路径
    pub fn file_search_paths(&self) -> Vec<String> {
        self.config.lock().unwrap().search.file_search_paths.clone()
    }

    /// 添加文件搜索路径
    pub fn add_file_search_path(&self, path: &str) -> anyhow::Result<()> {
        self.update_config(|config| {
            if !config.search.file_search_paths.contains(&path.to_string()) {
                config.search.file_search_paths.push(path.to_string());
            }
        })
    }

    /// 移除文件搜索路径
    pub fn remove_file_search_path(&self, path: &str) -> anyhow::Result<()> {
        self.update_config(|config| {
            config.search.file_search_paths.retain(|p| p != path);
        })
    }

    /// 保存当前配置
    pub fn save(&self) -> anyhow::Result<()> {
        let config = self.config.lock().unwrap();
        config.save()
    }
}

impl Default for ConfigManager {
    fn default() -> Self {
        Self::new()
    }
}

/// 全局配置实例
use once_cell::sync::Lazy;

static GLOBAL_CONFIG: Lazy<ConfigManager> = Lazy::new(ConfigManager::new);

/// 获取全局配置管理器
pub fn global_config() -> &'static ConfigManager {
    &GLOBAL_CONFIG
}
