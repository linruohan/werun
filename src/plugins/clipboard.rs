use std::sync::{Arc, Mutex};

use anyhow::Result;

/// 剪贴板历史插件
///
/// 管理剪贴板历史记录
use crate::core::plugin::Plugin;
use crate::{
    core::search::{ActionData, ResultType, SearchResult},
    utils::clipboard::ClipboardManager,
};

/// 剪贴板条目
#[derive(Clone, Debug)]
pub struct ClipboardEntry {
    /// 唯一标识
    pub id: String,
    /// 内容文本
    pub text: String,
    /// 时间戳
    pub timestamp: chrono::DateTime<chrono::Local>,
    /// 内容预览（截断）
    pub preview: String,
}

/// 剪贴板历史插件
pub struct ClipboardPlugin {
    /// 是否启用
    enabled: bool,
    /// 历史记录
    history: Arc<Mutex<Vec<ClipboardEntry>>>,
    /// 最大历史数量
    max_history: usize,
    /// 剪贴板管理器
    clipboard_manager: ClipboardManager,
}

impl ClipboardPlugin {
    /// 创建新的剪贴板插件
    pub fn new() -> Self {
        Self {
            enabled: true,
            history: Arc::new(Mutex::new(Vec::new())),
            max_history: 100,
            clipboard_manager: ClipboardManager::new(),
        }
    }

    /// 添加条目到历史
    pub fn add_entry(&self, text: String) {
        if text.is_empty() {
            return;
        }

        let preview = if text.len() > 100 { format!("{}...", &text[..100]) } else { text.clone() };

        let entry = ClipboardEntry {
            id: format!("clip:{}", chrono::Local::now().timestamp_millis()),
            text: text.clone(),
            timestamp: chrono::Local::now(),
            preview,
        };

        if let Ok(mut guard) = self.history.lock() {
            // 去重：如果最后一条相同则不添加
            if let Some(last) = guard.first() {
                if last.text == text {
                    return;
                }
            }

            guard.insert(0, entry);

            // 限制历史数量
            if guard.len() > self.max_history {
                guard.truncate(self.max_history);
            }
        }
    }

    /// 获取历史记录
    fn get_history(&self) -> Vec<ClipboardEntry> {
        self.history.lock().map(|guard| guard.clone()).unwrap_or_default()
    }

    /// 格式化时间
    fn format_time(&self, time: &chrono::DateTime<chrono::Local>) -> String {
        let now = chrono::Local::now();
        let diff = now.signed_duration_since(*time);

        if diff.num_seconds() < 60 {
            "刚刚".to_string()
        } else if diff.num_minutes() < 60 {
            format!("{} 分钟前", diff.num_minutes())
        } else if diff.num_hours() < 24 {
            format!("{} 小时前", diff.num_hours())
        } else {
            format!("{} 天前", diff.num_days())
        }
    }

    /// 复制文本到剪贴板
    fn copy_to_clipboard(&self, text: &str) -> Result<()> {
        self.clipboard_manager.set_text(text)
    }
}

impl Plugin for ClipboardPlugin {
    fn id(&self) -> &str {
        "clipboard"
    }

    fn name(&self) -> &str {
        "剪贴板历史"
    }

    fn description(&self) -> &str {
        "管理和快速粘贴剪贴板历史"
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
        log::info!("初始化剪贴板历史插件...");

        // 尝试读取当前剪贴板内容
        if let Ok(text) = self.clipboard_manager.get_text() {
            if !text.is_empty() {
                self.add_entry(text);
            }
        }

        Ok(())
    }

    fn search(&self, query: &str, limit: usize) -> Result<Vec<SearchResult>> {
        let history = self.get_history();
        let mut results = Vec::new();

        // 如果查询为空，显示最近的历史
        if query.is_empty() {
            for entry in history.iter().take(limit) {
                results.push(SearchResult::new(
                    entry.id.clone(),
                    entry.preview.clone(),
                    format!("{} · 按 Enter 粘贴", self.format_time(&entry.timestamp)),
                    ResultType::Clipboard,
                    0, // 按时间排序
                    ActionData::CopyToClipboard { text: entry.text.clone() },
                ));
            }
        } else {
            // 搜索历史
            for entry in history {
                if entry.text.to_lowercase().contains(&query.to_lowercase()) {
                    results.push(SearchResult::new(
                        entry.id.clone(),
                        entry.preview.clone(),
                        format!("{} · 按 Enter 粘贴", self.format_time(&entry.timestamp)),
                        ResultType::Clipboard,
                        50, // 中等优先级
                        ActionData::CopyToClipboard { text: entry.text.clone() },
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
        if let ActionData::CopyToClipboard { text } = &result.action {
            self.copy_to_clipboard(text)?;
            log::info!("已复制到剪贴板: {}", text);
        }
        Ok(())
    }

    fn refresh(&mut self) -> Result<()> {
        // 清空历史
        if let Ok(mut guard) = self.history.lock() {
            guard.clear();
        }
        Ok(())
    }
}

impl Default for ClipboardPlugin {
    fn default() -> Self {
        Self::new()
    }
}
