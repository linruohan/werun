use anyhow::Result;

use crate::core::plugin::Plugin;
use crate::core::search::{ActionData, ResultType, SearchResult};

#[derive(Clone, Debug)]
pub struct SystemCommand {
    pub id: String,
    pub name: String,
    pub description: String,
    pub command: String,
    pub icon: Option<String>,
}

pub struct SystemCommandsPlugin {
    enabled: bool,
    commands: Vec<SystemCommand>,
}

impl SystemCommandsPlugin {
    pub fn new() -> Self {
        let commands = vec![
            SystemCommand {
                id: "shutdown".to_string(),
                name: "关机".to_string(),
                description: "关闭计算机".to_string(),
                command: "shutdown /s /t 0".to_string(),
                icon: None,
            },
            SystemCommand {
                id: "restart".to_string(),
                name: "重启".to_string(),
                description: "重新启动计算机".to_string(),
                command: "shutdown /r /t 0".to_string(),
                icon: None,
            },
            SystemCommand {
                id: "logoff".to_string(),
                name: "注销".to_string(),
                description: "注销当前用户".to_string(),
                command: "shutdown /l".to_string(),
                icon: None,
            },
            SystemCommand {
                id: "lock".to_string(),
                name: "锁屏".to_string(),
                description: "锁定计算机".to_string(),
                command: "rundll32.exe user32.dll,LockWorkStation".to_string(),
                icon: None,
            },
            SystemCommand {
                id: "sleep".to_string(),
                name: "睡眠".to_string(),
                description: "进入睡眠模式".to_string(),
                command: "rundll32.exe powrprof.dll,SetSuspendState 0,1,0".to_string(),
                icon: None,
            },
            SystemCommand {
                id: "hibernate".to_string(),
                name: "休眠".to_string(),
                description: "进入休眠模式".to_string(),
                command: "rundll32.exe powrprof.dll,SetSuspendState 1,1,0".to_string(),
                icon: None,
            },
            SystemCommand {
                id: "control".to_string(),
                name: "控制面板".to_string(),
                description: "打开控制面板".to_string(),
                command: "control".to_string(),
                icon: None,
            },
            SystemCommand {
                id: "settings".to_string(),
                name: "设置".to_string(),
                description: "打开 Windows 设置".to_string(),
                command: "ms-settings:".to_string(),
                icon: None,
            },
            SystemCommand {
                id: "taskmgr".to_string(),
                name: "任务管理器".to_string(),
                description: "打开任务管理器".to_string(),
                command: "taskmgr".to_string(),
                icon: None,
            },
            SystemCommand {
                id: "explorer".to_string(),
                name: "文件资源管理器".to_string(),
                description: "打开文件资源管理器".to_string(),
                command: "explorer".to_string(),
                icon: None,
            },
            SystemCommand {
                id: "cmd".to_string(),
                name: "命令提示符".to_string(),
                description: "打开命令提示符".to_string(),
                command: "cmd".to_string(),
                icon: None,
            },
            SystemCommand {
                id: "powershell".to_string(),
                name: "PowerShell".to_string(),
                description: "打开 PowerShell".to_string(),
                command: "powershell".to_string(),
                icon: None,
            },
            SystemCommand {
                id: "tasklist".to_string(),
                name: "进程列表".to_string(),
                description: "查看当前运行的进程".to_string(),
                command: "tasklist".to_string(),
                icon: None,
            },
            SystemCommand {
                id: "ipconfig".to_string(),
                name: "IP 配置".to_string(),
                description: "查看网络 IP 配置".to_string(),
                command: "ipconfig".to_string(),
                icon: None,
            },
            SystemCommand {
                id: "ncpa.cpl".to_string(),
                name: "网络连接".to_string(),
                description: "打开网络连接设置".to_string(),
                command: "ncpa.cpl".to_string(),
                icon: None,
            },
            SystemCommand {
                id: "devmgmt".to_string(),
                name: "设备管理器".to_string(),
                description: "打开设备管理器".to_string(),
                command: "devmgmt.msc".to_string(),
                icon: None,
            },
            SystemCommand {
                id: "diskmgmt".to_string(),
                name: "磁盘管理".to_string(),
                description: "打开磁盘管理".to_string(),
                command: "diskmgmt.msc".to_string(),
                icon: None,
            },
            SystemCommand {
                id: "services".to_string(),
                name: "服务".to_string(),
                description: "打开服务管理".to_string(),
                command: "services.msc".to_string(),
                icon: None,
            },
            SystemCommand {
                id: "compmgmt".to_string(),
                name: "计算机管理".to_string(),
                description: "打开计算机管理".to_string(),
                command: "compmgmt.msc".to_string(),
                icon: None,
            },
            SystemCommand {
                id: "regedit".to_string(),
                name: "注册表编辑器".to_string(),
                description: "打开注册表编辑器".to_string(),
                command: "regedit".to_string(),
                icon: None,
            },
            SystemCommand {
                id: "mstsc".to_string(),
                name: "远程桌面".to_string(),
                description: "打开远程桌面连接".to_string(),
                command: "mstsc".to_string(),
                icon: None,
            },
            SystemCommand {
                id: "calc".to_string(),
                name: "计算器".to_string(),
                description: "打开计算器".to_string(),
                command: "calc".to_string(),
                icon: None,
            },
            SystemCommand {
                id: "notepad".to_string(),
                name: "记事本".to_string(),
                description: "打开记事本".to_string(),
                command: "notepad".to_string(),
                icon: None,
            },
            SystemCommand {
                id: "snippingtool".to_string(),
                name: "截图工具".to_string(),
                description: "打开截图工具".to_string(),
                command: "snippingtool".to_string(),
                icon: None,
            },
            SystemCommand {
                id: "osk".to_string(),
                name: "屏幕键盘".to_string(),
                description: "打开屏幕键盘".to_string(),
                command: "osk".to_string(),
                icon: None,
            },
            SystemCommand {
                id: "magnify".to_string(),
                name: "放大镜".to_string(),
                description: "打开放大镜".to_string(),
                command: "magnify".to_string(),
                icon: None,
            },
            SystemCommand {
                id: "narrator".to_string(),
                name: "讲述人".to_string(),
                description: "打开讲述人".to_string(),
                command: "narrator".to_string(),
                icon: None,
            },
            SystemCommand {
                id: "dpi".to_string(),
                name: "显示设置".to_string(),
                description: "打开显示设置".to_string(),
                command: "ms-settings:display".to_string(),
                icon: None,
            },
            SystemCommand {
                id: "sound".to_string(),
                name: "声音设置".to_string(),
                description: "打开声音设置".to_string(),
                command: "ms-settings:sound".to_string(),
                icon: None,
            },
            SystemCommand {
                id: "bluetooth".to_string(),
                name: "蓝牙设置".to_string(),
                description: "打开蓝牙设置".to_string(),
                command: "ms-settings:bluetooth".to_string(),
                icon: None,
            },
            SystemCommand {
                id: "wifi".to_string(),
                name: "WiFi 设置".to_string(),
                description: "打开 WiFi 设置".to_string(),
                command: "ms-settings:network".to_string(),
                icon: None,
            },
            SystemCommand {
                id: "apps".to_string(),
                name: "应用设置".to_string(),
                description: "打开应用设置".to_string(),
                command: "ms-settings:appsfeatures".to_string(),
                icon: None,
            },
            SystemCommand {
                id: "date".to_string(),
                name: "日期和时间".to_string(),
                description: "打开日期和时间设置".to_string(),
                command: "ms-settings:dateandtime".to_string(),
                icon: None,
            },
        ];

        Self { enabled: true, commands }
    }

