use serde::Serialize;

mod backup;
mod browser;
mod browser_engine;
mod cache;
mod clash;
mod deployment;
mod open;
mod platform;
mod profiles;
mod region;
mod source;
mod verification;
mod vps;

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct RuntimeInfo {
    runtime: &'static str,
    platform: &'static str,
    architecture: &'static str,
    locale: Option<String>,
    timezone: Option<String>,
}

#[derive(Debug, Serialize)]
struct ChangePlan {
    source: String,
    target_region: String,
    actions: Vec<String>,
    requires_confirmation: Vec<String>,
    side_effects: Vec<String>,
}

#[tauri::command]
fn get_runtime_info() -> RuntimeInfo {
    RuntimeInfo {
        runtime: "tauri",
        platform: std::env::consts::OS,
        architecture: std::env::consts::ARCH,
        locale: std::env::var("LC_ALL")
            .ok()
            .or_else(|| std::env::var("LANG").ok()),
        timezone: std::env::var("TZ").ok(),
    }
}

#[tauri::command]
fn create_change_plan(source: String, target_region: String) -> ChangePlan {
    let source_label = match source.as_str() {
        "vps" => "自建 VPS",
        "static" => "静态 IP 订阅",
        "airport" => "机场订阅",
        _ => "未识别来源",
    };

    ChangePlan {
        source: source_label.to_string(),
        target_region,
        actions: vec![
            "写入 Clash Verge Rev 托管扩展配置".to_string(),
            "启用规则模式与 TUN".to_string(),
            "准备 DNS / UDP 验收".to_string(),
        ],
        requires_confirmation: vec![
            "系统设置可能需要管理员授权".to_string(),
            "清理缓存前展示具体范围".to_string(),
        ],
        side_effects: Vec::new(),
    }
}

#[tauri::command]
fn get_platform_snapshot() -> platform::PlatformSnapshot {
    platform::snapshot()
}

#[tauri::command]
fn get_clash_change_plan(source_kind: String, target_region: String) -> clash::ClashChangePlan {
    clash::change_plan(source_kind, target_region)
}

#[tauri::command]
fn apply_clash_extension(
    source_kind: String,
    target_region: String,
    confirm: bool,
) -> Result<clash::ClashApplyResult, String> {
    clash::apply_extension(source_kind, target_region, confirm)
}

#[tauri::command]
fn create_clash_backup(confirm: bool) -> Result<backup::ClashBackupResult, String> {
    backup::create_clash_backup(confirm)
}

#[tauri::command]
fn list_clash_backups() -> Result<Vec<backup::ClashBackupSummary>, String> {
    backup::list_clash_backups()
}

#[tauri::command]
fn restore_clash_backup(
    backup_id: String,
    confirm: bool,
) -> Result<backup::ClashRestoreResult, String> {
    backup::restore_clash_backup(backup_id, confirm)
}

#[tauri::command]
fn get_region_change_plan(target_region: String) -> region::RegionChangePlan {
    region::change_plan(target_region)
}

#[tauri::command]
fn apply_region_change(
    target_region: String,
    confirm: bool,
) -> Result<region::RegionApplyResult, String> {
    region::apply(target_region, confirm)
}

#[tauri::command]
fn restore_region_snapshot(
    snapshot_id: String,
    confirm: bool,
) -> Result<region::RegionRestoreResult, String> {
    region::restore(snapshot_id, confirm)
}

#[tauri::command]
fn preflight_source(
    source_kind: String,
    input: String,
    key_path: Option<String>,
    confirm: bool,
) -> Result<source::SourcePreflightResult, String> {
    source::preflight_with_key(source_kind, input, key_path, confirm)
}

#[tauri::command]
fn import_clash_subscription(
    source_kind: String,
    target_region: String,
    snapshot_id: String,
    input: String,
    confirm: bool,
) -> Result<clash::ClashImportResult, String> {
    clash::import_subscription(source_kind, target_region, snapshot_id, input, confirm)
}

#[tauri::command]
fn import_local_clash_subscription(
    target_region: String,
    snapshot_id: String,
    confirm: bool,
) -> Result<clash::ClashImportResult, String> {
    clash::import_local_subscription(target_region, snapshot_id, confirm)
}

#[tauri::command]
fn check_vps_read_only(
    target_input: String,
    key_path: String,
    host_key_id: String,
    confirm: bool,
) -> Result<vps::VpsCheckResult, String> {
    vps::check(target_input, key_path, host_key_id, confirm)
}

