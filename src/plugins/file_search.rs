/// 文件搜索插件
///
/// 提供文件搜索功能
use crate::core::plugin::Plugin;
use crate::core::search::{ActionData, ResultType, SearchResult};
use anyhow::Result;
use std::sync::{Arc, Mutex};

/// 文件信息
#[derive(Clone, Debug)]
pub struct FileInfo {
    /// 文件名称
    pub name: String,
    /// 完整路径
    pub path: String,
    /// 文件大小
    pub size: u64,
    /// 是否目录
    pub is_dir: bool,
    /// 修改时间
    pub modified: std::time::SystemTime,
}

/// 文件搜索插件
pub struct FileSearchPlugin {
    /// 是否启用
    enabled: bool,
    /// 索引的文件列表
    files: Arc<Mutex<Vec<FileInfo>>>,
    /// 搜索路径
    search_paths: Vec<String>,
}

impl FileSearchPlugin {
    /// 创建新的文件搜索插件
    pub fn new() -> Self {
        let search_paths = vec![
            dirs::desktop_dir()
                .map(|p| p.to_string_lossy().to_string())
                .unwrap_or_default(),
            dirs::document_dir()
                .map(|p| p.to_string_lossy().to_string())
                .unwrap_or_default(),
            dirs::download_dir()
                .map(|p| p.to_string_lossy().to_string())
                .unwrap_or_default(),
        ];

        Self {
            enabled: true,
            files: Arc::new(Mutex::new(Vec::new())),
            search_paths,
        }
    }

    /// 扫描文件
    fn scan_files(&self) -> Result<Vec<FileInfo>> {
        let mut files = Vec::new();

        for path_str in &self.search_paths {
            let path = std::path::Path::new(path_str);
            if path.exists() {
                self.scan_directory(path, &mut files, 2)?; // 限制递归深度
            }
        }

        Ok(files)
    }

    /// 递归扫描目录
    fn scan_directory(
        &self,
        path: &std::path::Path,
        files: &mut Vec<FileInfo>,
        depth: usize,
    ) -> Result<()> {
        if depth == 0 {
            return Ok(());
        }

        if let Ok(entries) = std::fs::read_dir(path) {
            for entry in entries.flatten() {
                let path = entry.path();
                let metadata = entry.metadata().ok();

                let name = path
                    .file_name()
                    .map(|s| s.to_string_lossy().to_string())
                    .unwrap_or_default();

                let is_dir = path.is_dir();
                let size = metadata.as_ref().map(|m| m.len()).unwrap_or(0);
                let modified = metadata
                    .as_ref()
                    .and_then(|m| m.modified().ok())
                    .unwrap_or(std::time::SystemTime::UNIX_EPOCH);

                files.push(FileInfo {
                    name,
                    path: path.to_string_lossy().to_string(),
                    size,
                    is_dir,
                    modified,
                });

                // 递归扫描子目录
                if is_dir && depth > 1 {
                    let _ = self.scan_directory(&path, files, depth - 1);
                }
            }
        }

        Ok(())
    }

    /// 格式化文件大小
    fn format_size(&self, size: u64) -> String {
        const UNITS: &[&str] = &["B", "KB", "MB", "GB", "TB"];
        let mut size = size as f64;
        let mut unit_index = 0;

        while size >= 1024.0 && unit_index < UNITS.len() - 1 {
            size /= 1024.0;
            unit_index += 1;
        }

        format!("{:.1} {}", size, UNITS[unit_index])
    }

    /// 打开文件或目录
    fn open_file(&self, path: &str) -> Result<()> {
        std::process::Command::new("explorer").arg(path).spawn()?;
        Ok(())
    }
}

impl Plugin for FileSearchPlugin {
    fn id(&self) -> &str {
        "file_search"
    }

    fn name(&self) -> &str {
        "文件搜索"
    }

    fn description(&self) -> &str {
        "搜索文件和文件夹"
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
        log::info!("初始化文件搜索插件...");

        // 扫描文件
        let files = self.scan_files()?;

        // 存储文件列表
        if let Ok(mut guard) = self.files.lock() {
            *guard = files;
            log::info!("已索引 {} 个文件", guard.len());
        }

        Ok(())
    }

    fn search(&self, query: &str, limit: usize) -> Result<Vec<SearchResult>> {
        // 文件搜索需要至少 2 个字符
        if query.len() < 2 {
            return Ok(Vec::new());
        }

        let files = self.files.lock().unwrap();
        let mut results = Vec::new();

        for file in files.iter() {
            // 简单的模糊匹配
            if file.name.to_lowercase().contains(&query.to_lowercase()) {
                let result_type = if file.is_dir {
                    ResultType::Folder
                } else {
                    ResultType::File
                };

                let description = if file.is_dir {
                    "文件夹".to_string()
                } else {
                    format!("文件 · {}", self.format_size(file.size))
                };

                results.push(SearchResult {
                    id: format!("file:{}", file.path),
                    title: file.name.clone(),
                    description,
                    icon: None,
                    result_type,
                    score: 80,
                    action: ActionData::OpenFile {
                        path: file.path.clone(),
                    },
                });

                if results.len() >= limit {
                    break;
                }
            }
        }

        Ok(results)
    }

    fn execute(&self, result: &SearchResult) -> Result<()> {
        if let ActionData::OpenFile { path } = &result.action {
            self.open_file(path)?;
        }
        Ok(())
    }

    fn refresh(&mut self) -> Result<()> {
        self.initialize()
    }
}

impl Default for FileSearchPlugin {
    fn default() -> Self {
        Self::new()
    }
}
