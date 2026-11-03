use serde::Serialize;
use std::fs;
use std::io;
use std::path::Path;

use crate::source;

#[derive(Debug, Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct PlannedFile {
    pub label: String,
    pub relative_path: String,
    pub exists: bool,
    pub contains_secrets: bool,
}

#[derive(Debug, Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct ClashChangePlan {
    pub platform: String,
    pub data_dir_exists: bool,
    pub profile_files: Vec<PlannedFile>,
    pub managed_extension: String,
    pub target_region: String,
    pub source_kind: String,
    pub claude_rule_count: usize,
    pub dns_and_tun_fields: Vec<String>,
    pub warnings: Vec<String>,
    pub requires_confirmation: Vec<String>,
}

#[derive(Debug, Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct ClashApplyResult {
    pub target_region: String,
    pub source_kind: String,
    pub status: String,
    pub extension_relative_path: String,
    pub written_fields: Vec<String>,
    pub warnings: Vec<String>,
    pub requires_reload: bool,
}

#[derive(Debug, Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct ClashImportResult {
    pub status: String,
    pub profile_uid: String,
    pub profile_relative_path: String,
    pub source_kind: String,
    pub target_region: String,
    pub activated: bool,
    pub warnings: Vec<String>,
}

const CLAUDE_DOMAINS: [&str; 9] = [
    "anthropic.com",
    "claude.ai",
    "claude.com",
    "claude.site",
    "claudemcpclient.com",
    "claudemcpcontent.com",
    "claudeusercontent.com",
    "clau.de",
    "servd-anthropic-website.b-cdn.net",
];

pub fn change_plan(source_kind: String, target_region: String) -> ClashChangePlan {
    let data_dir = data_dir();
    let data_dir_exists = data_dir.as_ref().is_some_and(|path| path.exists());
    let profile_files = data_dir
        .as_ref()
        .map(|path| {
            [
                ("应用设置", "verge.yaml", true),
                ("核心配置", "config.yaml", true),
                ("订阅索引", "profiles.yaml", true),
                ("订阅与扩展目录", "profiles/", true),
            ]
            .into_iter()
            .map(|(label, relative_path, contains_secrets)| PlannedFile {
                label: label.to_string(),
                relative_path: relative_path.to_string(),
                exists: path.join(relative_path).exists(),
                contains_secrets,
            })
            .collect()
        })
        .unwrap_or_default();

    ClashChangePlan {
        platform: std::env::consts::OS.to_string(),
        data_dir_exists,
        profile_files,
        managed_extension: "Clash Verge Rev · Global Merge（profiles/Merge.yaml）".to_string(),
        target_region,
        source_kind,
        claude_rule_count: CLAUDE_DOMAINS.len(),
        dns_and_tun_fields: vec![
            "mode: rule".to_string(),
            "tun.enable".to_string(),
            "tun.auto-route".to_string(),
            "tun.auto-detect-interface".to_string(),
            "tun.dns-hijack".to_string(),
            "dns.enhanced-mode".to_string(),
            "dns.respect-rules".to_string(),
        ],
        warnings: vec![
            "未确认前不会读取或输出订阅 URL、节点密码、UUID 或日志；确认导入时只读取 profiles.yaml 结构字段".to_string(),
            "Clash Verge 应用设置可能覆盖部分扩展字段，必须读取最终生效态".to_string(),
            "Claude 规则默认指向 PROXY 策略组，必须在最终生效态确认该组存在".to_string(),
            "未确认前不会写入现有文件或启用 TUN；执行后仍需重新加载并复核".to_string(),
        ],
        requires_confirmation: vec![
            "创建带时间戳的本地备份".to_string(),
            "覆盖/替换 profiles/Merge.yaml（可由刚创建的快照恢复）".to_string(),
            "让 Clash Verge 重新生成并验证运行配置".to_string(),
        ],
    }
}

