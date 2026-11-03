use serde::{Deserialize, Serialize};
use std::fs;
use std::io;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::time::{SystemTime, UNIX_EPOCH};

const INDEX_VERSION: u32 = 1;
const MAX_PROFILES: usize = 8;
const MAX_NAME_CHARS: usize = 32;

#[derive(Debug, Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct EnvironmentProfile {
    pub id: String,
    pub name: String,
    pub target_region: String,
    pub created_at_unix_ms: u64,
    pub claude_config_dir: String,
    pub browser_profile_dir: String,
    pub launcher_path: String,
    pub browser: String,
    pub browser_label: String,
    pub browser_available: bool,
    pub preview_only: bool,
    pub isolation: Vec<String>,
    pub warnings: Vec<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
struct StoredProfile {
    id: String,
    name: String,
    target_region: String,
    created_at_unix_ms: u64,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct ProfileIndex {
    version: u32,
    profiles: Vec<StoredProfile>,
}

impl Default for ProfileIndex {
    fn default() -> Self {
        Self {
            version: INDEX_VERSION,
            profiles: Vec::new(),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum SupportedBrowser {
    Chrome,
    Edge,
}

impl SupportedBrowser {
    fn id(self) -> &'static str {
        match self {
            Self::Chrome => "chrome",
            Self::Edge => "edge",
        }
    }

    fn label(self) -> &'static str {
        match self {
            Self::Chrome => "Google Chrome",
            Self::Edge => "Microsoft Edge",
        }
    }
}

pub fn list() -> Result<Vec<EnvironmentProfile>, String> {
    let root = profile_root_or_default();
    let index = read_index(&root)?;
    index
        .profiles
        .iter()
        .map(|profile| profile_summary(profile, &root))
        .collect::<Result<Vec<_>, _>>()
}

pub fn create(
    name: String,
    target_region: String,
    confirmed: bool,
) -> Result<EnvironmentProfile, String> {
    create_at(&profile_root_or_default(), name, target_region, confirmed)
}

fn create_at(
    root: &Path,
    name: String,
    target_region: String,
    confirmed: bool,
) -> Result<EnvironmentProfile, String> {
    if !confirmed {
        return Err("创建账号档案前必须确认隔离范围".to_string());
    }
    let name = normalize_name(&name)?;
    if !crate::region::is_supported_region(&target_region) {
        return Err("目标地区不在支持列表中".to_string());
    }

    ensure_private_directory(root)?;
    let mut index = read_index(root)?;
    if index.profiles.len() >= MAX_PROFILES {
        return Err(format!("最多保存 {MAX_PROFILES} 个本地账号档案"));
    }
    if index
        .profiles
        .iter()
        .any(|profile| profile.name.eq_ignore_ascii_case(&name))
    {
        return Err("已存在同名账号档案，请换一个名称".to_string());
    }

    let id = new_profile_id()?;
    let stored = StoredProfile {
        id: id.clone(),
        name,
        target_region,
        created_at_unix_ms: id_timestamp(&id)?,
    };
    let directory = profile_directory_at(root, &id)?;
    fs::create_dir_all(directory.join("claude-config"))
        .map_err(|error| io_error("创建 Claude CLI 配置目录", error))?;
    fs::create_dir_all(directory.join("browser-profile"))
        .map_err(|error| io_error("创建隔离浏览器目录", error))?;
    protect_directory(&directory.join("claude-config"))?;
    protect_directory(&directory.join("browser-profile"))?;
    write_launcher(&stored, &directory)?;

    index.profiles.push(stored);
    if let Err(error) = write_index(root, &index) {
        let _ = fs::remove_dir_all(&directory);
        return Err(error);
    }

    profile_summary(
        index.profiles.last().expect("profile was just inserted"),
        root,
    )
}

pub fn open(profile_id: String, action: String, confirmed: bool) -> Result<(), String> {
    if !confirmed {
        return Err("启动账号档案前必须确认".to_string());
    }
    let root = profile_root_or_default();
    let index = read_index(&root)?;
    let profile = index
        .profiles
        .iter()
        .find(|profile| profile.id == profile_id)
        .ok_or_else(|| "未找到该本地账号档案".to_string())?;
    let directory = profile_directory_at(&root, &profile.id)?;
    reject_symlink(&directory)?;
    if !directory.is_dir() {
        return Err("账号档案目录已不存在".to_string());
    }

    match action.as_str() {
        "browser" => open_isolated_browser(profile, &directory),
        "claude" => open_claude_launcher(&directory),
        "folder" => open_folder(&directory),
        _ => Err("不支持的档案操作".to_string()),
    }
}

fn profile_summary(profile: &StoredProfile, root: &Path) -> Result<EnvironmentProfile, String> {
    if !is_safe_profile_id(&profile.id) {
        return Err("档案索引包含不安全的标识，已停止读取".to_string());
    }
    let directory = profile_directory_at(root, &profile.id)?;
    let browser = supported_browser();
    Ok(EnvironmentProfile {
        id: profile.id.clone(),
        name: profile.name.clone(),
        target_region: profile.target_region.clone(),
        created_at_unix_ms: profile.created_at_unix_ms,
        claude_config_dir: directory.join("claude-config").display().to_string(),
        browser_profile_dir: directory.join("browser-profile").display().to_string(),
        launcher_path: launcher_path(&directory).display().to_string(),
        browser: browser
            .map_or("unavailable", SupportedBrowser::id)
            .to_string(),
        browser_label: browser
            .map_or("未安装 Chrome / Edge", SupportedBrowser::label)
            .to_string(),
        browser_available: browser.is_some(),
        preview_only: false,
        isolation: isolation_notes(),
        warnings: shared_environment_warnings(),
    })
}

fn isolation_notes() -> Vec<String> {
    vec![
        "Claude CLI 使用独立 CLAUDE_CONFIG_DIR，登录凭证与配置不共享".to_string(),
        "浏览器使用独立 --user-data-dir，Cookie、站点存储和登录会话不共享".to_string(),
        "每个档案目录只保存启动元数据，不读取或复制已有凭证内容".to_string(),
    ]
}

fn shared_environment_warnings() -> Vec<String> {
    vec![
        "系统时区是全机设置；不同地区的档案需要切换 Clash 出口并重新配置地区".to_string(),
        "独立档案不等于独立设备：硬件指纹、浏览器引擎特征和公网 IP 仍可能相关".to_string(),
        "同一时刻 Clash Verge Rev 只能使用一个活动出口；账号隔离不会创建多个物理网络".to_string(),
    ]
}

fn normalize_name(value: &str) -> Result<String, String> {
    let name = value.trim();
    let count = name.chars().count();
    if count == 0 || count > MAX_NAME_CHARS {
        return Err(format!("档案名称需要 1 至 {MAX_NAME_CHARS} 个字符"));
    }
    if name.chars().any(|character| {
        character.is_control()
            || matches!(
                character,
                '/' | '\\' | ':' | '*' | '?' | '"' | '<' | '>' | '|'
            )
    }) {
        return Err("档案名称不能包含路径分隔符或系统保留字符".to_string());
    }
    Ok(name.to_string())
}

fn new_profile_id() -> Result<String, String> {
    let timestamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_err(|_| "系统时间不可用，无法创建档案".to_string())?
        .as_millis();
    Ok(format!("profile-{timestamp}-{}", std::process::id()))
}

fn id_timestamp(id: &str) -> Result<u64, String> {
    id.split('-')
        .nth(1)
        .and_then(|value| value.parse::<u64>().ok())
        .ok_or_else(|| "无法解析档案时间".to_string())
}

fn is_safe_profile_id(value: &str) -> bool {
    !value.is_empty()
        && value.len() <= 64
        && value.starts_with("profile-")
        && value
            .bytes()
            .all(|byte| byte.is_ascii_alphanumeric() || byte == b'-')
}

fn read_index(root: &Path) -> Result<ProfileIndex, String> {
    let path = root.join("index.json");
    if !path.is_file() {
        return Ok(ProfileIndex::default());
    }
    let contents = fs::read(&path).map_err(|error| io_error("读取账号档案索引", error))?;
    let index: ProfileIndex = serde_json::from_slice(&contents)
        .map_err(|_| "账号档案索引格式无效，已停止操作".to_string())?;
    if index.version != INDEX_VERSION {
        return Err("账号档案索引版本不受支持".to_string());
    }
    if index
        .profiles
        .iter()
        .any(|profile| !is_safe_profile_id(&profile.id))
    {
        return Err("账号档案索引包含不安全的标识，已停止操作".to_string());
    }
    Ok(index)
}

fn write_index(root: &Path, index: &ProfileIndex) -> Result<(), String> {
    ensure_private_directory(root)?;
    let path = root.join("index.json");
    let temporary = root.join(format!(".index-{}.tmp", std::process::id()));
    let contents =
        serde_json::to_vec_pretty(index).map_err(|_| "无法序列化账号档案索引".to_string())?;
    fs::write(&temporary, contents).map_err(|error| io_error("写入账号档案索引", error))?;
    protect_file(&temporary)?;
    replace_file(&temporary, &path)
}

fn replace_file(source: &Path, destination: &Path) -> Result<(), String> {
    #[cfg(target_os = "windows")]
    {
        let original =
            destination.with_file_name(format!(".index-{}-original.tmp", std::process::id()));
        if original.exists() {
            fs::remove_file(&original).map_err(|error| io_error("清理档案索引临时文件", error))?;
        }
        if destination.exists() {
            fs::rename(destination, &original)
                .map_err(|error| io_error("替换账号档案索引", error))?;
        }
        if let Err(error) = fs::rename(source, destination) {
            if original.exists() {
                let _ = fs::rename(&original, destination);
            }
            let _ = fs::remove_file(source);
            return Err(io_error("替换账号档案索引", error));
        }
        if original.exists() {
            let _ = fs::remove_file(&original);
        }
        return Ok(());
    }

    #[cfg(not(target_os = "windows"))]
    {
        if let Err(error) = fs::rename(source, destination) {
            let _ = fs::remove_file(source);
            return Err(io_error("替换账号档案索引", error));
        }
        Ok(())
    }
}

fn write_launcher(profile: &StoredProfile, directory: &Path) -> Result<(), String> {
    let config_dir = directory.join("claude-config");
    let path = launcher_path(directory);

    #[cfg(target_os = "macos")]
    let contents = format!(
        "#!/bin/zsh\n# safecc generated launcher. This file contains no account credentials.\nexport CLAUDE_CONFIG_DIR={}\nif ! command -v claude >/dev/null 2>&1; then\n  echo '未找到 Claude Code / Claude CLI。请先安装 claude 命令。'\n  read -k 1 '?按任意键关闭…'\n  exit 127\nfi\nexec claude \"$@\"\n",
        shell_quote(&config_dir.display().to_string())
    );

    #[cfg(target_os = "windows")]
    let contents = format!(
        "@echo off\r\nrem safecc generated launcher. This file contains no account credentials.\r\nsetlocal\r\nset \"CLAUDE_CONFIG_DIR={}\"\r\nwhere claude >nul 2>nul\r\nif errorlevel 1 (\r\n  echo 未找到 Claude Code / Claude CLI。请先安装 claude 命令。\r\n  pause\r\n  exit /b 127\r\n)\r\nclaude %*\r\nif errorlevel 1 pause\r\n",
        config_dir.display()
    );

    #[cfg(not(any(target_os = "macos", target_os = "windows")))]
    let contents = format!(
        "#!/bin/sh\nexport CLAUDE_CONFIG_DIR={}\nexec claude \"$@\"\n",
        shell_quote(&config_dir.display().to_string())
    );

    fs::write(&path, contents).map_err(|error| io_error("写入 CLI 启动脚本", error))?;
    protect_file(&path)?;
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        fs::set_permissions(&path, fs::Permissions::from_mode(0o700))
            .map_err(|error| io_error("设置 CLI 启动脚本权限", error))?;
    }
    let _ = profile;
    Ok(())
}

fn open_isolated_browser(profile: &StoredProfile, directory: &Path) -> Result<(), String> {
    let browser = supported_browser()
        .ok_or_else(|| "未找到 Google Chrome 或 Microsoft Edge，无法启动隔离浏览器".to_string())?;
    let browser_dir = directory.join("browser-profile");
    reject_symlink(&browser_dir)?;
    if !browser_dir.is_dir() {
        return Err("隔离浏览器档案目录不存在".to_string());
    }
    let language = browser_language(&profile.target_region);
    let language_primary = language.split(',').next().unwrap_or("en-US");
    let args = browser_args(&browser_dir, &language, language_primary);

    #[cfg(target_os = "macos")]
    let status = Command::new("/usr/bin/open")
        .args(["-n", "-a", browser_application_name(browser), "--args"])
        .args(&args)
        .status();

    #[cfg(target_os = "windows")]
    let status = browser_executable(browser)
        .ok_or_else(|| "未找到 Chrome / Edge 可执行文件".to_string())
        .and_then(|executable| {
            Command::new(executable)
                .args(&args)
                .spawn()
                .map(|_| ())
                .map_err(|error| io_error("启动隔离浏览器", error))
        });

    #[cfg(not(any(target_os = "macos", target_os = "windows")))]
    let status = Err("当前平台暂不支持隔离浏览器启动".to_string());

    #[cfg(target_os = "macos")]
    {
        match status {
            Ok(status) if status.success() => Ok(()),
            _ => Err("无法启动隔离浏览器".to_string()),
        }
    }
    #[cfg(not(target_os = "macos"))]
    {
        status
    }
}

fn browser_args(browser_dir: &Path, language: &str, primary_language: &str) -> Vec<String> {
    vec![
        format!("--user-data-dir={}", browser_dir.display()),
        format!("--lang={primary_language}"),
        format!("--accept-lang={language}"),
        "--no-first-run".to_string(),
        "--no-default-browser-check".to_string(),
        "https://claude.ai/login".to_string(),
    ]
}

fn browser_language(target_region: &str) -> String {
    let primary = crate::region::target_language(target_region);
    if primary == "en-US" {
        "en-US,en".to_string()
    } else if primary.starts_with("en-") {
        format!("{primary},en-US,en")
    } else {
        let base = primary.split('-').next().unwrap_or("en");
        format!("{primary},{base},en-US,en")
    }
}

fn open_claude_launcher(directory: &Path) -> Result<(), String> {
    let path = launcher_path(directory);
    reject_symlink(&path)?;
    if !path.is_file() {
        return Err("Claude CLI 启动脚本不存在".to_string());
    }

    #[cfg(target_os = "macos")]
    let status = Command::new("/usr/bin/open").arg(&path).status();

    #[cfg(target_os = "windows")]
    let status = Command::new("cmd")
        .args(["/C", "start", "", &path.display().to_string()])
        .status();

    #[cfg(not(any(target_os = "macos", target_os = "windows")))]
    let status = Command::new("/bin/sh").arg(&path).status();

    match status {
        Ok(status) if status.success() => Ok(()),
        _ => Err("无法打开 Claude CLI 启动脚本".to_string()),
    }
}

fn open_folder(directory: &Path) -> Result<(), String> {
    #[cfg(target_os = "macos")]
    let status = Command::new("/usr/bin/open").arg(directory).status();

    #[cfg(target_os = "windows")]
    let status = Command::new("explorer").arg(directory).status();

    #[cfg(not(any(target_os = "macos", target_os = "windows")))]
    let status = Command::new("xdg-open").arg(directory).status();

    match status {
        Ok(status) if status.success() => Ok(()),
        _ => Err("无法打开账号档案目录".to_string()),
    }
}

fn supported_browser() -> Option<SupportedBrowser> {
    [SupportedBrowser::Chrome, SupportedBrowser::Edge]
        .into_iter()
        .find(|browser| browser_is_available(*browser))
}

fn browser_is_available(browser: SupportedBrowser) -> bool {
    #[cfg(target_os = "macos")]
    {
        return Path::new(browser_application_path(browser)).is_dir();
    }
    #[cfg(target_os = "windows")]
    {
        return browser_executable(browser).is_some();
    }
    #[cfg(not(any(target_os = "macos", target_os = "windows")))]
    {
        let _ = browser;
        false
    }
}

#[cfg(target_os = "macos")]
fn browser_application_name(browser: SupportedBrowser) -> &'static str {
    match browser {
        SupportedBrowser::Chrome => "Google Chrome",
        SupportedBrowser::Edge => "Microsoft Edge",
    }
}

