/// 图标工具模块
///
/// 提供图标加载和管理功能
use std::collections::HashMap;
use std::sync::{Arc, Mutex};

/// 图标缓存
pub struct IconCache {
    /// 缓存的图标
    cache: Arc<Mutex<HashMap<String, Vec<u8>>>>,
}

impl IconCache {
    /// 创建新的图标缓存
    pub fn new() -> Self {
        Self {
            cache: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    /// 获取图标
    pub fn get_icon(&self, path: &str) -> Option<Vec<u8>> {
        self.cache.lock().unwrap().get(path).cloned()
    }

    /// 设置图标
    pub fn set_icon(&self, path: &str, data: Vec<u8>) {
        self.cache.lock().unwrap().insert(path.to_string(), data);
    }

    /// 从可执行文件提取图标
    pub fn extract_icon_from_exe(&self, exe_path: &str) -> Option<Vec<u8>> {
        // 检查缓存
        if let Some(cached) = self.get_icon(exe_path) {
            return Some(cached);
        }

        // TODO: 实现 Windows 图标提取
        // 使用 windows crate 提取图标资源

        None
    }

    /// 获取文件类型的默认图标
    pub fn get_file_type_icon(&self, extension: &str) -> Option<Vec<u8>> {
        let cache_key = format!("ext:{}", extension.to_lowercase());

        // 检查缓存
        if let Some(cached) = self.get_icon(&cache_key) {
            return Some(cached);
        }

        // TODO: 获取系统文件类型图标

        None
    }

    /// 清空缓存
    pub fn clear(&self) {
        self.cache.lock().unwrap().clear();
    }
}

impl Default for IconCache {
    fn default() -> Self {
        Self::new()
    }
}

/// 图标类型
#[derive(Clone, Debug, PartialEq)]
pub enum IconType {
    /// 应用图标
    Application,
    /// 文件图标
    File,
    /// 文件夹图标
    Folder,
    /// 系统图标
    System,
    /// 自定义图标
    Custom(String),
}

/// 图标信息
#[derive(Clone, Debug)]
pub struct IconInfo {
    /// 图标类型
    pub icon_type: IconType,
    /// 图标路径或标识
    pub path: String,
    /// 图标数据 (PNG 格式)
    pub data: Option<Vec<u8>>,
}

impl IconInfo {
    /// 创建应用图标
    pub fn application(app_path: &str) -> Self {
        Self {
            icon_type: IconType::Application,
            path: app_path.to_string(),
            data: None,
        }
    }

    /// 创建文件图标
    pub fn file(file_path: &str) -> Self {
        Self {
            icon_type: IconType::File,
            path: file_path.to_string(),
            data: None,
        }
    }

    /// 创建文件夹图标
    pub fn folder(folder_path: &str) -> Self {
        Self {
            icon_type: IconType::Folder,
            path: folder_path.to_string(),
            data: None,
        }
    }

    /// 加载图标数据
    pub fn load(&mut self) -> anyhow::Result<()> {
        match self.icon_type {
            IconType::Application => {
                // TODO: 从可执行文件加载图标
            }
            IconType::File => {
                // TODO: 根据文件扩展名加载图标
                let extension = std::path::Path::new(&self.path)
                    .extension()
                    .and_then(|e| e.to_str())
                    .unwrap_or("");

                if !extension.is_empty() {
                    // 尝试加载文件类型图标
                }
            }
            IconType::Folder => {
                // 使用默认文件夹图标
            }
            IconType::System => {
                // 使用系统图标
            }
            IconType::Custom(_) => {
                // 加载自定义图标
            }
        }

        Ok(())
    }
}
