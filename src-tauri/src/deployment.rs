use serde::{Deserialize, Serialize};
use std::fs;
use std::path::{Path, PathBuf};
use std::time::{Duration, SystemTime, UNIX_EPOCH};

use tauri::{AppHandle, Manager};

use crate::source;

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct VpsDeployInput {
    pub target_input: String,
    pub pem_path: String,
    pub ppk_path: String,
    pub pub_path: String,
    pub proxy_host: String,
    pub proxy_port: u16,
    pub proxy_user: String,
    pub proxy_password: String,
    pub expected_ip: String,
    pub host_key_id: String,
    pub confirm: bool,
}

#[derive(Debug, Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct VpsDeployResult {
    pub status: String,
    pub workspace_id: String,
    pub subscription_snapshot_id: String,
    pub remote_changes_requested: bool,
    pub warnings: Vec<String>,
    pub actions: Vec<String>,
}

pub fn deploy(app: AppHandle, input: VpsDeployInput) -> Result<VpsDeployResult, String> {
    if !input.confirm {
        return Err("执行 VPS 远端部署前必须明确确认服务器变更范围".to_string());
    }
    let target = source::parse_vps_target(input.target_input.trim())?;
    if target.port != 22 {
        return Err("当前打包的 Debian 部署脚本只支持 SSH 22 端口".to_string());
    }
    validate_input(&input)?;
    let host_key = source::host_key_path(&input.host_key_id)?;
    let host_key_contents =
        fs::read(&host_key).map_err(|_| "无法读取已确认的 SSH 主机记录".to_string())?;
    let workspace_guard = WorkspaceGuard::new(create_workspace()?);
    let workspace = workspace_guard.path.clone();
    let known_hosts = workspace.join("known_hosts");
    write_private(&known_hosts, &host_key_contents)?;
    let config_path = workspace.join("connection.json");
    let script = resource_script(&app)?;
    let config = serde_json::json!({
        "vps": target.host,
        "pem": input.pem_path.trim(),
        "ppk": input.ppk_path.trim(),
        "pub": input.pub_path.trim(),
        "home_host": input.proxy_host.trim(),
        "home_port": input.proxy_port,
        "home_user": input.proxy_user,
        "home_pass": input.proxy_password,
        "expected_ip": input.expected_ip.trim(),
    });
    let config_json =
        serde_json::to_vec(&config).map_err(|_| "无法生成私有部署配置".to_string())?;
    run_skill(
        &script,
        "configure",
        &config_path,
        Some(&config_json),
        Duration::from_secs(20),
    )?;
    run_skill(
        &script,
        "check",
        &config_path,
        None,
        Duration::from_secs(120),
    )?;
    run_skill(
        &script,
        "deploy",
        &config_path,
        None,
        Duration::from_secs(1800),
    )?;
    run_skill(
        &script,
        "fetch",
        &config_path,
        None,
        Duration::from_secs(120),
    )?;

    let yaml = latest_subscription(&workspace)?;
    let subscription_snapshot_id = source::store_local_subscription(&yaml)?;
    Ok(VpsDeployResult {
        status: "deployed".to_string(),
        workspace_id: workspace_id(&workspace),
        subscription_snapshot_id,
        remote_changes_requested: true,
        warnings: vec![
            "远端脚本已完成自检并通过 SSH 新会话取回本地 Clash YAML；原始部署输出不会进入 safecc 日志".to_string(),
            "VPS 部署会修改系统包、Xray/Reality 服务、SSH 配置和防火墙；失败时可能留下部分远端变更".to_string(),
            "部署工作区退出时会清理连接配置、主机记录和临时 Clash YAML；已登记的本地 profile 仍由 Clash Verge Rev 管理".to_string(),
        ],
        actions: vec![
            "下一步将本地 YAML 登记为 Clash Verge Rev profile".to_string(),
            "打开 Clash Verge Rev 后仍需开启 TUN、关闭系统代理并进行真实验收".to_string(),
        ],
    })
}