#[cfg(target_os = "macos")]
fn browser_application_path(browser: SupportedBrowser) -> &'static str {
    match browser {
        SupportedBrowser::Chrome => "/Applications/Google Chrome.app",
        SupportedBrowser::Edge => "/Applications/Microsoft Edge.app",
    }
}

#[cfg(target_os = "windows")]
fn browser_executable(browser: SupportedBrowser) -> Option<PathBuf> {
    let local = std::env::var_os("LOCALAPPDATA").map(PathBuf::from);
    let program_files = std::env::var_os("PROGRAMFILES").map(PathBuf::from);
    let program_files_x86 = std::env::var_os("PROGRAMFILES(X86)").map(PathBuf::from);
    let relative = match browser {
        SupportedBrowser::Chrome => [
            "Google/Chrome/Application/chrome.exe",
            "Google/Chrome Beta/Application/chrome.exe",
        ],
        SupportedBrowser::Edge => ["Microsoft/Edge/Application/msedge.exe", ""],
    };
    [local, program_files, program_files_x86]
        .into_iter()
        .flatten()
        .flat_map(|root| {
            relative
                .iter()
                .filter(|path| !path.is_empty())
                .map(move |path| root.join(path))
        })
        .find(|path| path.is_file())
}

fn launcher_path(directory: &Path) -> PathBuf {
    #[cfg(target_os = "windows")]
    {
        directory.join("start-claude.cmd")
    }
    #[cfg(not(target_os = "windows"))]
    {
        directory.join("start-claude.command")
    }
}

