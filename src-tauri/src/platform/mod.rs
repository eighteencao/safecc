use std::process::Command;

use serde::Serialize;

#[derive(Debug, Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct PlatformSnapshot {
    pub platform: String,
    pub architecture: String,
    pub locale: Option<String>,
    pub timezone: Option<String>,
    pub default_browser: Option<String>,
    pub claude_desktop_installed: bool,
    pub clash_verge_installed: bool,
    pub location_permission: String,
    pub safe_cache_cleanup_available: bool,
}

pub fn snapshot() -> PlatformSnapshot {
    #[cfg(target_os = "macos")]
    {
        macos::snapshot()
    }
    #[cfg(all(not(target_os = "macos"), target_os = "windows"))]
    {
        windows::snapshot()
    }

    #[cfg(not(any(target_os = "macos", target_os = "windows")))]
    {
        PlatformSnapshot {
            platform: std::env::consts::OS.to_string(),
            architecture: std::env::consts::ARCH.to_string(),
            locale: std::env::var("LANG").ok(),
            timezone: std::env::var("TZ").ok(),
            default_browser: None,
            claude_desktop_installed: false,
            clash_verge_installed: false,
            location_permission: "manual-check-required".to_string(),
            safe_cache_cleanup_available: false,
        }
    }
}

fn command_output(program: &str, args: &[&str]) -> Option<String> {
    let output = Command::new(program).args(args).output().ok()?;
    if !output.status.success() {
        return None;
    }
    let value = String::from_utf8(output.stdout).ok()?.trim().to_string();
    (!value.is_empty()).then_some(value)
}

#[cfg(target_os = "macos")]
mod macos {
    use super::{command_output, PlatformSnapshot};
    use std::path::Path;

    pub fn snapshot() -> PlatformSnapshot {
        let home = std::env::var_os("HOME").map(std::path::PathBuf::from);
        let claude_installed = [
            Path::new("/Applications/Claude.app").to_path_buf(),
            home.as_ref()
                .map(|path| path.join("Applications/Claude.app"))
                .unwrap_or_default(),
        ]
        .iter()
        .any(|path| path.exists());
        let clash_installed = [
            Path::new("/Applications/Clash Verge Rev.app").to_path_buf(),
            Path::new("/Applications/Clash Verge.app").to_path_buf(),
            home.as_ref()
                .map(|path| path.join("Applications/Clash Verge Rev.app"))
                .unwrap_or_default(),
            home.as_ref()
                .map(|path| path.join("Applications/Clash Verge.app"))
                .unwrap_or_default(),
        ]
        .iter()
        .any(|path| path.exists());

        PlatformSnapshot {
            platform: "macos".to_string(),
            architecture: std::env::consts::ARCH.to_string(),
            locale: command_output("/usr/bin/defaults", &["read", "-g", "AppleLocale"]),
            timezone: std::fs::read_link("/etc/localtime")
                .ok()
                .and_then(|path| path.to_str().map(str::to_string))
                .map(|path| {
                    path.trim_start_matches("/var/db/timezone/zoneinfo/")
                        .to_string()
                }),
            default_browser: command_output(
                "/usr/bin/defaults",
                &[
                    "read",
                    "com.apple.LaunchServices/com.apple.launchservices.secure",
                    "LSHandlers",
                ],
            )
            .and_then(|handlers| parse_https_handler(&handlers)),
            claude_desktop_installed: claude_installed,
            clash_verge_installed: clash_installed,
            location_permission: "manual-check-required".to_string(),
            safe_cache_cleanup_available: true,
        }
    }

    fn parse_https_handler(value: &str) -> Option<String> {
        let mut scheme = None;
        let mut role = None;
        for line in value.lines() {
            let line = line.trim();
            if line.starts_with("LSHandlerRoleAll =") {
                role = parse_assignment(line);
            } else if line.starts_with("LSHandlerURLScheme =") {
                scheme = parse_assignment(line);
            } else if line.starts_with("},") {
                if scheme.as_deref() == Some("https") {
                    return role;
                }
                scheme = None;
                role = None;
            }
        }
        None
    }

    fn parse_assignment(line: &str) -> Option<String> {
        let value = line.split_once('=')?.1.trim().trim_end_matches(';').trim();
        Some(value.trim_matches('"').to_string())
    }

    #[cfg(test)]
    mod tests {
        use super::parse_https_handler;

        #[test]
        fn parses_browser_bundle_id_from_https_handler_block() {
            let handlers = r#"
                {
                    LSHandlerRoleAll = "com.google.chrome";
                    LSHandlerURLScheme = https;
                },
            "#;

            assert_eq!(
                parse_https_handler(handlers).as_deref(),
                Some("com.google.chrome")
            );
        }
    }
}

#[cfg(target_os = "windows")]
mod windows {
    use super::{command_output, PlatformSnapshot};

    pub fn snapshot() -> PlatformSnapshot {
        let local_app_data = std::env::var_os("LOCALAPPDATA").map(std::path::PathBuf::from);
        let claude_installed = local_app_data
            .as_ref()
            .map(|path| path.join("AnthropicClaude").exists())
            .unwrap_or(false);
        let clash_installed = local_app_data
            .as_ref()
            .map(|path| path.join("clash-verge-rev").exists())
            .unwrap_or(false);

        PlatformSnapshot {
            platform: "windows".to_string(),
            architecture: std::env::consts::ARCH.to_string(),
            locale: command_output(
                "powershell",
                &["-NoProfile", "-Command", "(Get-Culture).Name"],
            ),
            timezone: command_output("tzutil", &["/g"]),
            default_browser: default_browser(),
            claude_desktop_installed: claude_installed,
            clash_verge_installed: clash_installed,
            location_permission: "manual-check-required".to_string(),
            safe_cache_cleanup_available: true,
        }
    }

    fn default_browser() -> Option<String> {
        let output = command_output(
            "reg",
            &[
                "query",
                r"HKCU\Software\Microsoft\Windows\Shell\Associations\UrlAssociations\https\UserChoice",
                "/v",
                "ProgId",
            ],
        )?;
        output.lines().find_map(|line| {
            let value = line.split_whitespace().last()?;
            let normalized = value.to_ascii_lowercase();
            (normalized.contains("chrome")
                || normalized.contains("edge")
                || normalized.contains("firefox"))
            .then(|| value.to_string())
        })
    }
}
