use serde::Serialize;
use std::fs;
use std::io::{self, Write};
use std::path::PathBuf;
use std::process::{Command, Stdio};
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};

const MAX_SUBSCRIPTION_BYTES: usize = 10 * 1024 * 1024;

#[derive(Debug, Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct SourcePreflightResult {
    pub status: String,
    pub source_kind: String,
    pub source_id: Option<String>,
    pub format: String,
    pub node_count: usize,
    pub bytes: usize,
    pub warnings: Vec<String>,
    pub actions: Vec<String>,
    pub fingerprint: Option<String>,
}

pub fn preflight_with_key(
    source_kind: String,
    input: String,
    key_path: Option<String>,
    confirmed: bool,
) -> Result<SourcePreflightResult, String> {
    if !confirmed {
        return Err("预检线路前必须明确确认仅在本机拉取和保存临时订阅快照".to_string());
    }
    let input = input.trim();
    if input.is_empty() {
        return Err("请先填写线路地址或 VPS 主机信息".to_string());
    }

    match source_kind.as_str() {
        "vps" => preflight_vps(input, key_path),
        "static" | "airport" => preflight_subscription(&source_kind, input),
        _ => Err("线路来源不受支持".to_string()),
    }
}

fn preflight_vps(input: &str, key_path: Option<String>) -> Result<SourcePreflightResult, String> {
    let target = parse_vps_target(input)?;
    validate_key_path(key_path.as_deref())?;
    let known_hosts = scan_host_key(&target)?;
    let fingerprint = fingerprint_for_host_key(&known_hosts)?;
    let source_id = store_host_key(&known_hosts)?;

    Ok(SourcePreflightResult {
        status: "manual".to_string(),
        source_kind: "vps".to_string(),
        source_id: Some(source_id.clone()),
        format: "ssh-host-key".to_string(),
        node_count: 0,
        bytes: known_hosts.len(),
        warnings: vec![
            "safecc 只读取了 SSH 主机公钥指纹，没有登录服务器或执行远端命令".to_string(),
            "请把指纹与 VPS 服务商控制台或可信主机记录比对；不一致时不要继续".to_string(),
            "真正部署前仍需单独确认 SSH 密钥、服务器系统和远端变更范围".to_string(),
        ],
        actions: vec![
            format!("已保存受保护的 SSH 主机记录：{source_id}"),
            if key_path.is_some() {
                "已发现本机 PEM 路径；后续仍需再次确认后才会读取密钥".to_string()
            } else {
                "后续部署阶段需要用户选择本机 PEM 私钥，并由原生层验证权限".to_string()
            },
            "下一阶段会在明确授权后执行远端只读检查，再展示部署变更清单".to_string(),
        ],
        fingerprint: Some(fingerprint),
    })
}

pub(crate) fn validate_key_path(value: Option<&str>) -> Result<(), String> {
    let Some(value) = value else {
        return Ok(());
    };
    if value.is_empty() || value.len() > 4096 || value.chars().any(char::is_control) {
        return Err("SSH 私钥路径格式无效".to_string());
    }
    let path = std::path::Path::new(value);
    if !path.is_absolute() || path.is_symlink() || !path.is_file() {
        return Err("SSH 私钥路径必须是现有的本机文件，且不能是符号链接".to_string());
    }
    Ok(())
}

#[allow(dead_code)]
#[derive(Debug)]
pub(crate) struct VpsTarget {
    pub(crate) user: Option<String>,
    pub(crate) host: String,
    pub(crate) port: u16,
}

pub(crate) fn parse_vps_target(input: &str) -> Result<VpsTarget, String> {
    if contains_forbidden_input(input) || input.len() > 256 {
        return Err("VPS 主机信息格式无效".to_string());
    }
    let (user, host_port) = match input.rsplit_once('@') {
        Some((user, host_port)) if !user.is_empty() => {
            if !user
                .bytes()
                .all(|byte| byte.is_ascii_alphanumeric() || matches!(byte, b'.' | b'_' | b'-'))
            {
                return Err("SSH 用户名格式无效".to_string());
            }
            (Some(user.to_string()), host_port)
        }
        Some(_) => return Err("SSH 用户名格式无效".to_string()),
        None => (None, input),
    };
    let (host, port) = if let Some(stripped) = host_port.strip_prefix('[') {
        let Some((host, rest)) = stripped.split_once(']') else {
            return Err("VPS IPv6 主机格式无效".to_string());
        };
        let port = rest
            .strip_prefix(':')
            .map(parse_port)
            .transpose()?
            .unwrap_or(22);
        (host.to_string(), port)
    } else if host_port.matches(':').count() == 1 {
        let (host, port) = host_port.split_once(':').expect("one colon");
        if port.chars().all(|character| character.is_ascii_digit()) {
            (host.to_string(), parse_port(port)?)
        } else {
            (host_port.to_string(), 22)
        }
    } else {
        (host_port.to_string(), 22)
    };
    if host.is_empty()
        || host.contains('/')
        || host.contains(' ')
        || host.contains('%')
        || !host
            .bytes()
            .all(|byte| byte.is_ascii_alphanumeric() || matches!(byte, b'.' | b'-' | b':'))
    {
        return Err("VPS 主机信息格式无效".to_string());
    }
    Ok(VpsTarget { user, host, port })
}

