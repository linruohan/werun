/// 剪贴板操作工具
///
/// 提供 Windows 剪贴板读写功能
use windows::Win32::Foundation::{HANDLE, HGLOBAL, HWND};
use windows::Win32::System::DataExchange::{
    CloseClipboard, EmptyClipboard, GetClipboardData, OpenClipboard, SetClipboardData,
};
use windows::Win32::System::Memory::{GlobalAlloc, GlobalLock, GlobalUnlock, GMEM_MOVEABLE};
use windows::Win32::System::Ole::CF_UNICODETEXT;

/// 剪贴板管理器
pub struct ClipboardManager;

impl ClipboardManager {
    /// 创建新的剪贴板管理器
    pub fn new() -> Self {
        Self
    }

    /// 设置文本到剪贴板
    pub fn set_text(&self, text: &str) -> anyhow::Result<()> {
        unsafe {
            // 打开剪贴板
            OpenClipboard(HWND(std::ptr::null_mut()))?;

            // 清空剪贴板
            EmptyClipboard()?;

            // 将文本转换为宽字符
            let wide_text: Vec<u16> = text.encode_utf16().chain(std::iter::once(0)).collect();
            let size = wide_text.len() * std::mem::size_of::<u16>();

            // 分配全局内存
            let h_global: HGLOBAL = GlobalAlloc(GMEM_MOVEABLE, size)?;

            // 锁定内存并复制数据
            let ptr = GlobalLock(h_global) as *mut u16;
            if ptr.is_null() {
                return Err(anyhow::anyhow!("无法锁定全局内存"));
            }

            std::ptr::copy_nonoverlapping(wide_text.as_ptr(), ptr, wide_text.len());

            // 解锁内存
            GlobalUnlock(h_global)?;

            // 设置剪贴板数据
            SetClipboardData(CF_UNICODETEXT.0 as u32, HANDLE(h_global.0 as *mut _))?;

            // 关闭剪贴板
            CloseClipboard()?;

            Ok(())
        }
    }

    /// 从剪贴板获取文本
    pub fn get_text(&self) -> anyhow::Result<String> {
        unsafe {
            // 打开剪贴板
            OpenClipboard(HWND(std::ptr::null_mut()))?;

            // 获取剪贴板数据
            let h_data: HGLOBAL = HGLOBAL(GetClipboardData(CF_UNICODETEXT.0 as u32)?.0);

            // 锁定内存
            let ptr = GlobalLock(h_data) as *const u16;
            if ptr.is_null() {
                CloseClipboard()?;
                return Err(anyhow::anyhow!("无法锁定剪贴板数据"));
            }

            // 计算字符串长度
            let mut len = 0;
            while *ptr.add(len) != 0 {
                len += 1;
            }

            // 转换为 String
            let slice = std::slice::from_raw_parts(ptr, len);
            let text = String::from_utf16(slice)?;

            // 解锁内存
            GlobalUnlock(h_data)?;

            // 关闭剪贴板
            CloseClipboard()?;

            Ok(text)
        }
    }

    /// 检查剪贴板是否有文本
    pub fn has_text(&self) -> bool {
        unsafe {
            if OpenClipboard(HWND(std::ptr::null_mut())).is_ok() {
                let has_data = GetClipboardData(CF_UNICODETEXT.0 as u32).is_ok();
                let _ = CloseClipboard();
                has_data
            } else {
                false
            }
        }
    }
}

impl Default for ClipboardManager {
    fn default() -> Self {
        Self::new()
    }
}
