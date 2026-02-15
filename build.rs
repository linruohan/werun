/// 构建脚本
///
/// 在 Windows 平台上编译资源文件以嵌入应用程序图标
fn main() {
    #[cfg(target_os = "windows")]
    {
        let mut res = winres::WindowsResource::new();
        res.set_icon("assets/icon.ico");
        res.set("ProductName", "WeRun");
        res.set("FileDescription", "WeRun - 高效启动器");
        res.set("LegalCopyright", "Copyright (C) 2024");
        res.compile().unwrap();
    }
}
