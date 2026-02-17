use std::sync::{Arc, Mutex};

use anyhow::Result;

use crate::core::plugin::Plugin;
use crate::core::search::{ActionData, ResultType, SearchResult};

#[derive(Clone, Debug)]
pub struct CustomCommand {
    pub alias: String,
    pub command: String,
    pub description: String,
    pub working_dir: Option<String>,
    pub run_as_admin: bool,
}

pub struct CustomCommandsPlugin {
    enabled: bool,
    commands: Arc<Mutex<Vec<CustomCommand>>>,
    default_commands: Vec<CustomCommand>,
}

impl CustomCommandsPlugin {
    pub fn new() -> Self {
        let default_commands = vec![
            CustomCommand {
                alias: "git".to_string(),
                command: "git".to_string(),
                description: "Git 版本控制".to_string(),
                working_dir: None,
                run_as_admin: false,
            },
            CustomCommand {
                alias: "npm".to_string(),
                command: "npm".to_string(),
                description: "Node.js 包管理器".to_string(),
                working_dir: None,
                run_as_admin: false,
            },
            CustomCommand {
                alias: "yarn".to_string(),
                command: "yarn".to_string(),
                description: "Yarn 包管理器".to_string(),
                working_dir: None,
                run_as_admin: false,
            },
            CustomCommand {
                alias: "pnpm".to_string(),
                command: "pnpm".to_string(),
                description: "pnpm 包管理器".to_string(),
                working_dir: None,
                run_as_admin: false,
            },
            CustomCommand {
                alias: "cargo".to_string(),
                command: "cargo".to_string(),
                description: "Rust 包管理器".to_string(),
                working_dir: None,
                run_as_admin: false,
            },
            CustomCommand {
                alias: "python".to_string(),
                command: "python".to_string(),
                description: "Python 解释器".to_string(),
                working_dir: None,
                run_as_admin: false,
            },
            CustomCommand {
                alias: "pip".to_string(),
                command: "pip".to_string(),
                description: "Python 包管理器".to_string(),
                working_dir: None,
                run_as_admin: false,
            },
            CustomCommand {
                alias: "node".to_string(),
                command: "node".to_string(),
                description: "Node.js 运行时".to_string(),
                working_dir: None,
                run_as_admin: false,
            },
            CustomCommand {
                alias: "code".to_string(),
                command: "code".to_string(),
                description: "VS Code 编辑器".to_string(),
                working_dir: None,
                run_as_admin: false,
            },
            CustomCommand {
                alias: "rustc".to_string(),
                command: "rustc".to_string(),
                description: "Rust 编译器".to_string(),
                working_dir: None,
                run_as_admin: false,
            },
            CustomCommand {
                alias: "go".to_string(),
                command: "go".to_string(),
                description: "Go 编程语言".to_string(),
                working_dir: None,
                run_as_admin: false,
            },
            CustomCommand {
                alias: "docker".to_string(),
                command: "docker".to_string(),
                description: "Docker 容器平台".to_string(),
                working_dir: None,
                run_as_admin: false,
            },
            CustomCommand {
                alias: "kubectl".to_string(),
                command: "kubectl".to_string(),
                description: "Kubernetes CLI".to_string(),
                working_dir: None,
                run_as_admin: false,
            },
            CustomCommand {
                alias: "terraform".to_string(),
                command: "terraform".to_string(),
                description: "Terraform IaC".to_string(),
                working_dir: None,
                run_as_admin: false,
            },
            CustomCommand {
                alias: "curl".to_string(),
                command: "curl".to_string(),
                description: "HTTP 客户端".to_string(),
                working_dir: None,
                run_as_admin: false,
            },
            CustomCommand {
                alias: "wget".to_string(),
                command: "wget".to_string(),
                description: "文件下载工具".to_string(),
                working_dir: None,
                run_as_admin: false,
            },
            CustomCommand {
                alias: "ssh".to_string(),
                command: "ssh".to_string(),
                description: "SSH 远程连接".to_string(),
                working_dir: None,
                run_as_admin: false,
            },
            CustomCommand {
                alias: "scp".to_string(),
                command: "scp".to_string(),
                description: "安全文件复制".to_string(),
                working_dir: None,
                run_as_admin: false,
            },
            CustomCommand {
                alias: "rsync".to_string(),
                command: "rsync".to_string(),
                description: "文件同步工具".to_string(),
                working_dir: None,
                run_as_admin: false,
            },
            CustomCommand {
                alias: "tar".to_string(),
                command: "tar".to_string(),
                description: "归档工具".to_string(),
                working_dir: None,
                run_as_admin: false,
            },
            CustomCommand {
                alias: "zip".to_string(),
                command: "zip".to_string(),
                description: "ZIP 压缩工具".to_string(),
                working_dir: None,
                run_as_admin: false,
            },
            CustomCommand {
                alias: "unzip".to_string(),
                command: "unzip".to_string(),
                description: "ZIP 解压工具".to_string(),
                working_dir: None,
                run_as_admin: false,
            },
            CustomCommand {
                alias: "7z".to_string(),
                command: "7z".to_string(),
                description: "7-Zip 压缩工具".to_string(),
                working_dir: None,
                run_as_admin: false,
            },
        ];

        Self {
            enabled: true,
            commands: Arc::new(Mutex::new(default_commands.clone())),
            default_commands,
        }
    }

