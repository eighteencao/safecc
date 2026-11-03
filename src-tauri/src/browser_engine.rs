use serde::Serialize;
use std::path::{Path, PathBuf};
#[cfg(target_os = "macos")]
use std::process::Command;

const UNPACKED_ENGINE_NAME: &str = "safecc-browser.app";

#[derive(Debug, Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct FingerprintEngine {
    pub available: bool,
    pub source: String,
    pub label: String,
    pub version: String,
    pub engine_path: String,
    pub notes: Vec<String>,
}

pub fn detect() -> FingerprintEngine {
    if let Some(engine) = engine_from(candidate_paths()) {
        engine
    } else {
        FingerprintEngine {
            available: false,
            source: "missing".to_string(),
            label: "未发现指纹 Chromium 引擎".to_string(),
            version: "—".to_string(),
            engine_path: String::new(),
            notes: vec![
                "safecc 会检测本机已安装的 Fingerprint Chromium；不会把普通 Chrome/Edge 独立 Profile 称作指纹浏览器".to_string(),
                "若已安装 CCBaga-B，可直接接入；也可以把独立引擎放在 safecc 应用资源目录".to_string(),
                "当前安装包不分发第三方浏览器二进制；缺失时仍可使用独立 Chromium Profile 隔离 Cookie 和站点数据".to_string(),
            ],
        }
    }
}

pub fn launch(profile_dir: &Path, target_region: &str, seed: u32) -> Result<String, String> {
    let engine = detect();
    if !engine.available {
        return Err(
            "未发现指纹 Chromium 引擎；请先安装兼容引擎或使用独立 Chromium Profile".to_string(),
        );
    }
    if !profile_dir.is_dir() {
        return Err("账号档案的浏览器资料目录不存在".to_string());
    }
    if seed == 0 {
        return Err("指纹种子必须是 1 至 4294967295 的整数".to_string());
    }
    let args = launch_args(profile_dir, target_region, seed);
    #[cfg(not(target_os = "macos"))]
    let _ = &args;

    #[cfg(target_os = "macos")]
    {
        let executable = Path::new(&engine.engine_path).join("Contents/MacOS/Chromium");
        if !executable.is_file() {
            return Err("指纹 Chromium 可执行文件不存在或签名结构不完整".to_string());
        }
        Command::new(executable)
            .args(&args)
            .stdin(std::process::Stdio::null())
            .stdout(std::process::Stdio::null())
            .stderr(std::process::Stdio::null())
            .spawn()
            .map_err(|_| "无法启动指纹 Chromium".to_string())?;
    }

    #[cfg(target_os = "windows")]
    {
        return Err(
            "Windows 指纹浏览器引擎尚未随 safecc 安装包提供；请在真实 Windows 环境接入后验收"
                .to_string(),
        );
    }

    #[cfg(not(any(target_os = "macos", target_os = "windows")))]
    {
        return Err("当前平台暂不支持启动指纹 Chromium".to_string());
    }

    #[cfg(target_os = "macos")]
    Ok(format!(
        "已用 {} 指纹引擎打开独立档案；种子 {}，地区 {}。实际网页指纹与出口仍需检测页复核。",
        engine.label, seed, target_region
    ))
}

fn engine_from(paths: Vec<(String, PathBuf)>) -> Option<FingerprintEngine> {
    for (source, app) in paths {
        let executable = app.join("Contents/MacOS/Chromium");
        if !executable.is_file() {
            continue;
        }
        let version = app
            .join("Contents/Info.plist")
            .is_file()
            .then(|| read_plist_version(&app.join("Contents/Info.plist")))
            .flatten()
            .unwrap_or_else(|| "未知版本".to_string());
        let is_ccbaga = source == "ccbaga";
        return Some(FingerprintEngine {
            available: true,
            source,
            label: if is_ccbaga {
                "已安装 CCBaga-B / Fingerprint Chromium".to_string()
            } else {
                "safecc 独立 Fingerprint Chromium".to_string()
            },
            version,
            engine_path: app.display().to_string(),
            notes: vec![
                "每个账号档案使用独立 user-data-dir 和稳定的指纹种子".to_string(),
                "语言、时区和 WebRTC UDP 参数按所选地区注入；网络出口仍由 Clash TUN 决定"
                    .to_string(),
                "safecc 不会复制、覆盖或读取第三方浏览器中的登录资料".to_string(),
            ],
        });
    }
    None
}

