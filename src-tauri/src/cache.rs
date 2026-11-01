use serde::Serialize;
use std::fs;
use std::io;
use std::path::{Path, PathBuf};

const MAX_ENTRIES: usize = 10_000;
const MAX_BYTES: u64 = 1024 * 1024 * 1024;

#[derive(Debug, Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct CacheTarget {
    pub label: String,
    pub exists: bool,
}

#[derive(Debug, Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct CachePlan {
    pub platform: String,
    pub targets: Vec<CacheTarget>,
    pub dns_flush_available: bool,
    pub preserved: Vec<String>,
    pub warnings: Vec<String>,
}

#[derive(Debug, Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct CacheCleanupResult {
    pub status: String,
    pub cleaned_entries: usize,
    pub cleaned_bytes: u64,
    pub dns_flush_requested: bool,
    pub warnings: Vec<String>,
}

#[derive(Default)]
struct CleanupStats {
    entries: usize,
    bytes: u64,
}

pub fn plan() -> CachePlan {
    let targets = cache_targets()
        .into_iter()
        .map(|(label, path)| CacheTarget {
            label: label.to_string(),
            exists: path.is_dir(),
        })
        .collect();
    CachePlan {
        platform: std::env::consts::OS.to_string(),
        targets,
        dns_flush_available: cfg!(any(target_os = "macos", target_os = "windows")),
        preserved: vec![
            "Cookie 与登录会话".to_string(),
            "密码、书签和历史".to_string(),
            "Local Storage / IndexedDB".to_string(),
            "浏览器扩展与用户文件".to_string(),
        ],
        warnings: vec![
            "只清理可重建缓存，不删除站点数据或登录凭据".to_string(),
            "正在运行的浏览器或 Claude 进程可能让部分缓存无法删除".to_string(),
            "DNS 刷新不会改变 DNS 服务器或代理线路".to_string(),
        ],
    }
}

pub fn clean(confirm: bool) -> Result<CacheCleanupResult, String> {
    if !confirm {
        return Err("清理缓存前必须明确确认影响范围".to_string());
    }

    let mut stats = CleanupStats::default();
    for (_, path) in cache_targets() {
        if path.is_dir() {
            clean_directory(&path, &mut stats).map_err(|error| io_error("清理缓存", error))?;
        }
    }

    let dns_flush_requested = flush_dns();
    Ok(CacheCleanupResult {
        status: "cleaned".to_string(),
        cleaned_entries: stats.entries,
        cleaned_bytes: stats.bytes,
        dns_flush_requested,
        warnings: vec![
            "缓存清理不会触碰 Cookie、登录、密码、书签、历史或站点数据库".to_string(),
            "请重新启动浏览器、Clash Verge Rev 和 Claude Desktop 后再进行验收".to_string(),
        ],
    })
}

fn cache_targets() -> Vec<(&'static str, PathBuf)> {
    #[cfg(target_os = "windows")]
    {
        return windows_cache_targets();
    }

    #[cfg(not(target_os = "windows"))]
    {
        let Some(home) = std::env::var_os("HOME").map(PathBuf::from) else {
            return Vec::new();
        };

        #[cfg(target_os = "macos")]
        {
            return vec![
                (
                    "Clash Verge Rev 网络缓存",
                    home.join("Library/Caches/io.github.clash-verge-rev.clash-verge-rev"),
                ),
                (
                    "Chrome HTTP 缓存",
                    home.join("Library/Caches/Google/Chrome/Default/Cache"),
                ),
                (
                    "Edge HTTP 缓存",
                    home.join("Library/Caches/Microsoft Edge/Default/Cache"),
                ),
                (
                    "Claude Desktop 可重建缓存",
                    home.join("Library/Caches/com.anthropic.claude"),
                ),
            ];
        }

        #[cfg(not(any(target_os = "macos", target_os = "windows")))]
        {
            vec![
                (
                    "Clash Verge Rev 网络缓存",
                    home.join(".cache/io.github.clash-verge-rev.clash-verge-rev"),
                ),
                (
                    "Chrome HTTP 缓存",
                    home.join(".cache/google-chrome/Default/Cache"),
                ),
            ]
        }
    }
}

#[cfg(target_os = "windows")]
fn windows_cache_targets() -> Vec<(&'static str, PathBuf)> {
    let Some(local_app_data) = std::env::var_os("LOCALAPPDATA").map(PathBuf::from) else {
        return Vec::new();
    };
    vec![
        (
            "Clash Verge Rev 网络缓存",
            local_app_data.join("io.github.clash-verge-rev.clash-verge-rev/cache"),
        ),
        (
            "Chrome HTTP 缓存",
            local_app_data.join("Google/Chrome/User Data/Default/Cache"),
        ),
        (
            "Edge HTTP 缓存",
            local_app_data.join("Microsoft/Edge/User Data/Default/Cache"),
        ),
        (
            "Claude Desktop 可重建缓存",
            local_app_data.join("Claude/Cache"),
        ),
    ]
}

fn clean_directory(path: &Path, stats: &mut CleanupStats) -> io::Result<()> {
    for entry in fs::read_dir(path)? {
        let entry = entry?;
        let child = entry.path();
        let metadata = fs::symlink_metadata(&child)?;
        if metadata.file_type().is_symlink() {
            continue;
        }
        if metadata.is_dir() {
            clean_directory(&child, stats)?;
            let _ = fs::remove_dir(&child);
        } else if metadata.is_file() {
            let size = metadata.len();
            if stats.entries >= MAX_ENTRIES || stats.bytes.saturating_add(size) > MAX_BYTES {
                return Err(io::Error::new(
                    io::ErrorKind::FileTooLarge,
                    "缓存清理超过安全上限",
                ));
            }
            fs::remove_file(&child)?;
            stats.entries += 1;
            stats.bytes += size;
        }
    }
    Ok(())
}

fn flush_dns() -> bool {
    #[cfg(target_os = "macos")]
    {
        let first = std::process::Command::new("/usr/bin/dscacheutil")
            .args(["-flushcache"])
            .status()
            .is_ok_and(|status| status.success());
        let second = std::process::Command::new("/usr/bin/killall")
            .args(["-HUP", "mDNSResponder"])
            .status()
            .is_ok_and(|status| status.success());
        return first || second;
    }

    #[cfg(target_os = "windows")]
    {
        return std::process::Command::new("ipconfig")
            .args(["/flushdns"])
            .status()
            .is_ok_and(|status| status.success());
    }

    #[cfg(not(any(target_os = "macos", target_os = "windows")))]
    {
        false
    }
}

fn io_error(action: &str, error: io::Error) -> String {
    format!("{action}失败：{}", error.kind())
}

#[cfg(test)]
mod tests {
    use super::{clean, plan};

    #[test]
    fn cache_plan_preserves_account_data() {
        let cache_plan = plan();

        assert!(cache_plan
            .preserved
            .iter()
            .any(|item| item.contains("Cookie")));
        assert!(clean(false).is_err());
    }
}
