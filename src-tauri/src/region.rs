use serde::{Deserialize, Serialize};
use std::fs;
use std::io;
use std::path::PathBuf;
use std::time::{SystemTime, UNIX_EPOCH};

#[allow(dead_code)]
#[derive(Debug, Clone, Copy)]
struct RegionTarget {
    label: &'static str,
    timezone: &'static str,
    locale: &'static str,
    language: &'static str,
    windows_timezone: &'static str,
}

const REGION_TARGETS: &[RegionTarget] = &[
    RegionTarget {
        label: "美国 · 洛杉矶",
        timezone: "America/Los_Angeles",
        locale: "en_US",
        language: "en-US",
        windows_timezone: "Pacific Standard Time",
    },
    RegionTarget {
        label: "美国 · 旧金山",
        timezone: "America/Los_Angeles",
        locale: "en_US",
        language: "en-US",
        windows_timezone: "Pacific Standard Time",
    },
    RegionTarget {
        label: "美国 · 西雅图",
        timezone: "America/Los_Angeles",
        locale: "en_US",
        language: "en-US",
        windows_timezone: "Pacific Standard Time",
    },
    RegionTarget {
        label: "美国 · 纽约",
        timezone: "America/New_York",
        locale: "en_US",
        language: "en-US",
        windows_timezone: "Eastern Standard Time",
    },
    RegionTarget {
        label: "美国 · 芝加哥",
        timezone: "America/Chicago",
        locale: "en_US",
        language: "en-US",
        windows_timezone: "Central Standard Time",
    },
    RegionTarget {
        label: "加拿大 · 多伦多",
        timezone: "America/Toronto",
        locale: "en_CA",
        language: "en-CA",
        windows_timezone: "Eastern Standard Time",
    },
    RegionTarget {
        label: "加拿大 · 温哥华",
        timezone: "America/Vancouver",
        locale: "en_CA",
        language: "en-CA",
        windows_timezone: "Pacific Standard Time",
    },
    RegionTarget {
        label: "墨西哥 · 墨西哥城",
        timezone: "America/Mexico_City",
        locale: "es_MX",
        language: "es-MX",
        windows_timezone: "Central Standard Time (Mexico)",
    },
    RegionTarget {
        label: "巴西 · 圣保罗",
        timezone: "America/Sao_Paulo",
        locale: "pt_BR",
        language: "pt-BR",
        windows_timezone: "E. South America Standard Time",
    },
    RegionTarget {
        label: "阿根廷 · 布宜诺斯艾利斯",
        timezone: "America/Argentina/Buenos_Aires",
        locale: "es_AR",
        language: "es-AR",
        windows_timezone: "Argentina Standard Time",
    },
    RegionTarget {
        label: "英国 · 伦敦",
        timezone: "Europe/London",
        locale: "en_GB",
        language: "en-GB",
        windows_timezone: "GMT Standard Time",
    },
    RegionTarget {
        label: "爱尔兰 · 都柏林",
        timezone: "Europe/Dublin",
        locale: "en_IE",
        language: "en-IE",
        windows_timezone: "GMT Standard Time",
    },
    RegionTarget {
        label: "德国 · 法兰克福",
        timezone: "Europe/Berlin",
        locale: "de_DE",
        language: "de-DE",
        windows_timezone: "W. Europe Standard Time",
    },
    RegionTarget {
        label: "法国 · 巴黎",
        timezone: "Europe/Paris",
        locale: "fr_FR",
        language: "fr-FR",
        windows_timezone: "Romance Standard Time",
    },
    RegionTarget {
        label: "荷兰 · 阿姆斯特丹",
        timezone: "Europe/Amsterdam",
        locale: "nl_NL",
        language: "nl-NL",
        windows_timezone: "W. Europe Standard Time",
    },
    RegionTarget {
        label: "瑞士 · 苏黎世",
        timezone: "Europe/Zurich",
        locale: "de_CH",
        language: "de-CH",
        windows_timezone: "W. Europe Standard Time",
    },
    RegionTarget {
        label: "西班牙 · 马德里",
        timezone: "Europe/Madrid",
        locale: "es_ES",
        language: "es-ES",
        windows_timezone: "Romance Standard Time",
    },
    RegionTarget {
        label: "意大利 · 米兰",
        timezone: "Europe/Rome",
        locale: "it_IT",
        language: "it-IT",
        windows_timezone: "W. Europe Standard Time",
    },
    RegionTarget {
        label: "葡萄牙 · 里斯本",
        timezone: "Europe/Lisbon",
        locale: "pt_PT",
        language: "pt-PT",
        windows_timezone: "GMT Standard Time",
    },
    RegionTarget {
        label: "波兰 · 华沙",
        timezone: "Europe/Warsaw",
        locale: "pl_PL",
        language: "pl-PL",
        windows_timezone: "Central European Standard Time",
    },
    RegionTarget {
        label: "瑞典 · 斯德哥尔摩",
        timezone: "Europe/Stockholm",
        locale: "sv_SE",
        language: "sv-SE",
        windows_timezone: "W. Europe Standard Time",
    },
    RegionTarget {
        label: "日本 · 东京",
        timezone: "Asia/Tokyo",
        locale: "ja_JP",
        language: "ja-JP",
        windows_timezone: "Tokyo Standard Time",
    },
    RegionTarget {
        label: "韩国 · 首尔",
        timezone: "Asia/Seoul",
        locale: "ko_KR",
        language: "ko-KR",
        windows_timezone: "Korea Standard Time",
    },
    RegionTarget {
        label: "新加坡",
        timezone: "Asia/Singapore",
        locale: "en_SG",
        language: "en-SG",
        windows_timezone: "Singapore Standard Time",
    },
    RegionTarget {
        label: "台湾 · 台北",
        timezone: "Asia/Taipei",
        locale: "zh_TW",
        language: "zh-TW",
        windows_timezone: "Taipei Standard Time",
    },
    RegionTarget {
        label: "澳大利亚 · 悉尼",
        timezone: "Australia/Sydney",
        locale: "en_AU",
        language: "en-AU",
        windows_timezone: "AUS Eastern Standard Time",
    },
    RegionTarget {
        label: "澳大利亚 · 墨尔本",
        timezone: "Australia/Melbourne",
        locale: "en_AU",
        language: "en-AU",
        windows_timezone: "AUS Eastern Standard Time",
    },
    RegionTarget {
        label: "新西兰 · 奥克兰",
        timezone: "Pacific/Auckland",
        locale: "en_NZ",
        language: "en-NZ",
        windows_timezone: "New Zealand Standard Time",
    },
    RegionTarget {
        label: "印度 · 孟买",
        timezone: "Asia/Kolkata",
        locale: "en_IN",
        language: "en-IN",
        windows_timezone: "India Standard Time",
    },
    RegionTarget {
        label: "马来西亚 · 吉隆坡",
        timezone: "Asia/Kuala_Lumpur",
        locale: "ms_MY",
        language: "ms-MY",
        windows_timezone: "Singapore Standard Time",
    },
    RegionTarget {
        label: "泰国 · 曼谷",
        timezone: "Asia/Bangkok",
        locale: "th_TH",
        language: "th-TH",
        windows_timezone: "SE Asia Standard Time",
    },
    RegionTarget {
        label: "越南 · 胡志明市",
        timezone: "Asia/Ho_Chi_Minh",
        locale: "vi_VN",
        language: "vi-VN",
        windows_timezone: "SE Asia Standard Time",
    },
    RegionTarget {
        label: "菲律宾 · 马尼拉",
        timezone: "Asia/Manila",
        locale: "en_PH",
        language: "en-PH",
        windows_timezone: "Singapore Standard Time",
    },
    RegionTarget {
        label: "印度尼西亚 · 雅加达",
        timezone: "Asia/Jakarta",
        locale: "id_ID",
        language: "id-ID",
        windows_timezone: "SE Asia Standard Time",
    },
    RegionTarget {
        label: "以色列 · 特拉维夫",
        timezone: "Asia/Jerusalem",
        locale: "he_IL",
        language: "he-IL",
        windows_timezone: "Israel Standard Time",
    },
    RegionTarget {
        label: "阿联酋 · 迪拜",
        timezone: "Asia/Dubai",
        locale: "ar_AE",
        language: "ar-AE",
        windows_timezone: "Arabian Standard Time",
    },
    RegionTarget {
        label: "南非 · 约翰内斯堡",
        timezone: "Africa/Johannesburg",
        locale: "en_ZA",
        language: "en-ZA",
        windows_timezone: "South Africa Standard Time",
    },
];

