use crate::i18n::keys::CommandKey as Key;

/// 返回美式英文 Tauri 命令错误根因文案。
pub fn label(key: Key) -> &'static str {
    match key {
        Key::DragSourceFilesMissing => "The dragged source files no longer exist",
        Key::DragImageMissing => "The image file no longer exists",
        Key::DragTextEmpty => "Text content is empty",
        Key::ExternalUrlUnsupported => "Only links starting with http or https can be opened",
        Key::SyncNotRunning => "Turn on cross-device sync first",
        Key::SyncNoDevice => {
            "No device showing a pairing code was found. Enter its IP address and try again"
        }
        Key::SyncCodeInvalid => "The pairing code is wrong or has expired",
        Key::SyncAddressInvalid => "The IP address is invalid",
        Key::SyncUnreachable => {
            "Cannot reach that IP. Make sure both devices are on the same network and sync is on"
        }
    }
}
