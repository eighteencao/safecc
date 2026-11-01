use std::process::Command;

pub fn open_target(target: String) -> Result<(), String> {
    if target == "clash" {
        return open_clash();
    }
    let destination = match target.as_str() {
        "verification" => "https://ip.net.coffee/claude/",
        "location" => {
            #[cfg(target_os = "macos")]
            {
                "x-apple.systempreferences:com.apple.preference.security?Privacy_LocationServices"
            }
            #[cfg(target_os = "windows")]
            {
                "ms-settings:privacy-location"
            }
            #[cfg(not(any(target_os = "macos", target_os = "windows")))]
            {
                "https://support.apple.com/guide/mac-help/control-location-information-mac-mh35873/mac"
            }
        }
        "claude-settings" => "https://claude.ai/settings",
        "claude-login" => "https://claude.ai/login",
        "bandwagon" => "https://bandwagonhost.com/",
        "dmit" => "https://www.dmit.io/",
        _ => return Err("不允许打开未登记的外部地址".to_string()),
    };

    #[cfg(target_os = "macos")]
    let result = Command::new("/usr/bin/open").arg(destination).status();

    #[cfg(target_os = "windows")]
    let result = Command::new("cmd")
        .args(["/C", "start", "", destination])
        .status();

    #[cfg(not(any(target_os = "macos", target_os = "windows")))]
    let result = Command::new("xdg-open").arg(destination).status();

    match result {
        Ok(status) if status.success() => Ok(()),
        _ => Err("无法打开系统或浏览器设置".to_string()),
    }
}

fn open_clash() -> Result<(), String> {
    #[cfg(target_os = "macos")]
    {
        for application in ["Clash Verge Rev", "Clash Verge"] {
            let status = Command::new("/usr/bin/open")
                .args(["-a", application])
                .status();
            if status.is_ok_and(|value| value.success()) {
                return Ok(());
            }
        }
        return Err("未找到 Clash Verge Rev，请先安装受支持的客户端".to_string());
    }

    #[cfg(target_os = "windows")]
    {
        let local_app_data = std::env::var_os("LOCALAPPDATA").map(std::path::PathBuf::from);
        let program_files = std::env::var_os("PROGRAMFILES").map(std::path::PathBuf::from);
        let mut candidates = Vec::new();
        for root in [local_app_data, program_files].into_iter().flatten() {
            candidates.push(root.join("Clash Verge Rev/Clash Verge Rev.exe"));
            candidates.push(root.join("Clash Verge/Clash Verge.exe"));
        }
        for path in candidates {
            if path.is_file() && Command::new(&path).spawn().is_ok() {
                return Ok(());
            }
        }
        return Err("未找到 Clash Verge Rev，请先安装受支持的客户端".to_string());
    }

    #[cfg(not(any(target_os = "macos", target_os = "windows")))]
    {
        Err("当前平台暂不支持自动打开 Clash Verge Rev".to_string())
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn target_allowlist_is_not_user_supplied_url() {
        assert!(super::open_target("https://example.com".to_string()).is_err());
    }

    #[test]
    fn clash_target_is_fixed_and_not_a_path_argument() {
        assert!(super::open_target("/tmp/Clash Verge Rev.app".to_string()).is_err());
    }
}