pub fn apply_extension(
    source_kind: String,
    target_region: String,
    confirmed: bool,
) -> Result<ClashApplyResult, String> {
    if !confirmed {
        return Err("写入 Clash 托管扩展前必须明确确认影响范围".to_string());
    }
    let source_label = match source_kind.as_str() {
        "vps" => "自建 VPS",
        "static" => "静态 IP 订阅",
        "airport" => "机场订阅",
        _ => "未识别来源",
    };
    let data_dir = data_dir().ok_or_else(|| "未找到 Clash Verge Rev 数据目录".to_string())?;
    if !data_dir.is_dir() {
        return Err("未发现 Clash Verge Rev 数据目录，无法写入托管扩展".to_string());
    }

    let profiles = data_dir.join("profiles");
    fs::create_dir_all(&profiles).map_err(|error| io_error("创建 Clash 扩展目录", error))?;
    let target = profiles.join("Merge.yaml");
    let temporary = profiles.join("Merge.safecc.tmp");
    if temporary.exists() {
        fs::remove_file(&temporary).map_err(|error| io_error("清理临时扩展文件", error))?;
    }
    fs::write(&temporary, managed_yaml())
        .map_err(|error| io_error("写入 Clash 托管扩展", error))?;
    protect_file(&temporary).map_err(|error| io_error("保护 Clash 托管扩展", error))?;

    #[cfg(target_os = "windows")]
    if target.exists() {
        fs::remove_file(&target).map_err(|error| io_error("替换 Clash 托管扩展", error))?;
    }
    if let Err(error) = fs::rename(&temporary, &target) {
        let _ = fs::remove_file(&temporary);
        return Err(io_error("替换 Clash 托管扩展", error));
    }
    protect_file(&target).map_err(|error| io_error("保护 Clash 托管扩展", error))?;

    Ok(ClashApplyResult {
        target_region,
        source_kind: source_label.to_string(),
        status: "written".to_string(),
        extension_relative_path: "profiles/Merge.yaml".to_string(),
        written_fields: vec![
            "mode: rule".to_string(),
            "profile.store-selected".to_string(),
            "tun.enable".to_string(),
            "tun.auto-route".to_string(),
            "tun.auto-detect-interface".to_string(),
            "tun.dns-hijack".to_string(),
            "dns.enable".to_string(),
            "dns.enhanced-mode".to_string(),
            "dns.respect-rules".to_string(),
            "Claude DOMAIN-SUFFIX rules".to_string(),
        ],
        warnings: vec![
            "Claude 规则使用 PROXY 策略组；请在 Clash Verge 最终生效态确认该组存在".to_string(),
            "写入后需要在 Clash Verge 中重新更新/激活配置，再进行 DNS 与 WebRTC 验收".to_string(),
        ],
        requires_reload: true,
    })
}

pub fn import_subscription(
    source_kind: String,
    target_region: String,
    snapshot_id: String,
    input: String,
    confirmed: bool,
) -> Result<ClashImportResult, String> {
    if !confirmed {
        return Err("导入订阅到 Clash Verge Rev 前必须明确确认影响范围".to_string());
    }
    if !matches!(source_kind.as_str(), "static" | "airport") {
        return Err("当前来源不是可自动导入的 HTTPS 订阅".to_string());
    }
    if !crate::region::is_supported_region(&target_region) {
        return Err("目标地区不受支持，已停止导入".to_string());
    }
    source::validate_subscription_url(input.trim())?;
    let body = source::load_snapshot(&snapshot_id)?;
    import_profile_body(
        source_kind,
        target_region,
        snapshot_id,
        body,
        Some(input.trim()),
        confirmed,
    )
}

pub fn import_local_subscription(
    target_region: String,
    snapshot_id: String,
    confirmed: bool,
) -> Result<ClashImportResult, String> {
    if !confirmed {
        return Err("导入 VPS 本地订阅到 Clash Verge Rev 前必须明确确认影响范围".to_string());
    }
    if !crate::region::is_supported_region(&target_region) {
        return Err("目标地区不受支持，已停止导入".to_string());
    }
    let body = source::load_snapshot(&snapshot_id)?;
    import_profile_body(
        "vps".to_string(),
        target_region,
        snapshot_id,
        body,
        None,
        confirmed,
    )
}

