use std::sync::{Arc, Mutex};

use anyhow::Result;

/// 插件系统接口
///
/// 定义所有插件必须实现的 trait
use super::search::SearchResult;

/// 插件 trait
///
/// 所有功能模块（应用启动、文件搜索等）都需要实现此 trait
pub trait Plugin: Send + Sync {
    /// 插件唯一标识
    fn id(&self) -> &str;

    /// 插件名称
    fn name(&self) -> &str;

    /// 插件描述
    fn description(&self) -> &str;

    /// 插件版本
    fn version(&self) -> &str;

    /// 是否启用
    fn is_enabled(&self) -> bool;

    /// 设置启用状态
    fn set_enabled(&mut self, enabled: bool);

    /// 初始化插件
    fn initialize(&mut self) -> Result<()>;

    /// 执行搜索
    fn search(&self, query: &str, limit: usize) -> Result<Vec<SearchResult>>;

    /// 执行动作
    fn execute(&self, result: &SearchResult) -> Result<()>;

    /// 刷新插件数据（如重新索引）
    fn refresh(&mut self) -> Result<()>;
}

/// 插件管理器
pub struct PluginManager {
    /// 已注册的插件列表
    plugins: Vec<Arc<Mutex<dyn Plugin>>>,
}

impl PluginManager {
    /// 创建新的插件管理器
    pub fn new() -> Self {
        Self { plugins: Vec::new() }
    }

    /// 注册插件
    pub fn register(&mut self, plugin: impl Plugin + 'static) {
        let plugin = Arc::new(Mutex::new(plugin));
        log::info!("注册插件");
        self.plugins.push(plugin);
    }

    /// 获取所有插件数量
    pub fn plugin_count(&self) -> usize {
        self.plugins.len()
    }

    /// 初始化所有插件
    pub fn initialize_all(&mut self) -> Result<()> {
        for plugin in &self.plugins {
            if let Ok(mut guard) = plugin.lock() {
                log::info!("初始化插件: {}", guard.name());
                if let Err(e) = guard.initialize() {
                    log::error!("初始化插件 {} 失败: {:?}", guard.name(), e);
                }
            }
        }
        Ok(())
    }

    /// 搜索所有插件
    pub fn search_all(&self, query: &str, limit: usize) -> Vec<SearchResult> {
        let mut results = Vec::new();

        for plugin in &self.plugins {
            if let Ok(guard) = plugin.lock() {
                if guard.is_enabled() {
                    match guard.search(query, limit) {
                        Ok(mut plugin_results) => {
                            results.append(&mut plugin_results);
                        },
                        Err(e) => {
                            log::error!("插件 {} 搜索失败: {:?}", guard.name(), e);
                        },
                    }
                }
            }
        }

        // 按分数排序
        results.sort_by(|a, b| b.score.cmp(&a.score));
        results.truncate(limit);

        results
    }

    /// 执行结果
    pub fn execute(&self, result: &SearchResult) -> Result<()> {
        // 根据 ID 前缀找到对应的插件
        for plugin in &self.plugins {
            if let Ok(guard) = plugin.lock() {
                let plugin_id = guard.id();
                // 支持两种匹配方式：
                // 1. result.id 以 "plugin_id:" 开头
                // 2. result.id 等于 plugin_id
                if result.id.starts_with(&format!("{}:", plugin_id)) || result.id == plugin_id {
                    return guard.execute(result);
                }
            }
        }

        Err(anyhow::anyhow!("未找到对应的插件"))
    }
}

impl Default for PluginManager {
    fn default() -> Self {
        Self::new()
    }
}
