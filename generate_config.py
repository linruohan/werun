#!/usr/bin/env python3
"""
生成 WeRun 启动器的默认配置文件

该脚本会在项目根目录中创建 config.toml 文件
包含所有默认配置值
"""
import os
import platform

# 默认配置
def get_default_config_toml():
    """获取默认配置的 TOML 格式"""
    return '''# WeRun 启动器配置文件
# 项目根目录中的配置文件

[window]
width = 800.0
height = 500.0
opacity = 0.98
border_radius = 12.0
always_on_top = true
hide_on_blur = true
animation_duration_ms = 150

[theme]
current_theme = "dark"
follow_system = true

[search]
max_results = 50
debounce_ms = 50
enable_file_search = true
file_search_paths = []
file_ignore_patterns = [
    "*.tmp",
    "*.log",
    "node_modules",
    ".git"
]

[keybindings]
toggle_launcher = "Alt+Space"
navigate_up = "ArrowUp"
navigate_down = "ArrowDown"
confirm = "Enter"
close = "Escape"

[plugins]
enabled = [
    "app_launcher",
    "calculator",
    "clipboard"
]

[plugins.settings]
# 插件特定配置
'''

# 生成配置文件
def generate_config():
    """生成配置文件"""
    config_path = os.path.join(".", "config.toml")
    
    # 生成默认配置
    config_content = get_default_config_toml()
    
    # 写入配置文件
    with open(config_path, "w", encoding="utf-8") as f:
        f.write(config_content)
    
    print(f"配置文件已生成: {os.path.abspath(config_path)}")
    print("\n默认配置内容:")
    print(config_content)

if __name__ == "__main__":
    generate_config()