fn import_profile_body(
    source_kind: String,
    target_region: String,
    snapshot_id: String,
    body: Vec<u8>,
    url: Option<&str>,
    confirmed: bool,
) -> Result<ClashImportResult, String> {
    if !confirmed {
        return Err("导入订阅到 Clash Verge Rev 前必须明确确认影响范围".to_string());
    }
    if body.is_empty() || body.len() > 10 * 1024 * 1024 {
        return Err("订阅快照为空或超过安全大小上限".to_string());
    }
    let data_dir = data_dir().ok_or_else(|| "未找到 Clash Verge Rev 数据目录".to_string())?;
    if !data_dir.is_dir() {
        return Err("未发现 Clash Verge Rev 数据目录，无法导入订阅".to_string());
    }
    let profiles_dir = data_dir.join("profiles");
    fs::create_dir_all(&profiles_dir).map_err(|error| io_error("创建 Clash 订阅目录", error))?;
    let index_path = data_dir.join("profiles.yaml");
    let index = fs::read_to_string(&index_path)
        .map_err(|_| "未找到 Clash Verge Rev 订阅索引，无法安全导入".to_string())?;
    if !index.lines().any(|line| line.trim() == "items:") {
        return Err("Clash Verge Rev 订阅索引格式不受支持，已停止导入".to_string());
    }
    let profile_uid = format!("safecc-{}", snapshot_id);
    if index.lines().any(|line| {
        line.trim_start()
            .strip_prefix("- uid:")
            .map(|value| value.trim() == profile_uid)
            .unwrap_or(false)
    }) {
        return Err("该订阅快照已经导入过，避免重复创建配置".to_string());
    }
    let profile_file = format!("{profile_uid}.yaml");
    let profile_path = profiles_dir.join(&profile_file);
    let refs = extract_profile_refs(&index);
    let next_index = append_profile(
        &index,
        &profile_uid,
        &profile_file,
        url,
        &source_kind,
        &target_region,
        &refs,
    )?;
    let profile_temp = profiles_dir.join(format!(".{profile_file}.safecc.tmp"));
    let index_temp = data_dir.join(".profiles.yaml.safecc.tmp");
    remove_if_exists(&profile_temp)?;
    remove_if_exists(&index_temp)?;
    fs::write(&profile_temp, &body).map_err(|error| io_error("写入 Clash 订阅快照", error))?;
    protect_file(&profile_temp).map_err(|error| io_error("保护 Clash 订阅快照", error))?;
    fs::write(&index_temp, next_index.as_bytes())
        .map_err(|error| io_error("写入 Clash 订阅索引", error))?;
    protect_file(&index_temp).map_err(|error| io_error("保护 Clash 订阅索引", error))?;
    if let Err(error) = replace_file(&profile_temp, &profile_path) {
        let _ = remove_if_exists(&index_temp);
        return Err(error);
    }
    if let Err(error) = replace_file(&index_temp, &index_path) {
        let _ = remove_if_exists(&profile_path);
        let _ = remove_if_exists(&index_temp);
        return Err(error);
    }
    Ok(ClashImportResult {
        status: "imported".to_string(),
        profile_uid,
        profile_relative_path: format!("profiles/{profile_file}"),
        source_kind,
        target_region,
        activated: true,
        warnings: vec![
            if url.is_some() {
                "订阅已登记为 Clash Verge Rev 的 remote profile，并将其设为当前配置".to_string()
            } else {
                "VPS 自检取回的本地 YAML 已登记为 Clash Verge Rev profile，并将其设为当前配置"
                    .to_string()
            },
            "仍需在 Clash Verge Rev 中刷新并复核最终生效态".to_string(),
            "safecc 不读取、打印或上传订阅原文；更新失败时保留本机快照".to_string(),
        ],
    })
}

#[derive(Default)]
struct ProfileRefs {
    merge: Option<String>,
    script: Option<String>,
    rules: Option<String>,
    proxies: Option<String>,
    groups: Option<String>,
}

