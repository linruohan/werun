use std::{fs, path::PathBuf, sync::Arc};

use anyhow::Result;
use chrono::{DateTime, Local};
use parking_lot::RwLock;
use serde::{Deserialize, Serialize};

use crate::core::{
    plugin::Plugin,
    search::{ActionData, ResultType, SearchResult},
};

/// 任务状态
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum TaskStatus {
    /// 待处理
    Pending,
    /// 进行中
    InProgress,
    /// 已完成
    Completed,
    /// 已取消
    Cancelled,
}

impl TaskStatus {
    /// 获取状态的中文显示
    pub fn display(&self) -> &str {
        match self {
            TaskStatus::Pending => "待处理",
            TaskStatus::InProgress => "进行中",
            TaskStatus::Completed => "已完成",
            TaskStatus::Cancelled => "已取消",
        }
    }
}

/// 任务优先级
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum TaskPriority {
    /// 低
    Low,
    /// 中
    Medium,
    /// 高
    High,
    /// 紧急
    Urgent,
}

impl TaskPriority {
    /// 获取优先级的中文显示
    pub fn display(&self) -> &str {
        match self {
            TaskPriority::Low => "低",
            TaskPriority::Medium => "中",
            TaskPriority::High => "高",
            TaskPriority::Urgent => "紧急",
        }
    }

    /// 获取排序权重
    pub fn weight(&self) -> u32 {
        match self {
            TaskPriority::Low => 100,
            TaskPriority::Medium => 200,
            TaskPriority::High => 300,
            TaskPriority::Urgent => 400,
        }
    }
}

/// 任务结构
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Task {
    /// 任务 ID
    pub id: String,
    /// 任务标题
    pub title: String,
    /// 任务描述
    pub description: String,
    /// 任务状态
    pub status: TaskStatus,
    /// 任务优先级
    pub priority: TaskPriority,
    /// 创建时间
    pub created_at: DateTime<Local>,
    /// 更新时间
    pub updated_at: DateTime<Local>,
    /// 标签
    pub tags: Vec<String>,
}

impl Task {
    /// 创建新任务
    pub fn new(title: String, description: String, priority: TaskPriority) -> Self {
        let now = Local::now();
        Self {
            id: uuid_simple(),
            title,
            description,
            status: TaskStatus::Pending,
            priority,
            created_at: now,
            updated_at: now,
            tags: Vec::new(),
        }
    }

    /// 更新任务状态
    pub fn update_status(&mut self, status: TaskStatus) {
        self.status = status;
        self.updated_at = Local::now();
    }

    /// 添加标签
    pub fn add_tag(&mut self, tag: String) {
        if !self.tags.contains(&tag) {
            self.tags.push(tag);
            self.updated_at = Local::now();
        }
    }

    /// 获取格式化显示
    pub fn display(&self) -> String {
        format!(
            "[{}] {} - {} ({})",
            self.priority.display(),
            self.title,
            self.status.display(),
            self.created_at.format("%Y-%m-%d %H:%M")
        )
    }
}

/// 生成简单 UUID
fn uuid_simple() -> String {
    use std::time::{SystemTime, UNIX_EPOCH};
    let timestamp = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_millis();
    format!("task_{}", timestamp)
}

/// 任务管理器插件
///
/// 提供简单的任务跟踪和管理功能
pub struct TaskManagerPlugin {
    /// 是否启用
    enabled: bool,
    /// 任务列表
    tasks: Arc<RwLock<Vec<Task>>>,
    /// 数据文件路径
    data_file: PathBuf,
}

impl TaskManagerPlugin {
    /// 创建新的任务管理器插件
    pub fn new() -> Self {
        let data_file = dirs::data_local_dir()
            .unwrap_or_else(|| PathBuf::from("."))
            .join("werun")
            .join("tasks.json");

        Self { enabled: true, tasks: Arc::new(RwLock::new(Vec::new())), data_file }
    }

    /// 加载任务
    fn load_tasks(&self) -> Result<()> {
        if self.data_file.exists() {
            let content = fs::read_to_string(&self.data_file)?;
            let tasks: Vec<Task> = serde_json::from_str(&content)?;
            *self.tasks.write() = tasks;
        }
        Ok(())
    }

    /// 保存任务
    fn save_tasks(&self) -> Result<()> {
        if let Some(parent) = self.data_file.parent() {
            fs::create_dir_all(parent)?;
        }

        let tasks = self.tasks.read();
        let content = serde_json::to_string_pretty(&*tasks)?;
        fs::write(&self.data_file, content)?;
        Ok(())
    }

    /// 创建任务
    fn create_task(&self, title: String, description: String, priority: TaskPriority) -> Task {
        let task = Task::new(title, description, priority);
        self.tasks.write().push(task.clone());
        let _ = self.save_tasks();
        task
    }

    /// 更新任务状态
    fn update_task_status(&self, task_id: &str, status: TaskStatus) -> Option<Task> {
        let mut tasks = self.tasks.write();
        if let Some(task) = tasks.iter_mut().find(|t| t.id == task_id) {
            task.update_status(status.clone());
            let updated_task = task.clone();
            let _ = self.save_tasks();
            Some(updated_task)
        } else {
            None
        }
    }

