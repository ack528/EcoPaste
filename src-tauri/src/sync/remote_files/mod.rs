//! 接收侧「虚拟文件」剪贴板：对端复制的文件只放清单，用户在任意应用粘贴时才按需拉取内容。
//!
//! - Windows：自实现 `IDataObject`（`FileGroupDescriptorW` + 按需 `IStream` 的 `FileContents`）；
//! - macOS：TODO，需要 `NSFilePromiseProvider`（当前环境无法验证），暂时仅记录日志。

#[cfg(target_os = "windows")]
mod windows;

use tauri::AppHandle;

use super::manager::SyncState;
use super::protocol::FileEntry;

/// 测试用：被捕获的虚拟文件 offer，替代真实剪贴板放置。
#[cfg(test)]
#[allow(dead_code)]
pub struct Captured {
    pub state: SyncState,
    pub peer_id: String,
    pub offer: String,
    pub entries: Vec<FileEntry>,
}

/// 测试用：`Some` 时 [`present`] 只记录 offer，不触碰系统剪贴板。
#[cfg(test)]
pub static CAPTURED: std::sync::Mutex<Option<Vec<Captured>>> = std::sync::Mutex::new(None);

#[cfg(all(test, target_os = "windows"))]
pub use windows::create_data_object;

/// 把对端文件清单呈现为本机剪贴板上的虚拟文件。
pub fn present(
    app: &AppHandle,
    state: SyncState,
    peer_id: &str,
    offer: &str,
    entries: Vec<FileEntry>,
) {
    #[cfg(test)]
    if let Some(captured) = CAPTURED.lock().expect("capture poisoned").as_mut() {
        captured.push(Captured {
            state,
            peer_id: peer_id.to_owned(),
            offer: offer.to_owned(),
            entries,
        });
        return;
    }

    #[cfg(target_os = "windows")]
    windows::present(app, state, peer_id, offer, entries);

    #[cfg(not(target_os = "windows"))]
    {
        let _ = (app, state, offer);
        log::info!(
            "sync: received {} file entries from {peer_id}; virtual file clipboard is not implemented on this platform yet",
            entries.len()
        );
    }
}