#[derive(Debug, Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct RegionChangePlan {
    pub platform: String,
    pub target_region: String,
    pub timezone: String,
    pub locale: String,
    pub language: String,
    pub actions: Vec<String>,
    pub permission_prompts: Vec<String>,
    pub restart_requirements: Vec<String>,
    pub manual_steps: Vec<String>,
}

#[derive(Debug, Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct RegionApplyResult {
    pub target_region: String,
    pub status: String,
    pub snapshot_id: String,
    pub changed: Vec<String>,
    pub permission_requested: bool,
    pub restart_requirements: Vec<String>,
    pub warnings: Vec<String>,
}

#[derive(Debug, Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct RegionRestoreResult {
    pub snapshot_id: String,
    pub pre_restore_snapshot_id: String,
    pub status: String,
    pub restored: Vec<String>,
    pub permission_requested: bool,
    pub warnings: Vec<String>,
}

pub fn change_plan(target_region: String) -> RegionChangePlan {
    let target = target_for(&target_region);
    RegionChangePlan {
        platform: std::env::consts::OS.to_string(),
        target_region,
        timezone: target.timezone.to_string(),
        locale: target.locale.to_string(),
        language: target.language.to_string(),
        actions: vec![
            "保存当前时区、语言和地区设置快照".to_string(),
            "设置系统时区与地区格式".to_string(),
            "设置当前用户的首选语言".to_string(),
            "重新读取结果并报告需要注销或重启的项目".to_string(),
        ],
        permission_prompts: vec![
            "修改系统时区时请求管理员权限".to_string(),
            "Windows 可能弹出 UAC；macOS 可能要求输入管理员密码".to_string(),
        ],
        restart_requirements: vec![
            "语言设置可能要注销当前用户后才完全生效".to_string(),
            "Clash、浏览器和 Claude Desktop 需要重新启动后再验收".to_string(),
        ],
        manual_steps: vec![
            "系统定位权限仍在系统隐私设置中单独确认".to_string(),
            "浏览器语言、定位和安全 DNS 在浏览器步骤中复验".to_string(),
        ],
    }
}