struct WorkspaceGuard {
    path: PathBuf,
}

impl WorkspaceGuard {
    fn new(path: PathBuf) -> Self {
        Self { path }
    }
}

impl Drop for WorkspaceGuard {
    fn drop(&mut self) {
        // The workspace contains proxy credentials, private key paths and the
        // temporary YAML returned by the remote script. It is app-owned and is
        // always removed on both success and failure. The YAML is copied into
        // safecc's protected local subscription snapshot before this guard drops.
        let _ = fs::remove_dir_all(&self.path);
    }
}

fn validate_input(input: &VpsDeployInput) -> Result<(), String> {
    source::validate_key_path(Some(input.pem_path.trim()))?;
    validate_key_file(&input.ppk_path, ".ppk")?;
    validate_key_file(&input.pub_path, ".pub")?;
    validate_network_text(&input.proxy_host, "代理主机")?;
    validate_network_text(&input.proxy_user, "代理用户名")?;
    if input.proxy_password.is_empty()
        || input.proxy_password.len() > 4096
        || input.proxy_password.chars().any(char::is_control)
    {
        return Err("代理密码格式无效".to_string());
    }
    if input.proxy_port == 0 {
        return Err("代理端口格式无效".to_string());
    }
    let expected_ip = input.expected_ip.trim();
    if expected_ip.contains('%') || expected_ip.parse::<std::net::IpAddr>().is_err() {
        return Err("预期出口必须是固定的 IPv4/IPv6 地址".to_string());
    }
    Ok(())
}

fn validate_network_text(value: &str, label: &str) -> Result<(), String> {
    if value.trim().is_empty()
        || value.len() > 4096
        || value
            .chars()
            .any(|character| character.is_control() || matches!(character, '\'' | '"' | '\\'))
    {
        return Err(format!("{label}格式无效"));
    }
    Ok(())
}

fn validate_key_file(value: &str, suffix: &str) -> Result<(), String> {
    let path = Path::new(value.trim());
    if !path.is_absolute()
        || path.is_symlink()
        || !path.is_file()
        || path.extension().and_then(|extension| extension.to_str())
            != Some(suffix.trim_start_matches('.'))
    {
        return Err(format!(
            "SSH {suffix} 文件必须是现有的本机文件，且不能是符号链接"
        ));
    }
    let metadata = fs::metadata(path).map_err(|_| "无法读取 SSH 密钥文件属性".to_string())?;
    if metadata.len() == 0 || metadata.len() > 64 * 1024 {
        return Err("SSH 密钥文件大小不在安全范围内".to_string());
    }
    #[cfg(unix)]
    {
        use std::os::unix::fs::{MetadataExt, PermissionsExt};
        if metadata.uid() != current_uid() || metadata.permissions().mode() & 0o077 != 0 {
            return Err("SSH 私钥文件必须由当前用户拥有且权限不超过 0600".to_string());
        }
    }
    Ok(())
}

fn current_uid() -> u32 {
    #[cfg(unix)]
    {
        unsafe extern "C" {
            fn getuid() -> u32;
        }
        // SAFETY: getuid is a side-effect-free libc call available on Unix.
        return unsafe { getuid() };
    }
    #[cfg(not(unix))]
    {
        0
    }
}

fn resource_script(app: &AppHandle) -> Result<PathBuf, String> {
    let resource_dir = app
        .path()
        .resource_dir()
        .map_err(|_| "无法定位 safecc 部署资源目录".to_string())?;
    for relative in [
        "ccbaga-network/scripts/network.py",
        "deployment/ccbaga-network/scripts/network.py",
        "_up_/deployment/ccbaga-network/scripts/network.py",
    ] {
        let bundled = resource_dir.join(relative);
        if bundled.is_file() {
            return Ok(bundled);
        }
    }
    let development = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../deployment/ccbaga-network/scripts/network.py");
    development
        .is_file()
        .then_some(development)
        .ok_or_else(|| "safecc 部署资源未打包，无法启动 VPS 流程".to_string())
}