    /// 获取所有任务
    fn get_all_tasks(&self) -> Vec<Task> {
        self.tasks.read().clone()
    }

    /// 按状态过滤任务
    fn filter_by_status(&self, status: TaskStatus) -> Vec<Task> {
        self.tasks.read().iter().filter(|t| t.status == status).cloned().collect()
    }

    /// 搜索任务
    fn search_tasks(&self, query: &str) -> Vec<Task> {
        let query_lower = query.to_lowercase();
        self.tasks
            .read()
            .iter()
            .filter(|t| {
                t.title.to_lowercase().contains(&query_lower)
                    || t.description.to_lowercase().contains(&query_lower)
                    || t.tags.iter().any(|tag| tag.to_lowercase().contains(&query_lower))
            })
            .cloned()
            .collect()
    }

    /// 检查是否是任务相关查询
    fn is_task_query(&self, query: &str) -> bool {
        let trimmed = query.trim().to_lowercase();
        trimmed.starts_with("task")
            || trimmed.starts_with("todo")
            || trimmed.starts_with("任务")
            || trimmed.starts_with("add task")
            || trimmed.starts_with("新建任务")
    }

    /// 解析任务创建命令
    fn parse_task_create(&self, query: &str) -> Option<(String, String, TaskPriority)> {
        let trimmed = query.trim();

        // 移除前缀
        let content = trimmed
            .strip_prefix("task add")
            .or_else(|| trimmed.strip_prefix("add task"))
            .or_else(|| trimmed.strip_prefix("新建任务"))
            .or_else(|| trimmed.strip_prefix("任务添加"))
            .unwrap_or(trimmed)
            .trim();

        if content.is_empty() {
            return None;
        }

        // 简单解析：标题 | 描述 | 优先级
        let parts: Vec<&str> = content.split('|').collect();

        let title = parts.first()?.trim().to_string();
        let description = parts.get(1).map(|s| s.trim()).unwrap_or("").to_string();

        let priority = parts
            .get(2)
            .map(|s| s.trim().to_lowercase())
            .map(|s| match s.as_str() {
                "urgent" | "紧急" => TaskPriority::Urgent,
                "high" | "高" => TaskPriority::High,
                "low" | "低" => TaskPriority::Low,
                _ => TaskPriority::Medium,
            })
            .unwrap_or(TaskPriority::Medium);

        Some((title, description, priority))
    }
}

impl Plugin for TaskManagerPlugin {
    fn id(&self) -> &str {
        "task_manager"
    }

    fn name(&self) -> &str {
        "任务管理器"
    }

    fn description(&self) -> &str {
        "简单任务跟踪和管理"
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
        log::info!("初始化任务管理器插件...");
        self.load_tasks()?;
        log::info!("已加载 {} 个任务", self.tasks.read().len());
        Ok(())
    }

    fn search(&self, query: &str, limit: usize) -> Result<Vec<SearchResult>> {
        let mut results = Vec::new();

        // 检查是否是任务相关查询
        if self.is_task_query(query) {
            // 尝试解析任务创建
            if let Some((title, description, priority)) = self.parse_task_create(query) {
                results.push(SearchResult::new(
                    format!("task_create:{}", title),
                    format!("创建任务：{}", title),
                    format!("优先级：{} | 按 Enter 创建", priority.display()),
                    ResultType::Task,
                    950 + priority.weight(),
                    ActionData::CreateTask { title, description, priority },
                ));
            } else {
                // 显示任务列表或搜索任务
                let tasks = if query.trim().len() > 5 {
                    self.search_tasks(query)
                } else {
                    self.get_all_tasks()
                };

                for task in tasks.iter().take(limit) {
                    results.push(SearchResult::new(
                        format!("task:{}", task.id),
                        task.display(),
                        "按 Enter 查看详情".to_string(),
                        ResultType::Task,
                        800 + task.priority.weight(),
                        ActionData::ViewTask { task: task.clone() },
                    ));
                }

                // 添加创建任务的快捷选项
                if !results.is_empty() && query.trim().len() > 5 {
                    results.push(SearchResult::new(
                        "task_create_new".to_string(),
                        format!("创建新任务：{}", query),
                        "按 Enter 创建".to_string(),
                        ResultType::Task,
                        500,
                        ActionData::CreateTask {
                            title: query.to_string(),
                            description: String::new(),
                            priority: TaskPriority::Medium,
                        },
                    ));
                }
            }
        }

        Ok(results)
    }

    fn execute(&self, result: &SearchResult) -> Result<()> {
        match &result.action {
            ActionData::CreateTask { title, description, priority } => {
                let task = self.create_task(title.clone(), description.clone(), priority.clone());
                log::info!("创建任务：{}", task.display());
                println!("任务已创建：{}", task.display());
            },
            ActionData::ViewTask { task } => {
                log::info!("查看任务：{}", task.display());
                println!("任务详情:\n{}", task.display());
                println!("描述：{}", task.description);
                if !task.tags.is_empty() {
                    println!("标签：{}", task.tags.join(", "));
                }
            },
            _ => {},
        }
        Ok(())
    }

    fn refresh(&mut self) -> Result<()> {
        self.load_tasks()
    }
}

impl Default for TaskManagerPlugin {
    fn default() -> Self {
        Self::new()
    }
}
