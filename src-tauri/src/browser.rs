use serde::Serialize;
use serde_json::{json, Value};
use std::fs;
use std::io;
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum BrowserKind {
    Chrome,
    Edge,
    Firefox,
    Safari,
    Unknown,
}

impl BrowserKind {
    fn id(self) -> &'static str {
        match self {
            Self::Chrome => "chrome",
            Self::Edge => "edge",
            Self::Firefox => "firefox",
            Self::Safari => "safari",
            Self::Unknown => "unknown",
        }
    }

    fn label(self) -> &'static str {
        match self {
            Self::Chrome => "Google Chrome",
            Self::Edge => "Microsoft Edge",
            Self::Firefox => "Firefox",
            Self::Safari => "Safari",
            Self::Unknown => "未识别浏览器",
        }
    }
}

#[derive(Debug, Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct BrowserChangePlan {
    pub platform: String,
    pub browser: String,
    pub browser_label: String,
    pub target_region: String,
    pub language: String,
    pub profile_exists: bool,
    pub automatic: bool,
    pub actions: Vec<String>,
    pub permission_prompts: Vec<String>,
    pub restart_requirements: Vec<String>,
    pub manual_steps: Vec<String>,
    pub warnings: Vec<String>,
}

#[derive(Debug, Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct BrowserApplyResult {
    pub status: String,
    pub browser: String,
    pub changed: Vec<String>,
    pub backup_id: Option<String>,
    pub requires_restart: bool,
    pub warnings: Vec<String>,
}

pub fn change_plan(target_region: String, default_browser: Option<String>) -> BrowserChangePlan {
    let browser = classify(default_browser.as_deref());
    let language = language_for(&target_region);
    let profile_exists = profile_path(browser).is_some_and(|path| path.is_file());
    let automatic = matches!(browser, BrowserKind::Chrome | BrowserKind::Edge) && profile_exists;
    let mut manual_steps = vec![
        "定位权限保持由浏览器设置控制；无法稳定写入时请在设置中选择询问或阻止".to_string(),
        "安全 DNS / WebRTC UDP 以真实检测页结果为准，不通过脚本伪造".to_string(),
    ];
    if !automatic {
        manual_steps.insert(
            0,
            "当前浏览器没有安全的自动写入入口，请打开浏览器设置完成语言和定位确认".to_string(),
        );
    }

    BrowserChangePlan {
        platform: std::env::consts::OS.to_string(),
        browser: browser.id().to_string(),
        browser_label: browser.label().to_string(),
        target_region,
        language,
        profile_exists,
        automatic,
        actions: vec![
            "让浏览器语言偏好与目标地区一致".to_string(),
            "将网站定位默认值设为阻止，避免静默暴露位置".to_string(),
            "保持时区跟随系统，不注入网页脚本伪造时区".to_string(),
            "关闭 Chromium 安全 DNS（DoH），让 DNS 由 Clash TUN 接管".to_string(),
        ],
        permission_prompts: vec![
            "写入浏览器 Preferences 前要求用户确认，并在写入前保存本地快照".to_string(),
            "如果浏览器仍在运行，safecc 会停止操作并要求先退出浏览器".to_string(),
        ],
        restart_requirements: vec![
            "修改后需要重新启动浏览器；正在运行的浏览器不会被强制结束".to_string()
        ],
        manual_steps,
        warnings: vec![
            "safecc 不读取、显示或上传浏览器 Cookie、密码、历史和站点数据".to_string(),
            "浏览器偏好备份可能包含账户相关设置，只保存在本机受保护目录".to_string(),
            "浏览器策略可能覆盖安全 DNS 或 WebRTC 偏好，最终结果必须以验收页为准".to_string(),
        ],
    }
}

pub fn apply(
    target_region: String,
    default_browser: Option<String>,
    confirmed: bool,
) -> Result<BrowserApplyResult, String> {
    if !confirmed {
        return Err("修改浏览器环境前必须明确确认影响范围".to_string());
    }

    let browser = classify(default_browser.as_deref());
    let Some(profile_path) = profile_path(browser) else {
        return Err("当前默认浏览器没有安全的自动配置入口，请按页面提示手动设置".to_string());
    };
    if !profile_path.is_file() {
        return Err(format!(
            "未找到 {} 的默认配置文件，请先启动并退出浏览器后重试",
            browser.label()
        ));
    }
    if is_browser_running(browser) {
        return Err(format!(
            "请先完全退出 {}，safecc 不会强制结束浏览器进程",
            browser.label()
        ));
    }

    let original = fs::read(&profile_path).map_err(|error| io_error("读取浏览器偏好", error))?;
    let mut preferences: Value = serde_json::from_slice(&original)
        .map_err(|_| "浏览器 Preferences 不是可安全解析的 JSON，已停止修改".to_string())?;
    let language = language_for(&target_region);
    configure_preferences(&mut preferences, &language);

    let backup_id = backup_preferences(browser, &profile_path, &original)?;
    atomic_write_json(&profile_path, &preferences)?;

    Ok(BrowserApplyResult {
        status: "applied".to_string(),
        browser: browser.id().to_string(),
        changed: vec![
            format!("浏览器语言偏好 → {language}"),
            "网站定位默认值 → 阻止".to_string(),
            "浏览器安全 DNS / DoH → 关闭（由 Clash TUN 接管）".to_string(),
        ],
        backup_id: Some(backup_id),
        requires_restart: true,
        warnings: vec![
            "浏览器必须重新启动后再检查设置".to_string(),
            "WebRTC UDP、安全 DNS 和实际出口仍以验收页面为准".to_string(),
        ],
    })
}