pub fn apply(target_region: String, confirmed: bool) -> Result<RegionApplyResult, String> {
    if !confirmed {
        return Err("修改系统环境前必须明确确认影响范围".to_string());
    }
    let target = target_for(&target_region);
    let snapshot_id = capture_snapshot()?;

    #[cfg(target_os = "macos")]
    {
        return apply_macos(&target_region, &target, snapshot_id);
    }

    #[cfg(target_os = "windows")]
    {
        return apply_windows(&target_region, &target, snapshot_id);
    }

    #[cfg(not(any(target_os = "macos", target_os = "windows")))]
    {
        let _ = (target, snapshot_id);
        Err("当前平台暂不支持自动修改系统环境".to_string())
    }
}

pub fn restore(snapshot_id: String, confirmed: bool) -> Result<RegionRestoreResult, String> {
    if !confirmed {
        return Err("恢复系统地区前必须明确确认影响范围".to_string());
    }
    if !is_safe_snapshot_id(&snapshot_id) {
        return Err("地区快照编号格式无效".to_string());
    }
    let root = snapshot_root().ok_or_else(|| "无法确定 safecc 地区快照目录".to_string())?;
    let path = root.join(format!("{snapshot_id}.json"));
    let contents = fs::read(&path).map_err(|_| "找不到所选地区快照".to_string())?;
    let snapshot: RegionSnapshot = serde_json::from_slice(&contents)
        .map_err(|_| "地区快照格式无效，已停止恢复".to_string())?;
    let pre_restore_snapshot_id = capture_snapshot()?;

    #[cfg(target_os = "macos")]
    restore_macos(&snapshot)?;

    #[cfg(target_os = "windows")]
    restore_windows(&snapshot)?;

    #[cfg(not(any(target_os = "macos", target_os = "windows")))]
    {
        let _ = snapshot;
        return Err("当前平台暂不支持自动恢复系统地区".to_string());
    }

    Ok(RegionRestoreResult {
        snapshot_id,
        pre_restore_snapshot_id,
        status: "restored".to_string(),
        restored: vec![
            "系统时区".to_string(),
            "地区格式".to_string(),
            "首选语言".to_string(),
        ],
        permission_requested: true,
        warnings: vec!["部分应用需要重启或注销后才会完全恢复".to_string()],
    })
}