fn run_skill(
    script: &Path,
    action: &str,
    config_path: &Path,
    stdin: Option<&[u8]>,
    timeout: Duration,
) -> Result<(), String> {
    let python = if cfg!(target_os = "windows") {
        "python.exe"
    } else {
        "python3"
    };
    let script_text = script.to_string_lossy().to_string();
    let config_text = config_path.to_string_lossy().to_string();
    let args = [
        python,
        script_text.as_str(),
        action,
        "--config",
        config_text.as_str(),
    ];
    let output = source::run_bounded(python, &args[1..], stdin, timeout)?;
    if !output.status.success() {
        return Err(format!(
            "VPS {action} 阶段失败；原始输出已隐藏，请检查服务器状态后再决定是否重试"
        ));
    }
    Ok(())
}

fn create_workspace() -> Result<PathBuf, String> {
    let root = if cfg!(target_os = "windows") {
        std::env::var_os("APPDATA")
            .map(PathBuf::from)
            .map(|path| path.join("safecc/vps"))
    } else {
        std::env::var_os("HOME")
            .map(PathBuf::from)
            .map(|path| path.join("Library/Application Support/safecc/vps"))
    }
    .ok_or_else(|| "无法确定 safecc VPS 私密目录".to_string())?;
    fs::create_dir_all(&root).map_err(|_| "无法创建 safecc VPS 私密目录".to_string())?;
    protect_directory(&root)?;
    let id = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_err(|_| "系统时间不可用，无法创建 VPS workspace".to_string())?
        .as_nanos()
        .to_string();
    let workspace = root.join(&id);
    fs::create_dir(&workspace).map_err(|_| "无法创建 VPS workspace".to_string())?;
    protect_directory(&workspace)?;
    Ok(workspace)
}

fn workspace_id(path: &Path) -> String {
    path.file_name()
        .and_then(|value| value.to_str())
        .unwrap_or("unknown")
        .to_string()
}

fn latest_subscription(workspace: &Path) -> Result<Vec<u8>, String> {
    let mut candidates = fs::read_dir(workspace)
        .map_err(|_| "找不到 VPS 本地订阅目录".to_string())?
        .filter_map(Result::ok)
        .filter(|entry| {
            entry.file_name().to_string_lossy().starts_with("clash-")
                && entry.path().extension().and_then(|value| value.to_str()) == Some("yaml")
        })
        .collect::<Vec<_>>();
    candidates.sort_by_key(|entry| {
        entry
            .metadata()
            .and_then(|metadata| metadata.modified())
            .ok()
    });
    let path = candidates
        .last()
        .map(|entry| entry.path())
        .ok_or_else(|| "VPS 自检没有返回本地 Clash YAML".to_string())?;
    let contents = fs::read(&path).map_err(|_| "无法读取 VPS 本地 Clash YAML".to_string())?;
    if contents.is_empty() || contents.len() > 256 * 1024 {
        return Err("VPS 本地 Clash YAML 大小不在安全范围内".to_string());
    }
    Ok(contents)
}

fn write_private(path: &Path, contents: &[u8]) -> Result<(), String> {
    fs::write(path, contents).map_err(|_| "无法写入 safecc 私密部署文件".to_string())?;
    protect_file(path)
}

fn protect_directory(path: &Path) -> Result<(), String> {
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        fs::set_permissions(path, fs::Permissions::from_mode(0o700))
            .map_err(|_| "无法保护 safecc VPS 私密目录".to_string())?;
    }
    Ok(())
}

fn protect_file(path: &Path) -> Result<(), String> {
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        fs::set_permissions(path, fs::Permissions::from_mode(0o600))
            .map_err(|_| "无法保护 safecc VPS 私密文件".to_string())?;
    }
    Ok(())
}
