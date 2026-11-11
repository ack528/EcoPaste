//! 局域网剪贴板同步：mDNS 发现 + 6 位配对码（SPAKE2）配对 + 加密 TCP 通道。
//!
//! 结构：
//! - `identity`：本机设备 id 与已配对设备表（含长期密钥）；
//! - `pairing`：配对码生命周期与配对握手；
//! - `session`：已配对设备间的双向认证握手；
//! - `protocol`：消息定义与 AEAD 帧；
//! - `discovery`：mDNS 通告与浏览；
//! - `manager`：监听、连接、广播、配对入口；
//! - `apply`：接收到的文本 / 图片写入剪贴板并入库；
//! - `files` / `remote_files`：文件清单与按需拉取、接收侧虚拟文件剪贴板。

mod apply;
mod discovery;
#[cfg(test)]
mod e2e_tests;
mod files;
pub mod identity;
mod manager;
mod pairing;
mod protocol;
mod remote_files;
mod session;

pub use manager::{PairError, PeerStatus, SyncState, SyncStatus};
pub use pairing::PairingCode;

use tauri::{AppHandle, Manager};

use crate::core::Result;
use crate::db::models::ClipboardItem;

/// 设备列表 / 在线状态变化。与前端 `src/constants/events.ts` 同步。
pub const DEVICES_EVENT: &str = "sync://devices";
/// 配对进度：`{ state: "success" | "failed" | "expired", peer?: string }`。
pub const PAIRING_EVENT: &str = "sync://pairing";

/// 在 setup 中调用：注册 [`SyncState`]，设置已开启时立即启动。
pub fn init(app: &AppHandle) -> Result<()> {
    let store = identity::DeviceStore::load(app)?;
    let state = SyncState::new(app.clone(), store);
    app.manage(state.clone());

    tauri::async_runtime::spawn(async move {
        state.apply_settings().await;
    });
    Ok(())
}

/// 设置变化后调用，按 `sync.enabled` / 设备名启停服务。
pub fn apply_settings(app: &AppHandle) {
    let Some(state) = app.try_state::<SyncState>() else {
        return;
    };
    let state = state.inner().clone();
    tauri::async_runtime::spawn(async move {
        state.apply_settings().await;
    });
}

/// 剪贴板监听新入库一条本机复制后调用，推送给已配对设备。
pub fn broadcast_item(app: &AppHandle, item: &ClipboardItem) {
    if let Some(state) = app.try_state::<SyncState>() {
        state.broadcast(item.clone());
    }
}