fn configure_preferences(preferences: &mut Value, language: &str) {
    preferences["intl"]["accept_languages"] = Value::String(language.to_string());
    preferences["profile"]["default_content_setting_values"]["geolocation"] = json!(2);
    // Chrome/Edge may otherwise upgrade the system resolver to DoH and bypass
    // Clash's DNS hijack. The final detection page remains authoritative.
    preferences["dns_over_https"]["mode"] = Value::String("off".to_string());
}

fn classify(value: Option<&str>) -> BrowserKind {
    let value = value.unwrap_or_default().to_ascii_lowercase();
    if value.contains("chrome") || value.contains("google") {
        BrowserKind::Chrome
    } else if value.contains("edge") || value.contains("microsoft") {
        BrowserKind::Edge
    } else if value.contains("firefox") || value.contains("mozilla") {
        BrowserKind::Firefox
    } else if value.contains("safari") || value.contains("apple") {
        BrowserKind::Safari
    } else {
        BrowserKind::Unknown
    }
}

fn language_for(target_region: &str) -> String {
    let primary = crate::region::target_language(target_region);
    let base = primary.split('-').next().unwrap_or("en");
    if primary == "en-IE" {
        "en-IE,en-GB,en-US,en".to_string()
    } else if primary == "en-NZ" {
        "en-NZ,en-AU,en-US,en".to_string()
    } else if primary == "en-US" {
        "en-US,en".to_string()
    } else if primary.starts_with("en-") {
        format!("{primary},en-US,en")
    } else {
        format!("{primary},{base},en-US,en")
    }
}

fn profile_path(browser: BrowserKind) -> Option<PathBuf> {
    #[cfg(target_os = "macos")]
    {
        let home = std::env::var_os("HOME").map(PathBuf::from)?;
        return match browser {
            BrowserKind::Chrome => {
                Some(home.join("Library/Application Support/Google/Chrome/Default/Preferences"))
            }
            BrowserKind::Edge => {
                Some(home.join("Library/Application Support/Microsoft Edge/Default/Preferences"))
            }
            _ => None,
        };
    }

    #[cfg(target_os = "windows")]
    {
        let local_app_data = std::env::var_os("LOCALAPPDATA").map(PathBuf::from)?;
        return match browser {
            BrowserKind::Chrome => {
                Some(local_app_data.join("Google/Chrome/User Data/Default/Preferences"))
            }
            BrowserKind::Edge => {
                Some(local_app_data.join("Microsoft/Edge/User Data/Default/Preferences"))
            }
            _ => None,
        };
    }

    #[cfg(not(any(target_os = "macos", target_os = "windows")))]
    {
        let _ = browser;
        None
    }
}

fn is_browser_running(browser: BrowserKind) -> bool {
    #[cfg(target_os = "macos")]
    {
        let process = match browser {
            BrowserKind::Chrome => "Google Chrome",
            BrowserKind::Edge => "Microsoft Edge",
            _ => return false,
        };
        return std::process::Command::new("/usr/bin/pgrep")
            .args(["-x", process])
            .status()
            .is_ok_and(|status| status.success());
    }

    #[cfg(target_os = "windows")]
    {
        let process = match browser {
            BrowserKind::Chrome => "chrome.exe",
            BrowserKind::Edge => "msedge.exe",
            _ => return false,
        };
        return std::process::Command::new("tasklist")
            .args(["/FI", &format!("IMAGENAME eq {process}")])
            .output()
            .map(|output| {
                String::from_utf8_lossy(&output.stdout)
                    .to_ascii_lowercase()
                    .contains(&process.to_ascii_lowercase())
            })
            .unwrap_or(false);
    }

    #[cfg(not(any(target_os = "macos", target_os = "windows")))]
    {
        let _ = browser;
        false
    }
}

