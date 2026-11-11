//! 文件同步：发送侧的清单构建与按需读取；接收侧的「虚拟文件」剪贴板（平台实现）。

use std::path::{Path, PathBuf};

use anyhow::{anyhow, bail, Result};
use tokio::io::{AsyncReadExt, AsyncSeekExt};

use super::protocol::FileEntry;

/// 单次复制最多同步的条目数（含目录递归），防止误复制巨型目录树。
pub const MAX_ENTRIES: usize = 20_000;

/// 发送侧登记的一次文件 offer：与 [`FileEntry`] 一一对应的本地路径（目录为 `None`）。
pub struct LocalOffer {
    pub paths: Vec<Option<PathBuf>>,
    pub created: std::time::Instant,
}

/// 把复制的路径列表展开成清单；目录递归展开，符号链接与特殊文件跳过。
pub fn build_offer(roots: &[String]) -> Result<(Vec<FileEntry>, Vec<Option<PathBuf>>)> {
    let mut entries = Vec::new();
    let mut paths = Vec::new();

    for root in roots {
        let root = Path::new(root);
        let Some(name) = root.file_name().and_then(|n| n.to_str()) else {
            continue;
        };

        let meta = match std::fs::symlink_metadata(root) {
            Ok(meta) => meta,
            Err(err) => {
                log::warn!("sync: skip unreadable path {root:?}: {err}");
                continue;
            }
        };

        if meta.is_file() {
            entries.push(FileEntry {
                rel_path: name.to_owned(),
                is_dir: false,
                size: meta.len(),
            });
            paths.push(Some(root.to_path_buf()));
        } else if meta.is_dir() {
            for walked in walkdir::WalkDir::new(root).follow_links(false) {
                let Ok(walked) = walked else { continue };
                let Ok(rel) = walked.path().strip_prefix(root) else {
                    continue;
                };
                let rel_path = std::iter::once(name.to_owned())
                    .chain(
                        rel.components()
                            .map(|c| c.as_os_str().to_string_lossy().into_owned()),
                    )
                    .collect::<Vec<_>>()
                    .join("/");

                let file_type = walked.file_type();
                if file_type.is_dir() {
                    entries.push(FileEntry {
                        rel_path,
                        is_dir: true,
                        size: 0,
                    });
                    paths.push(None);
                } else if file_type.is_file() {
                    let size = walked.metadata().map(|m| m.len()).unwrap_or(0);
                    entries.push(FileEntry {
                        rel_path,
                        is_dir: false,
                        size,
                    });
                    paths.push(Some(walked.path().to_path_buf()));
                }

                if entries.len() > MAX_ENTRIES {
                    bail!("too many files to sync");
                }
            }
        }
    }

    if entries.is_empty() {
        bail!("no files to sync");
    }
    Ok((entries, paths))
}

/// 读取 offer 中某个文件的一段；越过文件末尾返回空。
pub async fn read_range(offer: &LocalOffer, index: u32, offset: u64, len: u32) -> Result<Vec<u8>> {
    let path = offer
        .paths
        .get(index as usize)
        .and_then(|p| p.as_ref())
        .ok_or_else(|| anyhow!("file index out of range"))?;

    let mut file = tokio::fs::File::open(path).await?;
    file.seek(std::io::SeekFrom::Start(offset)).await?;

    let mut buf = vec![0u8; len as usize];
    let mut filled = 0;
    while filled < buf.len() {
        let n = file.read(&mut buf[filled..]).await?;
        if n == 0 {
            break;
        }
        filled += n;
    }
    buf.truncate(filled);
    Ok(buf)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn offer_expands_directories_and_serves_ranges() {
        let root = std::env::temp_dir().join(format!("ecopaste-offer-{}", uuid::Uuid::new_v4()));
        let dir = root.join("album");
        std::fs::create_dir_all(dir.join("sub")).unwrap();
        std::fs::write(dir.join("a.txt"), b"hello world").unwrap();
        std::fs::write(dir.join("sub").join("b.txt"), b"bee").unwrap();

        let (entries, paths) = build_offer(&[dir.to_string_lossy().into_owned()]).unwrap();
        let names: Vec<_> = entries
            .iter()
            .map(|e| (e.rel_path.as_str(), e.is_dir))
            .collect();
        assert!(names.contains(&("album", true)));
        assert!(names.contains(&("album/a.txt", false)));
        assert!(names.contains(&("album/sub/b.txt", false)));

        let offer = LocalOffer {
            paths,
            created: std::time::Instant::now(),
        };
        let index = entries
            .iter()
            .position(|e| e.rel_path == "album/a.txt")
            .unwrap() as u32;
        assert_eq!(read_range(&offer, index, 6, 100).await.unwrap(), b"world");
        assert!(read_range(&offer, index, 50, 10).await.unwrap().is_empty());
        assert!(read_range(&offer, 0, 0, 10).await.is_err());

        std::fs::remove_dir_all(root).ok();
    }
}