#[cfg(target_os = "macos")]
fn read_plist_version(path: &Path) -> Option<String> {
    let output = Command::new("/usr/bin/plutil")
        .args(["-extract", "CFBundleShortVersionString", "raw", "-o", "-"])
        .arg(path)
        .output()
        .ok()?;
    if !output.status.success() {
        return None;
    }
    let value = String::from_utf8(output.stdout).ok()?.trim().to_string();
    (!value.is_empty()).then_some(value)
}

#[cfg(not(target_os = "macos"))]
fn read_plist_version(_path: &Path) -> Option<String> {
    None
}

fn candidate_paths() -> Vec<(String, PathBuf)> {
    let mut paths = Vec::new();
    if let Some(resource_dir) = std::env::var_os("SAFECC_RESOURCE_DIR").map(PathBuf::from) {
        paths.push((
            "bundled".to_string(),
            resource_dir.join(UNPACKED_ENGINE_NAME),
        ));
    }
    if let Some(home) = std::env::var_os("HOME").map(PathBuf::from) {
        paths.push((
            "bundled".to_string(),
            home.join("Applications").join(UNPACKED_ENGINE_NAME),
        ));
    }
    paths.push((
        "ccbaga".to_string(),
        PathBuf::from("/Applications/CCBaga-B.app"),
    ));
    if let Some(home) = std::env::var_os("HOME").map(PathBuf::from) {
        paths.push(("ccbaga".to_string(), home.join("Applications/CCBaga-B.app")));
    }
    paths
}

fn launch_args(profile_dir: &Path, target_region: &str, seed: u32) -> Vec<String> {
    let plan = crate::region::change_plan(target_region.to_string());
    let language = plan.language;
    let base_language = language.split('-').next().unwrap_or("en");
    let timezone = plan.timezone;
    vec![
        format!("--user-data-dir={}", profile_dir.display()),
        "--profile-directory=Default".to_string(),
        format!("--fingerprint={seed}"),
        "--fingerprint-brand=Chrome".to_string(),
        format!("--lang={language}"),
        format!("--accept-lang={language},{base_language}"),
        format!("--timezone={timezone}"),
        "--force-webrtc-ip-handling-policy=disable_non_proxied_udp".to_string(),
        "--disable-non-proxied-udp".to_string(),
        "--disable-quic".to_string(),
        "--no-first-run".to_string(),
        "--no-default-browser-check".to_string(),
        "https://ip.net.coffee/claude/".to_string(),
    ]
}

#[cfg(test)]
mod tests {
    use super::launch_args;
    use std::path::Path;

    #[test]
    fn fingerprint_launch_args_are_isolated_and_region_specific() {
        let args = launch_args(Path::new("/tmp/safecc-profile"), "日本 · 东京", 42);
        assert!(args
            .iter()
            .any(|arg| arg == "--user-data-dir=/tmp/safecc-profile"));
        assert!(args.iter().any(|arg| arg == "--fingerprint=42"));
        assert!(args.iter().any(|arg| arg == "--lang=ja-JP"));
        assert!(args.iter().any(|arg| arg == "--timezone=Asia/Tokyo"));
        assert!(args
            .iter()
            .any(|arg| arg == "--force-webrtc-ip-handling-policy=disable_non_proxied_udp"));
        assert!(args
            .iter()
            .any(|arg| arg == "https://ip.net.coffee/claude/"));
    }

    #[test]
    fn candidate_paths_are_fixed_and_do_not_accept_user_input() {
        let paths = super::candidate_paths();
        assert!(!paths.is_empty());
        assert!(paths
            .iter()
            .any(|(source, path)| source == "ccbaga"
                && path == &Path::new("/Applications/CCBaga-B.app")));
    }
}
