/// 插件系统接口
///
/// 定义所有插件必须实现的 trait
use super::search::SearchResult;
use anyhow::Result;
use std::sync::Arc;

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
    ///
    /// # Arguments
    /// * `query` - 搜索查询字符串
    /// * `limit` - 最大返回结果数
    ///
    /// # Returns
    /// 搜索结果列表
    fn search(&self, query: &str, limit: usize) -> Result<Vec<SearchResult>>;

    /// 执行动作
    ///
    /// # Arguments
    /// * `result` - 要执行的结果项
    ///
    /// # Returns
    /// 执行是否成功
    fn execute(&self, result: &SearchResult) -> Result<()>;

    /// 刷新插件数据（如重新索引）
    fn refresh(&mut self) -> Result<()>;
}

/// 插件管理器
pub struct PluginManager {
    /// 已注册的插件列表
    plugins: Vec<Arc<dyn Plugin>>,
}

impl PluginManager {
    /// 创建新的插件管理器
    pub fn new() -> Self {
        Self {
            plugins: Vec::new(),
        }
    }

    /// 注册插件
    pub fn register(&mut self, plugin: Arc<dyn Plugin>) {
        log::info!("注册插件: {} ({})", plugin.name(), plugin.id());
        self.plugins.push(plugin);
    }

    /// 获取所有启用的插件
    pub fn enabled_plugins(&self) -> Vec<Arc<dyn Plugin>> {
        self.plugins
            .iter()
            .filter(|p| p.is_enabled())
            .cloned()
            .collect()
    }

    /// 获取所有插件
    pub fn all_plugins(&self) -> &[Arc<dyn Plugin>] {
        &self.plugins
    }

    /// 根据 ID 查找插件
    pub fn get_plugin(&self, id: &str) -> Option<Arc<dyn Plugin>> {
        self.plugins.iter().find(|p| p.id() == id).cloned()
    }

    /// 初始化所有插件
    pub fn initialize_all(&mut self) -> Result<()> {
        for plugin in &self.plugins {
            // 注意：这里需要可变引用，可能需要使用 Mutex 或 RwLock
            log::info!("初始化插件: {}", plugin.name());
        }
        Ok(())
    }
}

impl Default for PluginManager {
    fn default() -> Self {
        Self::new()
    }
}