fn parse_port(value: &str) -> Result<u16, String> {
    value
        .parse::<u16>()
        .ok()
        .filter(|port| *port > 0)
        .ok_or_else(|| "SSH 端口格式无效".to_string())
}

fn scan_host_key(target: &VpsTarget) -> Result<Vec<u8>, String> {
    let program = if cfg!(target_os = "windows") {
        "ssh-keyscan.exe"
    } else {
        "/usr/bin/ssh-keyscan"
    };
    let port = target.port.to_string();
    let args = [
        "-T",
        "8",
        "-p",
        port.as_str(),
        "-t",
        "ed25519,ecdsa-sha2-nistp256,rsa-sha2-512",
        &target.host,
    ];
    let output = run_bounded(program, &args, None, Duration::from_secs(12))?;
    if output.stdout.is_empty() {
        return Err("无法读取 VPS SSH 主机指纹，请检查地址、端口或网络".to_string());
    }
    Ok(output.stdout)
}

fn fingerprint_for_host_key(known_hosts: &[u8]) -> Result<String, String> {
    let program = if cfg!(target_os = "windows") {
        "ssh-keygen.exe"
    } else {
        "/usr/bin/ssh-keygen"
    };
    let output = run_bounded(
        program,
        &["-lf", "-", "-E", "sha256"],
        Some(known_hosts),
        Duration::from_secs(5),
    )?;
    let text = String::from_utf8_lossy(&output.stdout);
    text.lines()
        .flat_map(str::split_whitespace)
        .find(|value| value.starts_with("SHA256:"))
        .map(str::to_string)
        .ok_or_else(|| "无法解析 VPS SSH 主机指纹".to_string())
}

pub(crate) fn host_key_path(snapshot_id: &str) -> Result<PathBuf, String> {
    if !is_safe_snapshot_id(snapshot_id) {
        return Err("SSH 主机记录编号格式无效".to_string());
    }
    let root = snapshot_root()
        .ok_or_else(|| "无法确定 safecc SSH 主机记录目录".to_string())?
        .join("ssh-hosts");
    let path = root.join(format!("{snapshot_id}.known_hosts"));
    if !path.is_file() || path.is_symlink() {
        return Err("找不到所选 SSH 主机记录，请重新读取指纹".to_string());
    }
    Ok(path)
}

pub(crate) fn run_bounded(
    program: &str,
    args: &[&str],
    stdin: Option<&[u8]>,
    timeout: Duration,
) -> Result<std::process::Output, String> {
    let mut child = Command::new(program)
        .args(args)
        .stdin(if stdin.is_some() {
            Stdio::piped()
        } else {
            Stdio::null()
        })
        .stdout(Stdio::piped())
        .stderr(Stdio::null())
        .spawn()
        .map_err(|_| "无法启动本机 SSH 工具，请确认 OpenSSH Client 已安装".to_string())?;
    if let Some(contents) = stdin {
        if let Some(mut pipe) = child.stdin.take() {
            pipe.write_all(contents)
                .map_err(|_| "无法读取 SSH 主机记录".to_string())?;
        }
    }
    let deadline = Instant::now() + timeout;
    loop {
        if child
            .try_wait()
            .map_err(|_| "无法读取 SSH 工具状态".to_string())?
            .is_some()
        {
            return child
                .wait_with_output()
                .map_err(|_| "无法读取 SSH 工具输出".to_string());
        }
        if Instant::now() >= deadline {
            let _ = child.kill();
            let _ = child.wait();
            return Err("SSH 主机指纹读取超时，已停止本机检查".to_string());
        }
        std::thread::sleep(Duration::from_millis(40));
    }
}

fn preflight_subscription(source_kind: &str, input: &str) -> Result<SourcePreflightResult, String> {
    validate_https_url(input)?;
    let body = fetch_subscription(input)?;
    if body.is_empty() {
        return Err("订阅返回为空，未保存任何内容".to_string());
    }
    let detected = detect_format(&body);
    if detected.format == "unknown" {
        return Err("订阅内容不是可识别的 Clash/Mihomo、节点列表或编码订阅".to_string());
    }
    let source_id = store_snapshot(&body)?;
    Ok(SourcePreflightResult {
        status: "valid".to_string(),
        source_kind: source_kind.to_string(),
        source_id: Some(source_id),
        format: detected.format,
        node_count: detected.node_count,
        bytes: body.len(),
        warnings: vec![
            "订阅原文仅保存在本机 safecc 目录，界面和日志不会显示 URL、UUID 或口令".to_string(),
            "safecc 不把订阅上传到第三方转换服务；更新失败时保留本地快照".to_string(),
        ],
        actions: vec![
            "下一步先备份 Clash Verge Rev 配置".to_string(),
            "托管扩展写入后仍需在 Clash Verge Rev 中激活并复核最终配置".to_string(),
        ],
        fingerprint: None,
    })
}