fn extract_profile_refs(index: &str) -> ProfileRefs {
    let mut refs = ProfileRefs::default();
    let mut uid: Option<String> = None;
    for line in index.lines() {
        if let Some(value) = line.trim_start().strip_prefix("- uid:") {
            let candidate = value.trim();
            uid = is_safe_profile_id(candidate).then(|| candidate.to_string());
        } else if let Some(value) = line.trim_start().strip_prefix("type:") {
            let Some(uid) = uid.as_deref() else {
                continue;
            };
            match value.trim() {
                "merge" => refs.merge = Some(uid.to_string()),
                "script" => refs.script = Some(uid.to_string()),
                "rules" => refs.rules = Some(uid.to_string()),
                "proxies" => refs.proxies = Some(uid.to_string()),
                "groups" => refs.groups = Some(uid.to_string()),
                _ => {}
            }
        }
    }
    refs
}

fn is_safe_profile_id(value: &str) -> bool {
    !value.is_empty()
        && value.len() <= 80
        && value
            .bytes()
            .all(|byte| byte.is_ascii_alphanumeric() || matches!(byte, b'-' | b'_'))
}

fn append_profile(
    index: &str,
    profile_uid: &str,
    profile_file: &str,
    url: Option<&str>,
    source_kind: &str,
    target_region: &str,
    refs: &ProfileRefs,
) -> Result<String, String> {
    let name = format!(
        "safecc · {} · {}",
        match source_kind {
            "airport" => "机场订阅",
            "vps" => "VPS 私有订阅",
            _ => "静态 IP 订阅",
        },
        target_region
    );
    let quoted_name = serde_json::to_string(&name).map_err(|_| "无法生成订阅名称".to_string())?;
    let quoted_url = url
        .map(serde_json::to_string)
        .transpose()
        .map_err(|_| "无法保护订阅地址".to_string())?;
    let mut output = String::with_capacity(index.len() + 640);
    let mut replaced_current = false;
    for line in index.lines() {
        if line.trim_start().starts_with("current:") && !replaced_current {
            output.push_str(&format!("current: {profile_uid}\n"));
            replaced_current = true;
        } else {
            output.push_str(line);
            output.push('\n');
        }
    }
    if !replaced_current {
        return Err("Clash Verge Rev 订阅索引缺少 current 字段，已停止导入".to_string());
    }
    output.push_str(&format!(
        "- uid: {profile_uid}\n  type: {}\n  name: {quoted_name}\n  file: {profile_file}\n{}  selected: []\n  option:\n    update_interval: 1440\n    allow_auto_update: true\n",
        if url.is_some() { "remote" } else { "local" },
        quoted_url
            .as_deref()
            .map(|value| format!("  url: {value}\n"))
            .unwrap_or_default()
    ));
    for (key, value) in [
        ("merge", refs.merge.as_deref()),
        ("script", refs.script.as_deref()),
        ("rules", refs.rules.as_deref()),
        ("proxies", refs.proxies.as_deref()),
        ("groups", refs.groups.as_deref()),
    ] {
        if let Some(value) = value {
            output.push_str(&format!("    {key}: {value}\n"));
        }
    }
    Ok(output)
}

fn remove_if_exists(path: &Path) -> Result<(), String> {
    if path.exists() {
        fs::remove_file(path).map_err(|error| io_error("清理临时导入文件", error))?;
    }
    Ok(())
}

fn replace_file(source: &Path, target: &Path) -> Result<(), String> {
    #[cfg(target_os = "windows")]
    if target.exists() {
        fs::remove_file(target).map_err(|error| io_error("替换 Clash 导入文件", error))?;
    }
    fs::rename(source, target).map_err(|error| io_error("替换 Clash 导入文件", error))
}

