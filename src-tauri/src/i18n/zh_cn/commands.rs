use crate::i18n::keys::CommandKey as Key;

/// 返回简体中文 Tauri 命令错误根因文案。
pub fn label(key: Key) -> &'static str {
    match key {
        Key::DragSourceFilesMissing => "拖拽源文件已不存在",
        Key::DragImageMissing => "图片文件已不存在",
        Key::DragTextEmpty => "文本内容为空",
        Key::ExternalUrlUnsupported => "只能打开 http 或 https 开头的链接",
        Key::SyncNotRunning => "请先开启跨设备同步",
        Key::SyncNoDevice => "没有发现正在显示匹配码的设备，可填写对方 IP 后重试",
        Key::SyncCodeInvalid => "匹配码错误或已过期",
        Key::SyncAddressInvalid => "IP 地址格式不正确",
        Key::SyncUnreachable => "无法连接该 IP，请确认两台设备在同一网络且对方已开启同步",
    }
}
