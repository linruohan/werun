use std::process::Command;

use anyhow::Result;

use crate::core::{
    plugin::Plugin,
    search::{ActionData, ResultType, SearchResult},
};

/// 命令执行器插件
///
/// 提供 Shell 命令执行功能
pub struct CommandExecutorPlugin {
    /// 是否启用
    enabled: bool,
    /// 命令超时时间 (秒)
    timeout_secs: u64,
}

impl CommandExecutorPlugin {
    /// 创建新的命令执行器插件
    pub fn new() -> Self {
        Self { enabled: true, timeout_secs: 30 }
    }

    /// 执行 Shell 命令
    fn execute_command(&self, cmd: &str) -> Result<(String, String)> {
        #[cfg(target_os = "windows")]
        {
            let output = Command::new("cmd").args(["/C", cmd]).output()?;

            let stdout = String::from_utf8_lossy(&output.stdout).to_string();
            let stderr = String::from_utf8_lossy(&output.stderr).to_string();
            Ok((stdout, stderr))
        }

        #[cfg(not(target_os = "windows"))]
        {
            let output = Command::new("sh").args(["-c", cmd]).output()?;

            let stdout = String::from_utf8_lossy(&output.stdout).to_string();
            let stderr = String::from_utf8_lossy(&output.stderr).to_string();
            Ok((stdout, stderr))
        }
    }

    /// 检查是否是命令执行查询
    fn is_command_query(&self, query: &str) -> bool {
        // 检查是否以 > 或 ! 开头，这是常见的命令执行前缀
        let trimmed = query.trim();
        trimmed.starts_with('>') || trimmed.starts_with('!')
    }

    /// 提取实际命令
    fn extract_command(&self, query: &str) -> String {
        let trimmed = query.trim();
        if trimmed.starts_with('>') || trimmed.starts_with('!') {
            trimmed[1..].trim().to_string()
        } else {
            trimmed.to_string()
        }
    }

    /// 格式化输出
    fn format_output(&self, stdout: &str, stderr: &str, exit_code: i32) -> String {
        let mut result = String::new();

        if !stdout.is_empty() {
            result.push_str("输出:\n");
            result.push_str(stdout);
        }

        if !stderr.is_empty() {
            if !result.is_empty() {
                result.push('\n');
            }
            result.push_str("错误:\n");
            result.push_str(stderr);
        }

        result.push_str(&format!("\n退出码：{}", exit_code));

        result
    }
}

impl Plugin for CommandExecutorPlugin {
    fn id(&self) -> &str {
        "command_executor"
    }

    fn name(&self) -> &str {
        "命令执行器"
    }

    fn description(&self) -> &str {
        "执行 Shell 命令并查看输出"
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
        log::info!("初始化命令执行器插件...");
        Ok(())
    }

    fn search(&self, query: &str, _limit: usize) -> Result<Vec<SearchResult>> {
        let mut results = Vec::new();

        // 检查是否是命令执行查询
        if self.is_command_query(query) {
            let cmd = self.extract_command(query);

            if !cmd.is_empty() {
                // 创建一个临时结果用于预览
                results.push(SearchResult::new(
                    format!("cmd:{}", cmd),
                    format!("执行：{}", cmd),
                    "按 Enter 执行命令".to_string(),
                    ResultType::SystemCommand,
                    900,
                    ActionData::ExecuteCommand { command: cmd },
                ));
            }
        }

        Ok(results)
    }

    fn execute(&self, result: &SearchResult) -> Result<()> {
        if let ActionData::ExecuteCommand { command } = &result.action {
            log::info!("执行命令：{}", command);

            match self.execute_command(command) {
                Ok((stdout, stderr)) => {
                    // 将输出复制到剪贴板
                    let output = self.format_output(
                        &stdout, &stderr, 0, // 成功执行
                    );

                    log::info!("命令执行成功:\n{}", output);

                    // TODO: 显示输出或复制到剪贴板
                    println!("{}", output);
                },
                Err(e) => {
                    log::error!("命令执行失败：{}", e);
                    // TODO: 显示错误信息
                },
            }
        }
        Ok(())
    }

    fn refresh(&mut self) -> Result<()> {
        Ok(())
    }
}

impl Default for CommandExecutorPlugin {
    fn default() -> Self {
        Self::new()
    }
}