fn profile_root() -> Option<PathBuf> {
    #[cfg(target_os = "windows")]
    {
        return std::env::var_os("USERPROFILE")
            .map(PathBuf::from)
            .map(|home| home.join(".safecc/profiles"));
    }
    #[cfg(not(target_os = "windows"))]
    {
        std::env::var_os("HOME")
            .map(PathBuf::from)
            .map(|home| home.join(".safecc/profiles"))
    }
}

fn profile_root_or_default() -> PathBuf {
    profile_root().unwrap_or_else(|| {
        #[cfg(target_os = "windows")]
        {
            PathBuf::from(".safecc/profiles")
        }
        #[cfg(not(target_os = "windows"))]
        {
            PathBuf::from(".safecc/profiles")
        }
    })
}

fn profile_directory_at(root: &Path, id: &str) -> Result<PathBuf, String> {
    if !is_safe_profile_id(id) {
        return Err("账号档案标识无效".to_string());
    }
    Ok(root.join(id))
}

fn ensure_private_directory(path: &Path) -> Result<(), String> {
    fs::create_dir_all(path).map_err(|error| io_error("创建账号档案目录", error))?;
    protect_directory(path)
}

fn protect_directory(path: &Path) -> Result<(), String> {
    reject_symlink(path)?;
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        fs::set_permissions(path, fs::Permissions::from_mode(0o700))
            .map_err(|error| io_error("保护账号档案目录", error))?;
    }
    let _ = path;
    Ok(())
}

