/// WeRun - Windows 启动器库
///
/// 提供启动器的核心功能和组件
pub mod app;
pub mod core;
pub mod platform;
pub mod plugins;
pub mod ui;
pub mod utils;

/// 版本信息
pub const VERSION: &str = env!("CARGO_PKG_VERSION");
pub const NAME: &str = env!("CARGO_PKG_NAME");
