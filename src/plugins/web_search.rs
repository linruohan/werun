use anyhow::Result;

/// 网页搜索插件
///
/// 支持多种搜索引擎快速搜索
use crate::core::plugin::Plugin;
use crate::core::search::{ActionData, ResultType, SearchResult};

/// 搜索引擎配置
#[derive(Clone, Debug)]
pub struct SearchEngine {
    /// 引擎名称
    pub name: String,
    /// 引擎ID
    pub id: String,
    /// 搜索URL模板（使用 {query} 作为占位符）
    pub url_template: String,
    /// 图标
    pub icon: Option<String>,
}

/// 网页搜索插件
pub struct WebSearchPlugin {
    /// 是否启用
    enabled: bool,
    /// 默认搜索引擎
    default_engine: String,
    /// 搜索引擎列表
    engines: Vec<SearchEngine>,
}

impl WebSearchPlugin {
    /// 创建新的网页搜索插件
    pub fn new() -> Self {
        let engines = vec![
            SearchEngine {
                name: "Google".to_string(),
                id: "google".to_string(),
                url_template: "https://www.google.com/search?q={query}".to_string(),
                icon: None,
            },
            SearchEngine {
                name: "Bing".to_string(),
                id: "bing".to_string(),
                url_template: "https://www.bing.com/search?q={query}".to_string(),
                icon: None,
            },
            SearchEngine {
                name: "百度".to_string(),
                id: "baidu".to_string(),
                url_template: "https://www.baidu.com/s?wd={query}".to_string(),
                icon: None,
            },
            SearchEngine {
                name: "DuckDuckGo".to_string(),
                id: "duckduckgo".to_string(),
                url_template: "https://duckduckgo.com/?q={query}".to_string(),
                icon: None,
            },
            SearchEngine {
                name: "GitHub".to_string(),
                id: "github".to_string(),
                url_template: "https://github.com/search?q={query}".to_string(),
                icon: None,
            },
            SearchEngine {
                name: "Stack Overflow".to_string(),
                id: "stackoverflow".to_string(),
                url_template: "https://stackoverflow.com/search?q={query}".to_string(),
                icon: None,
            },
        ];

        Self { enabled: true, default_engine: "google".to_string(), engines }
    }

    /// 获取搜索引擎
    fn get_engine(&self, id: &str) -> Option<&SearchEngine> {
        self.engines.iter().find(|e| e.id == id)
    }

    /// 构建搜索URL
    fn build_search_url(&self, engine_id: &str, query: &str) -> Option<String> {
        self.get_engine(engine_id).map(|engine| {
            let encoded_query = urlencoding::encode(query);
            engine.url_template.replace("{query}", &encoded_query)
        })
    }

    /// 在浏览器中打开URL
    fn open_url(&self, url: &str) -> Result<()> {
        std::process::Command::new("cmd").args(["/c", "start", "", url]).spawn()?;
        Ok(())
    }
}

impl Plugin for WebSearchPlugin {
    fn id(&self) -> &str {
        "web_search"
    }

    fn name(&self) -> &str {
        "网页搜索"
    }

    fn description(&self) -> &str {
        "使用多种搜索引擎搜索网页"
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
        log::info!("初始化网页搜索插件...");
        log::info!("可用搜索引擎: {}", self.engines.len());
        Ok(())
    }

    fn search(&self, query: &str, limit: usize) -> Result<Vec<SearchResult>> {
        let mut results = Vec::new();

        // 如果查询以特定前缀开头，使用对应的搜索引擎
        let (engine_id, search_query) = if query.starts_with("g ") {
            ("google", &query[2..])
        } else if query.starts_with("b ") {
            ("bing", &query[2..])
        } else if query.starts_with("bd ") {
            ("baidu", &query[3..])
        } else if query.starts_with("ddg ") {
            ("duckduckgo", &query[4..])
        } else if query.starts_with("gh ") {
            ("github", &query[3..])
        } else if query.starts_with("so ") {
            ("stackoverflow", &query[3..])
        } else {
            // 默认使用 Google
            (self.default_engine.as_str(), query)
        };

        if !search_query.is_empty() {
            if let Some(engine) = self.get_engine(engine_id) {
                if let Some(url) = self.build_search_url(engine_id, search_query) {
                    results.push(
                        SearchResult::new(
                            format!("web_search:{}:{}", engine_id, search_query),
                            format!("在 {} 搜索 \"{}\"", engine.name, search_query),
                            format!("使用 {} 搜索 \"{}\"", engine.name, search_query),
                            ResultType::Command,
                            80, // 较高的优先级
                            ActionData::OpenUrl { url },
                        )
                        .with_icon(engine.icon.clone()),
                    );
                }
            }
        }

        // 如果查询不为空，添加所有搜索引擎的选项
        if !query.is_empty() && !query.contains(' ') {
            for engine in &self.engines {
                if engine.id != engine_id {
                    if let Some(url) = self.build_search_url(&engine.id, query) {
                        results.push(
                            SearchResult::new(
                                format!("web_search:{}:{}", engine.id, query),
                                format!("在 {} 搜索 \"{}\"", engine.name, query),
                                format!("使用 {} 搜索 \"{}\"", engine.name, query),
                                ResultType::Command,
                                70,
                                ActionData::OpenUrl { url },
                            )
                            .with_icon(engine.icon.clone()),
                        );

                        if results.len() >= limit {
                            break;
                        }
                    }
                }
            }
        }

        Ok(results)
    }

    fn execute(&self, result: &SearchResult) -> Result<()> {
        if let ActionData::OpenUrl { url } = &result.action {
            self.open_url(url)?;
        }
        Ok(())
    }

    fn refresh(&mut self) -> Result<()> {
        Ok(())
    }
}

impl Default for WebSearchPlugin {
    fn default() -> Self {
        Self::new()
    }
}