fn backup_preferences(
    browser: BrowserKind,
    source: &Path,
    contents: &[u8],
) -> Result<String, String> {
    let backup_id = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_err(|_| "系统时间不可用，无法创建浏览器快照".to_string())?
        .as_millis()
        .to_string();
    let root = snapshot_root().ok_or_else(|| "无法确定 safecc 浏览器快照目录".to_string())?;
    let directory = root.join(&backup_id);
    fs::create_dir_all(&directory).map_err(|error| io_error("创建浏览器快照目录", error))?;
    let destination = directory.join(format!("{}-Preferences", browser.id()));
    fs::write(&destination, contents).map_err(|error| io_error("保存浏览器快照", error))?;
    protect_file(&destination).map_err(|error| io_error("保护浏览器快照", error))?;
    let _ = source;
    Ok(backup_id)
}

fn snapshot_root() -> Option<PathBuf> {
    #[cfg(target_os = "macos")]
    {
        return std::env::var_os("HOME")
            .map(PathBuf::from)
            .map(|home| home.join("Library/Application Support/safecc/browser-snapshots"));
    }
    #[cfg(target_os = "windows")]
    {
        return std::env::var_os("APPDATA")
            .map(PathBuf::from)
            .map(|root| root.join("safecc/browser-snapshots"));
    }
    #[cfg(not(any(target_os = "macos", target_os = "windows")))]
    {
        std::env::var_os("XDG_DATA_HOME")
            .map(PathBuf::from)
            .or_else(|| {
                std::env::var_os("HOME").map(|home| PathBuf::from(home).join(".local/share"))
            })
            .map(|root| root.join("safecc/browser-snapshots"))
    }
}

fn atomic_write_json(path: &Path, value: &Value) -> Result<(), String> {
    let contents = serde_json::to_vec(&value).map_err(|_| "无法序列化浏览器偏好".to_string())?;
    let temporary = path.with_file_name(format!(".safecc-{}.tmp", std::process::id()));
    fs::write(&temporary, contents).map_err(|error| io_error("写入浏览器临时偏好", error))?;
    protect_file(&temporary).map_err(|error| io_error("保护浏览器临时偏好", error))?;

    #[cfg(target_os = "windows")]
    {
        let original = path.with_file_name(format!(".safecc-{}-original.tmp", std::process::id()));
        if original.exists() {
            let _ = fs::remove_file(&original);
        }
        if let Err(error) = fs::rename(path, &original) {
            let _ = fs::remove_file(&temporary);
            return Err(io_error("替换浏览器偏好", error));
        }
        if let Err(error) = fs::rename(&temporary, path) {
            let _ = fs::rename(&original, path);
            let _ = fs::remove_file(&temporary);
            return Err(io_error("替换浏览器偏好", error));
        }
        let _ = fs::remove_file(&original);
    }
    #[cfg(not(target_os = "windows"))]
    {
        if let Err(error) = fs::rename(&temporary, path) {
            let _ = fs::remove_file(&temporary);
            return Err(io_error("替换浏览器偏好", error));
        }
    }
    Ok(())
}

fn protect_file(path: &Path) -> io::Result<()> {
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        fs::set_permissions(path, fs::Permissions::from_mode(0o600))?;
    }
    let _ = path;
    Ok(())
}

fn io_error(action: &str, error: io::Error) -> String {
    format!("{action}失败：{}", error.kind())
}

#[cfg(test)]
mod tests {
    use super::{classify, configure_preferences, language_for, BrowserKind};
    use serde_json::json;

    #[test]
    fn browser_classification_is_allowlisted() {
        assert_eq!(classify(Some("com.google.chrome")), BrowserKind::Chrome);
        assert_eq!(classify(Some("MSEdgeHTM")), BrowserKind::Edge);
        assert_eq!(classify(Some("something-else")), BrowserKind::Unknown);
    }

    #[test]
    fn target_region_controls_language_order() {
        assert_eq!(language_for("日本 · 东京"), "ja-JP,ja,en-US,en");
        assert_eq!(language_for("美国 · 洛杉矶"), "en-US,en");
        assert_eq!(language_for("台湾 · 台北"), "zh-TW,zh,en-US,en");
        assert_eq!(language_for("加拿大 · 温哥华"), "en-CA,en-US,en");
    }

    #[test]
    fn browser_preferences_disable_doh_without_touching_account_data() {
        let mut preferences = json!({
            "profile": { "name": "fixture" },
            "cookies": { "last_session": "untouched" }
        });

        configure_preferences(&mut preferences, "en-US,en");

        assert_eq!(preferences["intl"]["accept_languages"], "en-US,en");
        assert_eq!(
            preferences["profile"]["default_content_setting_values"]["geolocation"],
            2
        );
        assert_eq!(preferences["dns_over_https"]["mode"], "off");
        assert_eq!(preferences["profile"]["name"], "fixture");
        assert_eq!(preferences["cookies"]["last_session"], "untouched");
    }
}