fn managed_yaml() -> &'static str {
    "mode: rule\n\nprofile:\n  store-selected: true\n\ntun:\n  enable: true\n  stack: system\n  auto-route: true\n  auto-detect-interface: true\n  dns-hijack:\n    - any:53\n    - tcp://any:53\n\nipv6: false\n\ndns:\n  enable: true\n  ipv6: false\n  enhanced-mode: fake-ip\n  fake-ip-range: 198.18.0.1/16\n  respect-rules: true\n  proxy-server-nameserver:\n    - https://1.1.1.1/dns-query\n  nameserver:\n    - https://1.1.1.1/dns-query\n    - https://8.8.8.8/dns-query\n\nrules:\n  - DOMAIN-SUFFIX,anthropic.com,PROXY\n  - DOMAIN-SUFFIX,claude.ai,PROXY\n  - DOMAIN-SUFFIX,claude.com,PROXY\n  - DOMAIN-SUFFIX,claude.site,PROXY\n  - DOMAIN-SUFFIX,claudemcpclient.com,PROXY\n  - DOMAIN-SUFFIX,claudemcpcontent.com,PROXY\n  - DOMAIN-SUFFIX,claudeusercontent.com,PROXY\n  - DOMAIN-SUFFIX,clau.de,PROXY\n  - DOMAIN-SUFFIX,servd-anthropic-website.b-cdn.net,PROXY\n"
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

pub(crate) fn data_dir() -> Option<std::path::PathBuf> {
    #[cfg(target_os = "macos")]
    {
        return std::env::var_os("HOME")
            .map(std::path::PathBuf::from)
            .map(|home| {
                home.join("Library/Application Support/io.github.clash-verge-rev.clash-verge-rev")
            });
    }

    #[cfg(target_os = "windows")]
    {
        return std::env::var_os("APPDATA")
            .map(std::path::PathBuf::from)
            .map(|app_data| app_data.join("io.github.clash-verge-rev.clash-verge-rev"));
    }

    #[cfg(not(any(target_os = "macos", target_os = "windows")))]
    {
        std::env::var_os("XDG_DATA_HOME")
            .map(std::path::PathBuf::from)
            .or_else(|| {
                std::env::var_os("HOME")
                    .map(|home| std::path::PathBuf::from(home).join(".local/share"))
            })
            .map(|root| root.join("io.github.clash-verge-rev.clash-verge-rev"))
    }
}

#[cfg(test)]
mod tests {
    use super::{append_profile, change_plan, extract_profile_refs, managed_yaml};

    #[test]
    fn plan_marks_profile_files_as_sensitive() {
        let plan = change_plan("机场订阅".to_string(), "美国 · 洛杉矶".to_string());

        assert_eq!(plan.claude_rule_count, 9);
        assert!(plan.profile_files.iter().all(|file| file.contains_secrets));
        assert!(plan
            .warnings
            .iter()
            .any(|warning| warning.contains("不会读取")));
        assert!(plan
            .requires_confirmation
            .iter()
            .any(|item| item.contains("备份")));
    }

    #[test]
    fn managed_yaml_does_not_contain_subscription_secrets() {
        let yaml = managed_yaml();

        assert!(yaml.contains("tun:"));
        assert!(yaml.contains("mode: rule"));
        assert!(yaml.contains("dns-hijack:"));
        assert!(yaml.contains("respect-rules: true"));
        assert!(yaml.contains("claude.ai"));
        assert!(!yaml.contains("uuid"));
        assert!(!yaml.contains("password"));
    }

    #[test]
    fn import_profile_reuses_safe_managed_references() {
        let index =
            "current: old\nitems:\n- uid: Merge\n  type: merge\n- uid: Script\n  type: script\n";
        let refs = extract_profile_refs(index);
        let result = append_profile(
            index,
            "safecc-123",
            "safecc-123.yaml",
            Some("https://example.com/sub"),
            "airport",
            "美国 · 洛杉矶",
            &refs,
        )
        .expect("profile should be appended");
        assert!(result.contains("current: safecc-123"));
        assert!(result.contains("url: \"https://example.com/sub\""));
        assert!(result.contains("merge: Merge"));
        assert!(result.contains("script: Script"));
    }
}