fn protect_file(path: &Path) -> Result<(), String> {
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        fs::set_permissions(path, fs::Permissions::from_mode(0o600))
            .map_err(|error| io_error("保护账号档案文件", error))?;
    }
    let _ = path;
    Ok(())
}

fn reject_symlink(path: &Path) -> Result<(), String> {
    if fs::symlink_metadata(path).is_ok_and(|metadata| metadata.file_type().is_symlink()) {
        return Err("账号档案路径不能是符号链接".to_string());
    }
    Ok(())
}

#[cfg(unix)]
fn shell_quote(value: &str) -> String {
    format!("'{}'", value.replace('\'', "'\\''"))
}

fn io_error(action: &str, error: io::Error) -> String {
    format!("{action}失败：{}", error.kind())
}

#[cfg(test)]
mod tests {
    use super::{
        browser_args, browser_language, create_at, is_safe_profile_id, normalize_name,
        profile_directory_at,
    };
    use std::fs;
    use std::path::Path;
    use std::time::{SystemTime, UNIX_EPOCH};

    #[test]
    fn profile_names_allow_human_labels_and_reject_paths() {
        assert_eq!(normalize_name(" 账号 A ").unwrap(), "账号 A");
        assert!(normalize_name("../other").is_err());
        assert!(normalize_name("a/b").is_err());
        assert!(normalize_name("").is_err());
        assert!(normalize_name(&"x".repeat(33)).is_err());
    }

