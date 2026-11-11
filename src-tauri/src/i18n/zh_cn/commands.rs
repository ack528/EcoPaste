use crate::i18n::keys::CommandKey as Key;

/// 返回简体中文 Tauri 命令错误根因文案。
pub fn label(key: Key) -> &'static str {
    match key {
        Key::DragSourceFilesMissing => "拖拽源文件已不存在",
        Key::DragImageMissing => "图片文件已不存在",
        Key::DragTextEmpty => "文本内容为空",
        Key::ExternalUrlUnsupported => "只能打开 http 或 https 开头的链接",
        Key::SyncNotRunning => "请先开启跨设备同步",
        Key::SyncNoDevice => "没有找到正在显示匹配码的设备",
        Key::SyncCodeInvalid => "匹配码错误或已过期",
    }
}