    fn execute_command(&self, command: &str) -> Result<()> {
        if command.starts_with("ms-settings:") || command.starts_with("ms-") {
            std::process::Command::new("cmd").args(["/c", "start", "", command]).spawn()?;
        } else {
            std::process::Command::new("cmd").args(["/c", "start", "", command]).spawn()?;
        }
        Ok(())
    }
}

impl Plugin for SystemCommandsPlugin {
    fn id(&self) -> &str {
        "system_commands"
    }

    fn name(&self) -> &str {
        "系统命令"
    }

    fn description(&self) -> &str {
        "快速执行系统命令和打开系统工具"
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
        log::info!("初始化系统命令插件...");
        log::info!("已加载 {} 个系统命令", self.commands.len());
        Ok(())
    }

    fn search(&self, query: &str, limit: usize) -> Result<Vec<SearchResult>> {
        let mut results = Vec::new();
        let query_lower = query.to_lowercase();

        for cmd in &self.commands {
            if cmd.name.to_lowercase().contains(&query_lower)
                || cmd.description.to_lowercase().contains(&query_lower)
                || cmd.id.to_lowercase().contains(&query_lower)
            {
                results.push(
                    SearchResult::new(
                        format!("system_commands:{}", cmd.id),
                        cmd.name.clone(),
                        cmd.description.clone(),
                        ResultType::Command,
                        90,
                        ActionData::ExecuteCommand { command: cmd.command.clone() },
                    )
                    .with_icon(cmd.icon.clone()),
                );

                if results.len() >= limit {
                    break;
                }
            }
        }

        Ok(results)
    }

    fn execute(&self, result: &SearchResult) -> Result<()> {
        if let ActionData::ExecuteCommand { command } = &result.action {
            self.execute_command(command)?;
        }
        Ok(())
    }

    fn refresh(&mut self) -> Result<()> {
        Ok(())
    }
}

impl Default for SystemCommandsPlugin {
    fn default() -> Self {
        Self::new()
    }
}