    #[test]
    fn profile_ids_cannot_escape_the_profile_root() {
        assert!(is_safe_profile_id("profile-1760000000000-42"));
        assert!(!is_safe_profile_id("../profile-1760000000000-42"));
        assert!(!is_safe_profile_id("profile-../../etc"));
        assert!(!is_safe_profile_id("profile-1/2"));
    }

    #[test]
    fn browser_command_uses_isolated_profile_and_target_language() {
        let args = browser_args(
            Path::new("/tmp/safecc/browser-profile"),
            "ja-JP,ja,en-US,en",
            "ja-JP",
        );
        assert!(args.contains(&"--user-data-dir=/tmp/safecc/browser-profile".to_string()));
        assert!(args.contains(&"--lang=ja-JP".to_string()));
        assert!(args.contains(&"--accept-lang=ja-JP,ja,en-US,en".to_string()));
        assert!(args.contains(&"https://claude.ai/login".to_string()));
    }

    #[test]
    fn region_controls_browser_accept_language() {
        assert_eq!(browser_language("日本 · 东京"), "ja-JP,ja,en-US,en");
        assert_eq!(browser_language("美国 · 洛杉矶"), "en-US,en");
    }

    #[test]
    fn create_makes_empty_isolated_directories_and_launcher() {
        let fixture = temp_fixture_root();
        fs::create_dir_all(&fixture).unwrap();
        let profile = create_at(
            &fixture,
            "账号 A".to_string(),
            "日本 · 东京".to_string(),
            true,
        )
        .unwrap();
        let directory = profile_directory_at(&fixture, &profile.id).unwrap();
        assert!(directory
            .join("claude-config")
            .read_dir()
            .unwrap()
            .next()
            .is_none());
        assert!(directory
            .join("browser-profile")
            .read_dir()
            .unwrap()
            .next()
            .is_none());
        let launcher = directory.join("start-claude.command");
        let launcher_contents = fs::read_to_string(launcher).unwrap();
        assert!(launcher_contents.contains("CLAUDE_CONFIG_DIR="));
        assert!(!launcher_contents.contains("/.claude\""));
        assert_eq!(profile.target_region, "日本 · 东京");
        let _ = fs::remove_dir_all(&fixture);
    }

    #[test]
    fn duplicate_names_are_rejected_without_overwriting() {
        let fixture = temp_fixture_root();
        fs::create_dir_all(&fixture).unwrap();
        create_at(
            &fixture,
            "账号 A".to_string(),
            "美国 · 洛杉矶".to_string(),
            true,
        )
        .unwrap();
        assert!(create_at(
            &fixture,
            "账号 A".to_string(),
            "日本 · 东京".to_string(),
            true,
        )
        .is_err());
        let _ = fs::remove_dir_all(&fixture);
    }

    fn temp_fixture_root() -> std::path::PathBuf {
        let suffix = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        std::env::temp_dir().join(format!(
            "safecc-profile-test-{}-{suffix}",
            std::process::id()
        ))
    }
}
