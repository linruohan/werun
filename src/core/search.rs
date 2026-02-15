/// 搜索引擎模块
///
/// 提供高性能的模糊搜索功能
use std::sync::Arc;

/// 搜索结果项
#[derive(Clone, Debug)]
pub struct SearchResult {
    /// 唯一标识
    pub id: String,
    /// 显示标题
    pub title: String,
    /// 描述信息
    pub description: String,
    /// 图标路径或名称
    pub icon: Option<String>,
    /// 结果类型
    pub result_type: ResultType,
    /// 匹配分数 (越高越匹配)
    pub score: u32,
    /// 动作数据
    pub action: ActionData,
}

/// 结果类型
#[derive(Clone, Debug, PartialEq)]
pub enum ResultType {
    /// 应用程序
    Application,
    /// 文件
    File,
    /// 文件夹
    Folder,
    /// 命令
    Command,
    /// 计算器结果
    Calculator,
    /// 剪贴板历史
    Clipboard,
    /// 系统设置
    Settings,
    /// 插件自定义类型
    Custom(String),
}

/// 动作数据
#[derive(Clone, Debug)]
pub enum ActionData {
    /// 启动应用
    LaunchApp { path: String, args: Vec<String> },
    /// 打开文件
    OpenFile { path: String },
    /// 执行命令
    ExecuteCommand { command: String },
    /// 复制到剪贴板
    CopyToClipboard { text: String },
    /// 打开 URL
    OpenUrl { url: String },
    /// 自定义动作
    Custom { plugin: String, data: String },
}

/// 搜索引擎
pub struct SearchEngine {
    /// 查询字符串
    query: String,
    /// 结果限制
    limit: usize,
}

impl SearchEngine {
    /// 创建新的搜索引擎
    pub fn new() -> Self {
        Self {
            query: String::new(),
            limit: 50,
        }
    }

    /// 设置搜索查询
    pub fn set_query(&mut self, query: impl Into<String>) {
        self.query = query.into();
    }

    /// 设置结果限制
    pub fn set_limit(&mut self, limit: usize) {
        self.limit = limit;
    }

    /// 执行搜索
    ///
    /// 返回按匹配分数排序的结果列表
    pub fn search(&self, plugins: &[Arc<dyn super::plugin::Plugin>]) -> Vec<SearchResult> {
        let mut results = Vec::new();

        // 如果查询为空，返回空结果
        if self.query.is_empty() {
            return results;
        }

        // 并行搜索所有插件
        for plugin in plugins {
            if let Ok(plugin_results) = plugin.search(&self.query, self.limit) {
                results.extend(plugin_results);
            }
        }

        // 按分数排序
        results.sort_by(|a, b| b.score.cmp(&a.score));

        // 限制结果数量
        results.truncate(self.limit);

        results
    }
}

impl Default for SearchEngine {
    fn default() -> Self {
        Self::new()
    }
}