fn target_for(target_region: &str) -> RegionTarget {
    REGION_TARGETS
        .iter()
        .find(|target| target.label == target_region)
        .copied()
        .unwrap_or(REGION_TARGETS[0])
}

pub(crate) fn is_supported_region(target_region: &str) -> bool {
    REGION_TARGETS
        .iter()
        .any(|target| target.label == target_region)
}

pub(crate) fn target_language(target_region: &str) -> &'static str {
    target_for(target_region).language
}

#[derive(Debug, Deserialize, Serialize, Clone)]
#[serde(rename_all = "camelCase")]
struct RegionSnapshot {
    snapshot_id: String,
    timezone: Option<String>,
    locale: Option<String>,
    language: Option<String>,
}

fn is_safe_snapshot_id(value: &str) -> bool {
    !value.is_empty() && value.len() <= 80 && value.bytes().all(|byte| byte.is_ascii_digit())
}

fn capture_snapshot() -> Result<String, String> {
    let snapshot_id = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_err(|_| "系统时间不可用，无法保存地区快照".to_string())?
        .as_millis()
        .to_string();
    let snapshot = RegionSnapshot {
        snapshot_id: snapshot_id.clone(),
        timezone: current_timezone(),
        locale: current_locale(),
        language: current_language(),
    };
    let root = snapshot_root().ok_or_else(|| "无法确定 safecc 地区快照目录".to_string())?;
    fs::create_dir_all(&root).map_err(|error| io_error("创建地区快照目录", error))?;
    let path = root.join(format!("{snapshot_id}.json"));
    let contents =
        serde_json::to_vec_pretty(&snapshot).map_err(|_| "无法序列化地区快照".to_string())?;
    fs::write(&path, contents).map_err(|error| io_error("保存地区快照", error))?;
    protect_file(&path).map_err(|error| io_error("保护地区快照", error))?;
    Ok(snapshot_id)
}

fn snapshot_root() -> Option<PathBuf> {
    #[cfg(target_os = "macos")]
    {
        return std::env::var_os("HOME")
            .map(PathBuf::from)
            .map(|home| home.join("Library/Application Support/safecc/region-snapshots"));
    }

    #[cfg(target_os = "windows")]
    {
        return std::env::var_os("APPDATA")
            .map(PathBuf::from)
            .map(|root| root.join("safecc/region-snapshots"));
    }

    #[cfg(not(any(target_os = "macos", target_os = "windows")))]
    {
        std::env::var_os("XDG_DATA_HOME")
            .map(PathBuf::from)
            .or_else(|| {
                std::env::var_os("HOME").map(|home| PathBuf::from(home).join(".local/share"))
            })
            .map(|root| root.join("safecc/region-snapshots"))
    }
}

fn current_timezone() -> Option<String> {
    #[cfg(target_os = "macos")]
    {
        if let Ok(path) = std::fs::read_link("/etc/localtime") {
            let text = path.to_string_lossy();
            if let Some((_, timezone)) = text.split_once("zoneinfo/") {
                return Some(timezone.to_string());
            }
        }
        return None;
    }

    #[cfg(target_os = "windows")]
    {
        return read_command("tzutil", &["/g"]);
    }

    #[cfg(not(any(target_os = "macos", target_os = "windows")))]
    {
        None
    }
}