#[tauri::command]
fn deploy_vps(
    app: tauri::AppHandle,
    input: deployment::VpsDeployInput,
) -> Result<deployment::VpsDeployResult, String> {
    deployment::deploy(app, input)
}

#[tauri::command]
fn get_browser_change_plan(
    target_region: String,
    default_browser: Option<String>,
) -> browser::BrowserChangePlan {
    browser::change_plan(target_region, default_browser)
}

#[tauri::command]
fn apply_browser_environment(
    target_region: String,
    default_browser: Option<String>,
    confirm: bool,
) -> Result<browser::BrowserApplyResult, String> {
    browser::apply(target_region, default_browser, confirm)
}

#[tauri::command]
fn recognize_browser_environment(
    target_region: String,
    default_browser: Option<String>,
) -> Result<browser::BrowserRecognitionResult, String> {
    browser::recognize(target_region, default_browser)
}

#[tauri::command]
fn open_browser_setting_page_for(
    target: String,
    default_browser: Option<String>,
) -> Result<(), String> {
    browser::open_setting_page(target, default_browser)
}

#[tauri::command]
fn get_fingerprint_engine() -> browser_engine::FingerprintEngine {
    browser_engine::detect()
}

#[tauri::command]
fn open_fingerprint_browser(profile_id: String, confirm: bool) -> Result<String, String> {
    profiles::open_fingerprint_browser(profile_id, confirm)
}

#[tauri::command]
fn get_cache_plan() -> cache::CachePlan {
    cache::plan()
}

#[tauri::command]
fn clear_safe_cache(confirm: bool) -> Result<cache::CacheCleanupResult, String> {
    cache::clean(confirm)
}

#[tauri::command]
fn record_verification(
    target_region: String,
    route_alias: String,
    checks: Vec<String>,
    confirm: bool,
) -> Result<verification::VerificationRecord, String> {
    verification::record(target_region, route_alias, checks, confirm)
}

#[tauri::command]
fn open_safe_target(target: String) -> Result<(), String> {
    open::open_target(target)
}

#[tauri::command]
fn list_environment_profiles() -> Result<Vec<profiles::EnvironmentProfile>, String> {
    profiles::list()
}

#[tauri::command]
fn create_environment_profile(
    name: String,
    target_region: String,
    confirm: bool,
) -> Result<profiles::EnvironmentProfile, String> {
    profiles::create(name, target_region, confirm)
}

#[tauri::command]
fn open_environment_profile(
    profile_id: String,
    action: String,
    confirm: bool,
) -> Result<(), String> {
    profiles::open(profile_id, action, confirm)
}

pub fn run() {
    tauri::Builder::default()
        .invoke_handler(tauri::generate_handler![
            get_runtime_info,
            create_change_plan,
            get_platform_snapshot,
            get_clash_change_plan,
            apply_clash_extension,
            create_clash_backup,
            list_clash_backups,
            restore_clash_backup,
            get_region_change_plan,
            apply_region_change,
            restore_region_snapshot,
            preflight_source,
            import_clash_subscription,
            import_local_clash_subscription,
            check_vps_read_only,
            deploy_vps,
            get_browser_change_plan,
            apply_browser_environment,
            recognize_browser_environment,
            open_browser_setting_page_for,
            get_fingerprint_engine,
            open_fingerprint_browser,
            get_cache_plan,
            clear_safe_cache,
            record_verification,
            open_safe_target,
            list_environment_profiles,
            create_environment_profile,
            open_environment_profile
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

#[cfg(test)]
mod tests {
    use super::{create_change_plan, platform};

    #[test]
    fn change_plan_is_preview_only() {
        let plan = create_change_plan("airport".to_string(), "美国 · 洛杉矶".to_string());

        assert_eq!(plan.source, "机场订阅");
        assert_eq!(plan.target_region, "美国 · 洛杉矶");
        assert!(plan.side_effects.is_empty());
        assert!(!plan.actions.is_empty());
        assert!(!plan.requires_confirmation.is_empty());
    }

    #[test]
    fn unknown_source_does_not_expand_scope() {
        let plan = create_change_plan("unexpected".to_string(), "新加坡".to_string());

        assert_eq!(plan.source, "未识别来源");
        assert!(plan.side_effects.is_empty());
    }

    #[test]
    fn platform_snapshot_is_read_only_shape() {
        let snapshot = platform::snapshot();

        assert!(!snapshot.platform.is_empty());
        assert!(!snapshot.architecture.is_empty());
        assert_eq!(snapshot.location_permission, "manual-check-required");
    }
}