fn validate_https_url(input: &str) -> Result<(), String> {
    if contains_forbidden_input(input) || input.len() > 4096 || !input.starts_with("https://") {
        return Err("订阅地址必须是 HTTPS URL，且不能包含空格、引号或换行".to_string());
    }
    let authority = input[8..].split(['/', '?', '#']).next().unwrap_or_default();
    let host = authority
        .rsplit_once('@')
        .map(|(_, host)| host)
        .unwrap_or(authority);
    if host.is_empty() || host.starts_with(':') || host.ends_with(':') {
        return Err("订阅地址缺少有效主机".to_string());
    }
    Ok(())
}

fn contains_forbidden_input(value: &str) -> bool {
    value
        .chars()
        .any(|character| character.is_control() || matches!(character, '"' | '\'' | '\\'))
}

fn fetch_subscription(url: &str) -> Result<Vec<u8>, String> {
    let program = if cfg!(target_os = "windows") {
        "curl.exe"
    } else {
        "/usr/bin/curl"
    };
    let mut child = Command::new(program)
        .args(["--config", "-"])
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::null())
        .spawn()
        .map_err(|_| "无法启动本机 HTTPS 订阅预检".to_string())?;
    let config = format!(
        "url = \"{url}\"\nsilent\nshow-error\nfail\nlocation\nconnect-timeout = 8\nmax-time = 20\nmax-filesize = {MAX_SUBSCRIPTION_BYTES}\nuser-agent = \"safecc/0.1\"\n"
    );
    if let Some(stdin) = child.stdin.as_mut() {
        stdin
            .write_all(config.as_bytes())
            .map_err(|_| "无法提交订阅预检请求".to_string())?;
    }
    let output = child
        .wait_with_output()
        .map_err(|_| "订阅预检请求失败".to_string())?;
    if !output.status.success() {
        return Err("订阅预检请求失败，请检查地址、网络或供应商证书".to_string());
    }
    if output.stdout.len() > MAX_SUBSCRIPTION_BYTES {
        return Err("订阅内容超过 safecc 安全大小上限".to_string());
    }
    Ok(output.stdout)
}

struct DetectedFormat {
    format: String,
    node_count: usize,
}

fn detect_format(body: &[u8]) -> DetectedFormat {
    let Ok(text) = std::str::from_utf8(body) else {
        return DetectedFormat {
            format: "encoded-subscription".to_string(),
            node_count: 0,
        };
    };
    let lower = text.to_ascii_lowercase();
    if lower.contains("proxies:")
        || lower.contains("proxy-groups:")
        || lower.contains("mixed-port:")
    {
        let node_count = text
            .lines()
            .filter(|line| line.trim_start().starts_with("- name:"))
            .count();
        return DetectedFormat {
            format: "clash-yaml".to_string(),
            node_count,
        };
    }
    if (text.trim_start().starts_with('{') || text.trim_start().starts_with('['))
        && lower.contains("proxies")
    {
        return DetectedFormat {
            format: "clash-json".to_string(),
            node_count: 0,
        };
    }
    let node_count = text
        .lines()
        .filter(|line| {
            let value = line.trim();
            value.starts_with("ss://")
                || value.starts_with("vmess://")
                || value.starts_with("trojan://")
                || value.starts_with("hysteria")
                || value.starts_with("socks5://")
        })
        .count();
    if node_count > 0 {
        return DetectedFormat {
            format: "node-list".to_string(),
            node_count,
        };
    }
    if looks_like_base64(text) {
        return DetectedFormat {
            format: "encoded-subscription".to_string(),
            node_count: 0,
        };
    }
    DetectedFormat {
        format: "unknown".to_string(),
        node_count: 0,
    }
}

fn looks_like_base64(value: &str) -> bool {
    let compact: String = value
        .chars()
        .filter(|character| !character.is_whitespace())
        .collect();
    compact.len() >= 32
        && compact.len() % 4 == 0
        && compact
            .bytes()
            .all(|byte| byte.is_ascii_alphanumeric() || matches!(byte, b'+' | b'/' | b'='))
}