fn current_locale() -> Option<String> {
    #[cfg(target_os = "macos")]
    {
        return read_command("/usr/bin/defaults", &["read", "-g", "AppleLocale"]);
    }

    #[cfg(target_os = "windows")]
    {
        return read_command(
            "powershell",
            &["-NoProfile", "-Command", "(Get-Culture).Name"],
        );
    }

    #[cfg(not(any(target_os = "macos", target_os = "windows")))]
    {
        None
    }
}

fn current_language() -> Option<String> {
    #[cfg(target_os = "macos")]
    {
        return read_command("/usr/bin/defaults", &["read", "-g", "AppleLanguages"]).and_then(
            |value| {
                value
                    .lines()
                    .find(|line| line.contains('"'))
                    .map(str::trim)
                    .map(|line| line.trim_matches(',').trim_matches('"').to_string())
            },
        );
    }

    #[cfg(target_os = "windows")]
    {
        return read_command(
            "powershell",
            &["-NoProfile", "-Command", "(Get-Culture).Name"],
        );
    }

    #[cfg(not(any(target_os = "macos", target_os = "windows")))]
    {
        None
    }
}

fn read_command(program: &str, args: &[&str]) -> Option<String> {
    let output = std::process::Command::new(program)
        .args(args)
        .output()
        .ok()?;
    if !output.status.success() {
        return None;
    }
    let value = String::from_utf8(output.stdout).ok()?.trim().to_string();
    (!value.is_empty()).then_some(value)
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

#[cfg(any(target_os = "windows", test))]
fn windows_timezone_script(timezone: &str) -> String {
    format!(
        "$ErrorActionPreference='Stop'; $process = Start-Process -FilePath 'tzutil.exe' -ArgumentList '/s','{}' -Verb RunAs -Wait -PassThru; if ($process.ExitCode -ne 0) {{ exit $process.ExitCode }}",
        timezone
    )
}

#[cfg(any(target_os = "windows", test))]
fn windows_culture_script(language: &str) -> String {
    format!(
        "$ErrorActionPreference='Stop'; Set-Culture -CultureInfo '{}'",
        language
    )
}

#[cfg(target_os = "macos")]
fn restore_macos(snapshot: &RegionSnapshot) -> Result<(), String> {
    if let Some(timezone) = snapshot.timezone.as_deref() {
        run_macos_timezone_command(timezone)?;
    }
    if let Some(locale) = snapshot.locale.as_deref() {
        run_command(
            "/usr/bin/defaults",
            &["write", "-g", "AppleLocale", locale],
            "恢复 macOS 地区格式",
        )?;
    }
    if let Some(language) = snapshot.language.as_deref() {
        run_command(
            "/usr/bin/defaults",
            &["write", "-g", "AppleLanguages", "-array", language],
            "恢复 macOS 首选语言",
        )?;
    }
    Ok(())
}

#[cfg(target_os = "windows")]
fn restore_windows(snapshot: &RegionSnapshot) -> Result<(), String> {
    if let Some(timezone) = snapshot.timezone.as_deref() {
        let script = windows_timezone_script(timezone);
        run_command(
            "powershell",
            &["-NoProfile", "-Command", &script],
            "恢复 Windows 系统时区",
        )?;
    }
    if let Some(locale) = snapshot.locale.as_deref() {
        let script = windows_culture_script(locale);
        run_command(
            "powershell",
            &["-NoProfile", "-Command", &script],
            "恢复 Windows 地区格式",
        )?;
    }
    Ok(())
}

#[cfg(target_os = "macos")]
fn apply_macos(
    target_region: &str,
    target: &RegionTarget,
    snapshot_id: String,
) -> Result<RegionApplyResult, String> {
    run_macos_timezone_command(target.timezone)?;
    run_command(
        "/usr/bin/defaults",
        &["write", "-g", "AppleLocale", target.locale],
        "写入 macOS 地区格式",
    )?;
    run_command(
        "/usr/bin/defaults",
        &["write", "-g", "AppleLanguages", "-array", target.language],
        "写入 macOS 首选语言",
    )?;

    Ok(RegionApplyResult {
        target_region: target_region.to_string(),
        status: "applied".to_string(),
        snapshot_id,
        changed: vec![
            format!("系统时区 → {}", target.timezone),
            format!("地区格式 → {}", target.locale),
            format!("首选语言 → {}", target.language),
        ],
        permission_requested: true,
        restart_requirements: vec!["部分应用需要重启；语言完全生效可能需要注销当前用户".to_string()],
        warnings: vec![
            format!(
                "{} 的定位权限未自动修改，请在系统隐私设置中确认",
                target.label
            ),
            "Clash、浏览器和 Claude Desktop 需要重启后再验收".to_string(),
        ],
    })
}

#[cfg(target_os = "windows")]
fn apply_windows(
    target_region: &str,
    target: &RegionTarget,
    snapshot_id: String,
) -> Result<RegionApplyResult, String> {
    let script = windows_timezone_script(target.windows_timezone);
    run_command(
        "powershell",
        &["-NoProfile", "-Command", &script],
        "设置 Windows 系统时区",
    )?;
    let culture_script = windows_culture_script(target.language);
    run_command(
        "powershell",
        &["-NoProfile", "-Command", &culture_script],
        "设置 Windows 地区格式",
    )?;

    Ok(RegionApplyResult {
        target_region: target_region.to_string(),
        status: "applied".to_string(),
        snapshot_id,
        changed: vec![
            format!("系统时区 → {}", target.windows_timezone),
            format!("地区格式 → {}", target.language),
        ],
        permission_requested: true,
        restart_requirements: vec!["部分应用需要重启；语言完全生效可能需要注销当前用户".to_string()],
        warnings: vec![
            format!(
                "{} 的定位权限未自动修改，请在 Windows 位置设置中确认",
                target.label
            ),
            "Clash、浏览器和 Claude Desktop 需要重启后再验收".to_string(),
        ],
    })
}

#[cfg(target_os = "macos")]
fn run_macos_timezone_command(timezone: &str) -> Result<(), String> {
    let script = format!(
        "do shell script \"/usr/sbin/systemsetup -settimezone {}\" with administrator privileges",
        timezone
    );
    run_command(
        "/usr/bin/osascript",
        &["-e", &script],
        "设置 macOS 系统时区",
    )
}

fn run_command(program: &str, args: &[&str], action: &str) -> Result<(), String> {
    let output = std::process::Command::new(program)
        .args(args)
        .output()
        .map_err(|_| format!("{action}失败：无法启动系统命令"))?;
    if output.status.success() {
        return Ok(());
    }
    Err(format!("{action}失败：系统权限未授予或命令返回错误"))
}

#[cfg(test)]
mod tests {
    use super::{
        change_plan, is_safe_snapshot_id, target_for, windows_culture_script,
        windows_timezone_script,
    };

    #[test]
    fn region_plan_maps_target_values() {
        let plan = change_plan("日本 · 东京".to_string());

        assert_eq!(plan.timezone, "Asia/Tokyo");
        assert_eq!(plan.locale, "ja_JP");
        assert!(!plan.permission_prompts.is_empty());
        let taipei = change_plan("台湾 · 台北".to_string());
        assert_eq!(taipei.timezone, "Asia/Taipei");
        assert_eq!(taipei.language, "zh-TW");
    }

    #[test]
    fn unknown_region_uses_safe_default() {
        let target = target_for("unknown");

        assert_eq!(target.timezone, "America/Los_Angeles");
    }

    #[test]
    fn snapshot_id_is_path_safe() {
        assert!(is_safe_snapshot_id("1760000000123"));
        assert!(!is_safe_snapshot_id("../other"));
    }

    #[test]
    fn windows_scripts_request_uac_and_propagate_failures() {
        let timezone = windows_timezone_script("Asia/Tokyo");
        assert!(timezone.contains("-Verb RunAs"));
        assert!(timezone.contains("-PassThru"));
        assert!(timezone.contains("$process.ExitCode"));
        assert!(timezone.contains("$ErrorActionPreference='Stop'"));

        let culture = windows_culture_script("ja-JP");
        assert!(culture.contains("Set-Culture -CultureInfo 'ja-JP'"));
        assert!(culture.contains("$ErrorActionPreference='Stop'"));
    }
}
