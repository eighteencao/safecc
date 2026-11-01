use serde::Serialize;
use std::fs;
use std::path::Path;
use std::time::Duration;

use crate::source;

#[derive(Debug, Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct VpsCheckResult {
    pub status: String,
    pub host: String,
    pub user: String,
    pub port: u16,
    pub operating_system: String,
    pub remote_changes: bool,
    pub warnings: Vec<String>,
    pub actions: Vec<String>,
}

pub fn check(
    target_input: String,
    key_path: String,
    host_key_id: String,
    confirmed: bool,
) -> Result<VpsCheckResult, String> {
    if !confirmed {
        return Err("执行 VPS 只读检查前必须明确确认，不会安装或修改远端文件".to_string());
    }
    let target = source::parse_vps_target(target_input.trim())?;
    source::validate_key_path(Some(key_path.trim()))?;
    let host_key_path = source::host_key_path(&host_key_id)?;
    validate_private_key(Path::new(key_path.trim()))?;
    validate_private_file(&host_key_path)?;

    let ssh_program = if cfg!(target_os = "windows") {
        "ssh.exe"
    } else {
        "/usr/bin/ssh"
    };
    let port = target.port.to_string();
    let user = target.user.as_deref().unwrap_or("root");
    let known_hosts = host_key_path.to_string_lossy().to_string();
    let global_known_hosts = if cfg!(target_os = "windows") {
        "NUL"
    } else {
        "/dev/null"
    };
    let user_known_hosts_arg = format!("UserKnownHostsFile={known_hosts}");
    let global_known_hosts_arg = format!("GlobalKnownHostsFile={global_known_hosts}");
    let args = [
        "-F",
        "none",
        "-T",
        "-p",
        port.as_str(),
        "-l",
        user,
        "-i",
        key_path.trim(),
        "-o",
        "BatchMode=yes",
        "-o",
        "IdentitiesOnly=yes",
        "-o",
        "IdentityAgent=none",
        "-o",
        "PreferredAuthentications=publickey",
        "-o",
        "PasswordAuthentication=no",
        "-o",
        "KbdInteractiveAuthentication=no",
        "-o",
        "StrictHostKeyChecking=yes",
        "-o",
        user_known_hosts_arg.as_str(),
        "-o",
        global_known_hosts_arg.as_str(),
        "-o",
        "ConnectTimeout=15",
        "-o",
        "ServerAliveInterval=15",
        "-o",
        "ServerAliveCountMax=2",
        &target.host,
        "sh",
        "-s",
    ];
    let output = source::run_bounded(
        ssh_program,
        &args,
        Some(READ_ONLY_CHECK.as_bytes()),
        Duration::from_secs(45),
    )?;
    if !output.status.success() {
        return Err(
            "VPS SSH 只读检查失败；未执行安装或远端写入，请检查密钥、指纹、用户和系统版本"
                .to_string(),
        );
    }
    let stdout = String::from_utf8_lossy(&output.stdout);
    let operating_system = stdout
        .lines()
        .find_map(|line| line.strip_prefix("SAFECC_OS="))
        .filter(|value| matches!(*value, "debian-12" | "debian-13"))
        .ok_or_else(|| "远端未报告受支持的 Debian 12/13，已停止后续流程".to_string())?
        .to_string();

    Ok(VpsCheckResult {
        status: "verified".to_string(),
        host: target.host,
        user: user.to_string(),
        port: target.port,
        operating_system,
        remote_changes: false,
        warnings: vec![
            "本次只读检查没有安装软件、修改 SSH、防火墙或远端文件".to_string(),
            "真正部署可能修改系统包、服务和防火墙，必须重新查看变更清单并单独确认".to_string(),
        ],
        actions: vec![
            "已验证 SSH 公钥登录、root 身份、Debian 版本、curl 和 Python 3".to_string(),
            "下一步才会展示可审批的远端部署计划".to_string(),
        ],
    })
}

const READ_ONLY_CHECK: &str = r#"set -eu
test "$(id -u)" = 0
test -r /etc/os-release
. /etc/os-release
case "${ID:-}:${VERSION_ID:-}" in
  debian:12|debian:13) ;;
  *) exit 42 ;;
esac
command -v curl >/dev/null 2>&1
command -v python3 >/dev/null 2>&1
printf 'SAFECC_VPS_CHECK=verified\nSAFECC_OS=%s-%s\n' "$ID" "$VERSION_ID"
"#;

fn validate_private_key(path: &Path) -> Result<(), String> {
    validate_private_file(path)?;
    let size = fs::metadata(path)
        .map_err(|_| "无法读取 SSH 私钥文件属性".to_string())?
        .len();
    if size == 0 || size > 64 * 1024 {
        return Err("SSH 私钥文件大小不在安全范围内".to_string());
    }
    Ok(())
}

fn validate_private_file(path: &Path) -> Result<(), String> {
    let metadata = fs::symlink_metadata(path).map_err(|_| "无法读取受保护文件属性".to_string())?;
    if !metadata.is_file() || metadata.file_type().is_symlink() {
        return Err("受保护文件必须是普通文件，不能是符号链接".to_string());
    }
    #[cfg(unix)]
    {
        use std::os::unix::fs::{MetadataExt, PermissionsExt};
        if metadata.uid() != libc_getuid() || metadata.permissions().mode() & 0o077 != 0 {
            return Err("SSH 私钥或主机记录必须由当前用户拥有且权限不超过 0600".to_string());
        }
    }
    Ok(())
}

#[cfg(unix)]
fn libc_getuid() -> u32 {
    unsafe extern "C" {
        fn getuid() -> u32;
    }
    // SAFETY: getuid is a side-effect-free libc call available on Unix.
    unsafe { getuid() }
}

#[cfg(test)]
mod tests {
    use super::READ_ONLY_CHECK;

    #[test]
    fn read_only_probe_has_no_mutating_commands() {
        assert!(READ_ONLY_CHECK.contains("command -v curl"));
        assert!(!READ_ONLY_CHECK.contains("apt install"));
        assert!(!READ_ONLY_CHECK.contains("systemctl enable"));
        assert!(!READ_ONLY_CHECK.contains("ufw"));
    }
}