fn store_snapshot(contents: &[u8]) -> Result<String, String> {
    let snapshot_id = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_err(|_| "系统时间不可用，无法保存订阅快照".to_string())?
        .as_nanos()
        .to_string();
    let root = snapshot_root().ok_or_else(|| "无法确定 safecc 订阅快照目录".to_string())?;
    fs::create_dir_all(&root).map_err(|error| io_error("创建订阅快照目录", error))?;
    protect_directory(&root).map_err(|error| io_error("保护订阅快照目录", error))?;
    let path = root.join(format!("{snapshot_id}.bin"));
    let mut file = fs::OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(&path)
        .map_err(|error| io_error("保存订阅快照", error))?;
    file.write_all(contents)
        .map_err(|error| io_error("保存订阅快照", error))?;
    protect_file(&path).map_err(|error| io_error("保护订阅快照", error))?;
    Ok(snapshot_id)
}

pub(crate) fn store_local_subscription(contents: &[u8]) -> Result<String, String> {
    store_snapshot(contents)
}

fn store_host_key(contents: &[u8]) -> Result<String, String> {
    let snapshot_id = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_err(|_| "系统时间不可用，无法保存 SSH 主机记录".to_string())?
        .as_nanos()
        .to_string();
    let root = snapshot_root()
        .ok_or_else(|| "无法确定 safecc SSH 主机记录目录".to_string())?
        .join("ssh-hosts");
    fs::create_dir_all(&root).map_err(|error| io_error("创建 SSH 主机记录目录", error))?;
    protect_directory(&root).map_err(|error| io_error("保护 SSH 主机记录目录", error))?;
    let path = root.join(format!("{snapshot_id}.known_hosts"));
    let mut file = fs::OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(&path)
        .map_err(|error| io_error("保存 SSH 主机记录", error))?;
    file.write_all(contents)
        .map_err(|error| io_error("保存 SSH 主机记录", error))?;
    protect_file(&path).map_err(|error| io_error("保护 SSH 主机记录", error))?;
    Ok(snapshot_id)
}

pub(crate) fn load_snapshot(snapshot_id: &str) -> Result<Vec<u8>, String> {
    if !is_safe_snapshot_id(snapshot_id) {
        return Err("订阅快照编号格式无效".to_string());
    }
    let root = snapshot_root().ok_or_else(|| "无法确定 safecc 订阅快照目录".to_string())?;
    fs::read(root.join(format!("{snapshot_id}.bin")))
        .map_err(|_| "找不到所选订阅快照，请重新预检线路".to_string())
}

pub(crate) fn validate_subscription_url(input: &str) -> Result<(), String> {
    validate_https_url(input)
}

fn is_safe_snapshot_id(value: &str) -> bool {
    !value.is_empty() && value.len() <= 80 && value.bytes().all(|byte| byte.is_ascii_digit())
}

fn snapshot_root() -> Option<PathBuf> {
    #[cfg(target_os = "macos")]
    {
        return std::env::var_os("HOME")
            .map(PathBuf::from)
            .map(|home| home.join("Library/Application Support/safecc/subscriptions"));
    }
    #[cfg(target_os = "windows")]
    {
        return std::env::var_os("APPDATA")
            .map(PathBuf::from)
            .map(|root| root.join("safecc/subscriptions"));
    }
    #[cfg(not(any(target_os = "macos", target_os = "windows")))]
    {
        std::env::var_os("XDG_DATA_HOME")
            .map(PathBuf::from)
            .or_else(|| {
                std::env::var_os("HOME").map(|home| PathBuf::from(home).join(".local/share"))
            })
            .map(|root| root.join("safecc/subscriptions"))
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
    use super::{detect_format, parse_vps_target, preflight_with_key, validate_https_url};

    #[test]
    fn detects_clash_yaml_without_printing_content() {
        let result = detect_format(b"mixed-port: 7890\nproxies:\n  - name: node-a\n");
        assert_eq!(result.format, "clash-yaml");
        assert_eq!(result.node_count, 1);
    }

    #[test]
    fn rejects_insecure_or_injected_urls() {
        assert!(validate_https_url("http://example.com/sub").is_err());
        assert!(validate_https_url("https://example.com/\"bad").is_err());
    }

    #[test]
    fn preflight_requires_confirmation() {
        assert!(preflight_with_key(
            "airport".to_string(),
            "https://example.com/sub".to_string(),
            None,
            false
        )
        .is_err());
    }

    #[test]
    fn parses_safe_ssh_targets_without_shell_syntax() {
        let target = parse_vps_target("root@example.com:2222").expect("valid SSH target");
        assert_eq!(target.user.as_deref(), Some("root"));
        assert_eq!(target.host, "example.com");
        assert_eq!(target.port, 2222);
        assert!(parse_vps_target("root@example.com;whoami").is_err());
        assert!(parse_vps_target("root@[fe80::1%en0]:22").is_err());
    }
}