    pub fn add_command(&self, command: CustomCommand) {
        if let Ok(mut guard) = self.commands.lock() {
            guard.push(command);
        }
    }

    pub fn remove_command(&self, alias: &str) {
        if let Ok(mut guard) = self.commands.lock() {
            guard.retain(|c| c.alias != alias);
        }
    }

    pub fn get_commands(&self) -> Vec<CustomCommand> {
        self.commands.lock().map(|guard| guard.clone()).unwrap_or_default()
    }

    fn execute_command(&self, command: &CustomCommand, args: &[String]) -> Result<()> {
        let full_command = if args.is_empty() {
            command.command.clone()
        } else {
            format!("{} {}", command.command, args.join(" "))
        };

        let mut cmd = std::process::Command::new("cmd");
        cmd.args(["/c", &full_command]);

        if let Some(dir) = &command.working_dir {
            cmd.current_dir(dir);
        }

        if command.run_as_admin {
            #[cfg(target_os = "windows")]
            {
                use std::os::windows::process::CommandExt;
                const CREATE_NO_WINDOW: u32 = 0x08000000;
                cmd.creation_flags(CREATE_NO_WINDOW);
            }
        }

        cmd.spawn()?;
        Ok(())
    }

    fn parse_custom_command(&self, input: &str) -> Option<(String, Vec<String>)> {
        if !input.starts_with('>') && !input.starts_with(':') {
            return None;
        }

        let input = &input[1..];
        let parts: Vec<&str> = input.splitn(2, ' ').collect();

        let alias = parts[0].to_string();
        let args: Vec<String> = if parts.len() > 1 {
            parts[1].split(' ').map(|s| s.to_string()).collect()
        } else {
            Vec::new()
        };

        Some((alias, args))
    }
}

impl Plugin for CustomCommandsPlugin {
    fn id(&self) -> &str {
        "custom_commands"
    }

    fn name(&self) -> &str {
        "自定义命令"
    }

    fn description(&self) -> &str {
        "执行自定义命令和脚本"
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
        log::info!("初始化自定义命令插件...");
        log::info!("已加载 {} 个自定义命令", self.default_commands.len());
        Ok(())
    }

    fn search(&self, query: &str, limit: usize) -> Result<Vec<SearchResult>> {
        let mut results = Vec::new();
        let commands = self.get_commands();
        let query_lower = query.to_lowercase();

        for cmd in &commands {
            if cmd.alias.to_lowercase().contains(&query_lower)
                || cmd.description.to_lowercase().contains(&query_lower)
            {
                results.push(SearchResult::new(
                    format!("custom_commands:{}", cmd.alias),
                    format!("> {}", cmd.alias),
                    cmd.description.clone(),
                    ResultType::Command,
                    85,
                    ActionData::ExecuteCommand { command: cmd.command.clone() },
                ));

                if results.len() >= limit {
                    break;
                }
            }
        }

        if query.starts_with('>') || query.starts_with(':') {
            if let Some((alias, args)) = self.parse_custom_command(query) {
                for cmd in &commands {
                    if cmd.alias.to_lowercase() == alias.to_lowercase() {
                        let full_command = if args.is_empty() {
                            cmd.command.clone()
                        } else {
                            format!("{} {}", cmd.command, args.join(" "))
                        };

                        results.push(SearchResult::new(
                            format!("custom_commands:run:{}", alias),
                            format!("执行: {} {}", cmd.alias, args.join(" ")),
                            cmd.description.clone(),
                            ResultType::Command,
                            100,
                            ActionData::ExecuteCommand { command: full_command },
                        ));
                        break;
                    }
                }
            }
        }

        Ok(results)
    }

    fn execute(&self, result: &SearchResult) -> Result<()> {
        if let ActionData::ExecuteCommand { command } = &result.action {
            let commands = self.get_commands();
            for cmd in commands {
                if cmd.command == *command || cmd.alias == *command {
                    self.execute_command(&cmd, &[])?;
                    return Ok(());
                }
            }
            std::process::Command::new("cmd").args(["/c", command]).spawn()?;
        }
        Ok(())
    }

    fn refresh(&mut self) -> Result<()> {
        if let Ok(mut guard) = self.commands.lock() {
            *guard = self.default_commands.clone();
        }
        Ok(())
    }
}

impl Default for CustomCommandsPlugin {
    fn default() -> Self {
        Self::new()
    }
}
