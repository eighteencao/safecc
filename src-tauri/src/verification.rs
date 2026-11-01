use serde::Serialize;
use std::fs;
use std::io;
use std::path::PathBuf;
use std::time::{SystemTime, UNIX_EPOCH};

const REQUIRED_CHECKS: [&str; 4] = ["claude", "dns", "webrtc", "locale"];
const MANAGED_CONFIG_VERSION: &str = "safecc-managed-v1";

#[derive(Debug, Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct VerificationCheckResult {
    pub check: String,
    pub status: String,
}

#[derive(Debug, Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct VerificationRecord {
    pub record_id: String,
    pub target_region: String,
    pub route_alias: String,
    pub clash_config_version: String,
    pub recorded_at_unix_ms: u128,
    pub checks: Vec<String>,
    pub results: Vec<VerificationCheckResult>,
    pub status: String,
}

pub fn record(
    target_region: String,
    route_alias: String,
    checks: Vec<String>,
    confirmed: bool,
) -> Result<VerificationRecord, String> {
    if !confirmed {
        return Err("保存验收记录前必须明确确认检测页结果".to_string());
    }
    if !is_supported_region(&target_region) {
        return Err("目标地区不受支持，已停止保存验收记录".to_string());
    }
    validate_route_alias(&route_alias)?;

    let normalized = normalize_checks(checks)?;
    let results = normalized
        .iter()
        .map(|check| VerificationCheckResult {
            check: check.clone(),
            status: "confirmed".to_string(),
        })
        .collect();
    let recorded_at_unix_ms = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_err(|_| "系统时间不可用，无法保存验收记录".to_string())?
        .as_millis();
    let record_id = recorded_at_unix_ms.to_string();
    let record = VerificationRecord {
        record_id: record_id.clone(),
        target_region,
        route_alias,
        clash_config_version: MANAGED_CONFIG_VERSION.to_string(),
        recorded_at_unix_ms,
        checks: normalized,
        results,
        status: "recorded".to_string(),
    };
    let root = verification_root().ok_or_else(|| "无法确定 safecc 验收记录目录".to_string())?;
    fs::create_dir_all(&root).map_err(|error| io_error("创建验收记录目录", error))?;
    protect_directory(&root).map_err(|error| io_error("保护验收记录目录", error))?;
    let path = root.join(format!("{record_id}.json"));
    let contents =
        serde_json::to_vec_pretty(&record).map_err(|_| "无法序列化验收记录".to_string())?;
    fs::write(&path, contents).map_err(|error| io_error("保存验收记录", error))?;
    protect_file(&path).map_err(|error| io_error("保护验收记录", error))?;
    Ok(record)
}

fn validate_route_alias(value: &str) -> Result<(), String> {
    if value.is_empty()
        || value.len() > 120
        || value.chars().any(char::is_control)
        || ["://", "@", "?", "#", "\\", "/"]
            .iter()
            .any(|marker| value.contains(marker))
    {
        return Err("线路别名格式无效，验收记录不会保存地址或凭据".to_string());
    }
    Ok(())
}

fn normalize_checks(mut checks: Vec<String>) -> Result<Vec<String>, String> {
    checks.sort();
    checks.dedup();
    let mut expected = REQUIRED_CHECKS.to_vec();
    expected.sort();
    if checks != expected {
        return Err("验收记录必须包含 Claude、DNS、WebRTC UDP、时区语言四项".to_string());
    }
    Ok(checks)
}

fn is_supported_region(value: &str) -> bool {
    matches!(
        value,
        "美国 · 洛杉矶" | "美国 · 纽约" | "日本 · 东京" | "新加坡" | "英国 · 伦敦"
    )
}

fn verification_root() -> Option<PathBuf> {
    #[cfg(target_os = "macos")]
    {
        return std::env::var_os("HOME")
            .map(PathBuf::from)
            .map(|home| home.join("Library/Application Support/safecc/verification"));
    }

    #[cfg(target_os = "windows")]
    {
        return std::env::var_os("APPDATA")
            .map(PathBuf::from)
            .map(|root| root.join("safecc/verification"));
    }

    #[cfg(not(any(target_os = "macos", target_os = "windows")))]
    {
        std::env::var_os("XDG_DATA_HOME")
            .map(PathBuf::from)
            .or_else(|| {
                std::env::var_os("HOME").map(|home| PathBuf::from(home).join(".local/share"))
            })
            .map(|root| root.join("safecc/verification"))
    }
}

fn protect_directory(path: &std::path::Path) -> io::Result<()> {
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        fs::set_permissions(path, fs::Permissions::from_mode(0o700))?;
    }
    let _ = path;
    Ok(())
}

fn protect_file(path: &std::path::Path) -> io::Result<()> {
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
    use super::{is_supported_region, normalize_checks, validate_route_alias, REQUIRED_CHECKS};

    #[test]
    fn only_supported_regions_are_recordable() {
        assert!(is_supported_region("日本 · 东京"));
        assert!(!is_supported_region("未知地区"));
    }

    #[test]
    fn required_checks_are_fixed_and_secret_free() {
        assert_eq!(REQUIRED_CHECKS, ["claude", "dns", "webrtc", "locale"]);
    }

    #[test]
    fn required_checks_accept_any_order_but_reject_missing_items() {
        let checks = normalize_checks(vec![
            "locale".to_string(),
            "webrtc".to_string(),
            "dns".to_string(),
            "claude".to_string(),
        ])
        .expect("required checks should be order independent");
        assert_eq!(checks, vec!["claude", "dns", "locale", "webrtc"]);
        assert!(normalize_checks(vec!["claude".to_string()]).is_err());
    }

    #[test]
    fn route_alias_rejects_urls_and_credentials() {
        assert!(validate_route_alias("机场订阅 · 日本 · 东京").is_ok());
        assert!(validate_route_alias("https://example.com/sub").is_err());
        assert!(validate_route_alias("user:password@example.com").is_err());
    }
}
