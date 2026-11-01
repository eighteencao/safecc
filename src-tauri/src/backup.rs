use serde::Serialize;
use std::fs;
use std::io;
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

const MAX_BACKUP_FILES: usize = 512;
const MAX_BACKUP_BYTES: u64 = 50 * 1024 * 1024;

#[derive(Debug, Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct ClashBackupResult {
    pub backup_id: String,
    pub status: String,
    pub copied_files: usize,
    pub skipped_files: usize,
    pub backup_location: String,
    pub warnings: Vec<String>,
}

#[derive(Debug, Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct ClashBackupSummary {
    pub backup_id: String,
    pub copied_files: usize,
    pub total_bytes: u64,
}

#[derive(Debug, Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct ClashRestoreResult {
    pub backup_id: String,
    pub pre_restore_backup_id: String,
    pub status: String,
    pub restored_files: usize,
    pub warnings: Vec<String>,
}

#[derive(Default)]
struct CopyStats {
    copied_files: usize,
    skipped_files: usize,
    copied_bytes: u64,
}

pub fn create_clash_backup(confirmed: bool) -> Result<ClashBackupResult, String> {
    if !confirmed {
        return Err("创建备份前必须明确确认影响范围".to_string());
    }

    let source =
        crate::clash::data_dir().ok_or_else(|| "未找到 Clash Verge Rev 数据目录".to_string())?;
    if !source.is_dir() {
        return Err("未发现 Clash Verge Rev 数据目录，无法创建备份".to_string());
    }

    let root = assistant_backup_root().ok_or_else(|| "无法确定本机备份目录".to_string())?;
    fs::create_dir_all(&root).map_err(|error| io_error("创建备份目录", error))?;
    protect_directory(&root).map_err(|error| io_error("保护备份目录", error))?;
    let (backup_id, backup_dir) = create_unique_backup_dir(&root)?;
    protect_directory(&backup_dir).map_err(|error| io_error("保护备份快照", error))?;

    let result = copy_known_files(&source, &backup_dir);
    let stats = match result {
        Ok(stats) => stats,
        Err(error) => {
            let _ = fs::remove_dir_all(&backup_dir);
            return Err(io_error("创建 Clash 本地快照", error));
        }
    };

    if stats.copied_files == 0 {
        let _ = fs::remove_dir_all(&backup_dir);
        return Err("Clash 数据目录中没有可备份的配置文件".to_string());
    }

    Ok(ClashBackupResult {
        backup_id,
        status: "created".to_string(),
        copied_files: stats.copied_files,
        skipped_files: stats.skipped_files,
        backup_location: backup_dir.to_string_lossy().into_owned(),
        warnings: vec![
            "备份仅包含 Clash 配置文件与 profiles 目录，不包含日志和缓存".to_string(),
            "备份内容可能包含订阅地址、节点认证信息或 UUID，请勿上传或分享".to_string(),
        ],
    })
}

pub fn list_clash_backups() -> Result<Vec<ClashBackupSummary>, String> {
    let Some(root) = assistant_backup_root() else {
        return Ok(Vec::new());
    };
    if !root.is_dir() {
        return Ok(Vec::new());
    }

    let mut backups = Vec::new();
    for entry in fs::read_dir(&root).map_err(|error| io_error("读取备份目录", error))? {
        let entry = entry.map_err(|error| io_error("读取备份项目", error))?;
        let path = entry.path();
        let metadata =
            fs::symlink_metadata(&path).map_err(|error| io_error("读取备份项目属性", error))?;
        let backup_id = entry.file_name().to_string_lossy().into_owned();
        if !metadata.is_dir() || metadata.file_type().is_symlink() || !is_safe_backup_id(&backup_id)
        {
            continue;
        }

        let (copied_files, total_bytes) =
            inspect_files(&path).map_err(|error| io_error("读取备份摘要", error))?;
        if copied_files > 0 {
            backups.push(ClashBackupSummary {
                backup_id,
                copied_files,
                total_bytes,
            });
        }
    }
    backups.sort_by(|left, right| right.backup_id.cmp(&left.backup_id));
    Ok(backups)
}

pub fn restore_clash_backup(
    backup_id: String,
    confirmed: bool,
) -> Result<ClashRestoreResult, String> {
    if !confirmed {
        return Err("恢复配置前必须明确确认影响范围".to_string());
    }
    if !is_safe_backup_id(&backup_id) {
        return Err("备份编号格式无效".to_string());
    }

    let snapshot = assistant_backup_root()
        .ok_or_else(|| "无法确定本机备份目录".to_string())?
        .join(&backup_id);
    if !snapshot.is_dir() {
        return Err("找不到所选备份，可能已被移动或删除".to_string());
    }

    let files =
        collect_restore_files(&snapshot).map_err(|error| io_error("读取恢复快照", error))?;
    if files.is_empty() {
        return Err("所选备份没有可恢复的 Clash 配置文件".to_string());
    }
    if files.len() > MAX_BACKUP_FILES {
        return Err("恢复快照文件数量超过安全上限".to_string());
    }

    let current =
        crate::clash::data_dir().ok_or_else(|| "未找到 Clash Verge Rev 数据目录".to_string())?;
    if !current.is_dir() {
        return Err("未发现 Clash Verge Rev 数据目录，无法恢复".to_string());
    }

    // Always preserve the current state before applying a historical snapshot.
    let pre_restore = create_clash_backup(true)?;
    let root = assistant_backup_root().ok_or_else(|| "无法确定本机备份目录".to_string())?;
    let staging = create_staging_dir(&root, &backup_id)?;
    let mut staging_stats = CopyStats::default();
    let stage_result = files.iter().try_for_each(|relative| {
        copy_file(
            &snapshot.join(relative),
            &staging.join(relative),
            &mut staging_stats,
        )
    });
    if let Err(error) = stage_result {
        let _ = fs::remove_dir_all(&staging);
        return Err(io_error("准备恢复快照", error));
    }

    let mut restored_files = 0usize;
    for relative in &files {
        if let Err(error) = replace_file(&staging.join(relative), &current.join(relative)) {
            let _ = fs::remove_dir_all(&staging);
            return Err(io_error("恢复 Clash 配置", error));
        }
        restored_files += 1;
    }
    let _ = fs::remove_dir_all(&staging);

    Ok(ClashRestoreResult {
        backup_id,
        pre_restore_backup_id: pre_restore.backup_id,
        status: "restored".to_string(),
        restored_files,
        warnings: vec![
            "恢复只替换配置文件和 profiles/ 中的文件，不触碰日志与缓存".to_string(),
            "请重新加载 Clash Verge Rev 并读取最终生效状态".to_string(),
        ],
    })
}

fn is_safe_backup_id(value: &str) -> bool {
    !value.is_empty()
        && value.len() <= 80
        && value
            .bytes()
            .all(|byte| byte.is_ascii_digit() || byte == b'-')
}

fn inspect_files(root: &Path) -> io::Result<(usize, u64)> {
    let mut files = 0usize;
    let mut bytes = 0u64;
    inspect_files_recursive(root, &mut files, &mut bytes)?;
    Ok((files, bytes))
}

fn inspect_files_recursive(root: &Path, files: &mut usize, bytes: &mut u64) -> io::Result<()> {
    for entry in fs::read_dir(root)? {
        let entry = entry?;
        let path = entry.path();
        let metadata = fs::symlink_metadata(&path)?;
        if metadata.file_type().is_symlink() {
            continue;
        }
        if metadata.is_dir() {
            inspect_files_recursive(&path, files, bytes)?;
        } else if metadata.is_file() {
            *files += 1;
            *bytes = bytes.saturating_add(metadata.len());
        }
    }
    Ok(())
}

fn collect_restore_files(root: &Path) -> io::Result<Vec<PathBuf>> {
    let mut files = Vec::new();
    collect_restore_files_recursive(root, root, &mut files)?;
    Ok(files)
}

fn collect_restore_files_recursive(
    root: &Path,
    current: &Path,
    files: &mut Vec<PathBuf>,
) -> io::Result<()> {
    for entry in fs::read_dir(current)? {
        let entry = entry?;
        let path = entry.path();
        let metadata = fs::symlink_metadata(&path)?;
        if metadata.file_type().is_symlink() {
            continue;
        }
        if metadata.is_dir() {
            collect_restore_files_recursive(root, &path, files)?;
        } else if metadata.is_file() {
            let relative = path
                .strip_prefix(root)
                .map_err(|_| io::Error::new(io::ErrorKind::InvalidData, "恢复路径无效"))?
                .to_path_buf();
            if is_allowed_restore_path(&relative) {
                files.push(relative);
            }
        }
    }
    Ok(())
}

fn is_allowed_restore_path(path: &Path) -> bool {
    let mut components = path.components();
    match components.next() {
        Some(std::path::Component::Normal(name))
            if name == "verge.yaml" || name == "config.yaml" || name == "profiles.yaml" =>
        {
            components.next().is_none()
        }
        Some(std::path::Component::Normal(name)) if name == "profiles" => {
            components.all(|component| matches!(component, std::path::Component::Normal(_)))
        }
        _ => false,
    }
}

fn create_staging_dir(root: &Path, backup_id: &str) -> Result<PathBuf, String> {
    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_err(|_| "系统时间不可用，无法准备恢复".to_string())?;
    let base = format!(".safecc-restore-{backup_id}-{}", now.as_millis());
    for suffix in 0..100u32 {
        let name = if suffix == 0 {
            base.clone()
        } else {
            format!("{base}-{suffix}")
        };
        let path = root.join(name);
        match fs::create_dir(&path) {
            Ok(()) => {
                protect_directory(&path).map_err(|error| io_error("保护恢复暂存目录", error))?;
                return Ok(path);
            }
            Err(error) if error.kind() == io::ErrorKind::AlreadyExists => continue,
            Err(error) => return Err(io_error("创建恢复暂存目录", error)),
        }
    }
    Err("无法生成恢复暂存目录".to_string())
}

fn replace_file(source: &Path, destination: &Path) -> io::Result<()> {
    if let Ok(metadata) = fs::symlink_metadata(destination) {
        if metadata.file_type().is_symlink() {
            return Err(io::Error::new(
                io::ErrorKind::PermissionDenied,
                "目标配置路径是符号链接",
            ));
        }
        if !metadata.is_file() {
            return Err(io::Error::new(
                io::ErrorKind::InvalidInput,
                "目标配置路径不是普通文件",
            ));
        }
    }
    if let Some(parent) = destination.parent() {
        fs::create_dir_all(parent)?;
        protect_directory(parent)?;
    }
    fs::copy(source, destination)?;
    protect_file(destination)?;
    Ok(())
}

fn copy_known_files(source: &Path, destination: &Path) -> io::Result<CopyStats> {
    let mut stats = CopyStats::default();
    for relative in ["verge.yaml", "config.yaml", "profiles.yaml"] {
        let path = source.join(relative);
        if path.is_file() {
            copy_file(&path, &destination.join(relative), &mut stats)?;
        } else {
            stats.skipped_files += 1;
        }
    }

    let profiles = source.join("profiles");
    if profiles.is_dir() {
        copy_directory(&profiles, &destination.join("profiles"), &mut stats)?;
    } else {
        stats.skipped_files += 1;
    }
    Ok(stats)
}

fn copy_directory(source: &Path, destination: &Path, stats: &mut CopyStats) -> io::Result<()> {
    fs::create_dir_all(destination)?;
    protect_directory(destination)?;
    for entry in fs::read_dir(source)? {
        let entry = entry?;
        let source_path = entry.path();
        let destination_path = destination.join(entry.file_name());
        let metadata = fs::symlink_metadata(&source_path)?;

        if metadata.file_type().is_symlink() {
            stats.skipped_files += 1;
        } else if metadata.is_dir() {
            copy_directory(&source_path, &destination_path, stats)?;
        } else if metadata.is_file() {
            copy_file(&source_path, &destination_path, stats)?;
        } else {
            stats.skipped_files += 1;
        }
    }
    Ok(())
}

fn copy_file(source: &Path, destination: &Path, stats: &mut CopyStats) -> io::Result<()> {
    if stats.copied_files >= MAX_BACKUP_FILES {
        return Err(io::Error::new(
            io::ErrorKind::FileTooLarge,
            "备份文件数量超过安全上限",
        ));
    }
    let size = fs::metadata(source)?.len();
    if stats.copied_bytes.saturating_add(size) > MAX_BACKUP_BYTES {
        return Err(io::Error::new(
            io::ErrorKind::FileTooLarge,
            "备份总大小超过安全上限",
        ));
    }
    if let Some(parent) = destination.parent() {
        fs::create_dir_all(parent)?;
        protect_directory(parent)?;
    }
    fs::copy(source, destination)?;
    protect_file(destination)?;
    stats.copied_files += 1;
    stats.copied_bytes += size;
    Ok(())
}

fn create_unique_backup_dir(root: &Path) -> Result<(String, PathBuf), String> {
    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_err(|_| "系统时间不可用，无法创建备份编号".to_string())?;
    let base = format!("{}-{:03}", now.as_secs(), now.subsec_millis());

    for suffix in 0..100u32 {
        let backup_id = if suffix == 0 {
            base.clone()
        } else {
            format!("{base}-{suffix}")
        };
        let path = root.join(&backup_id);
        match fs::create_dir(&path) {
            Ok(()) => return Ok((backup_id, path)),
            Err(error) if error.kind() == io::ErrorKind::AlreadyExists => continue,
            Err(error) => return Err(io_error("创建备份快照目录", error)),
        }
    }
    Err("无法生成唯一的备份编号".to_string())
}

fn assistant_backup_root() -> Option<PathBuf> {
    #[cfg(target_os = "macos")]
    {
        return std::env::var_os("HOME")
            .map(PathBuf::from)
            .map(|home| home.join("Library/Application Support/safecc/backups"));
    }

    #[cfg(target_os = "windows")]
    {
        return std::env::var_os("APPDATA")
            .map(PathBuf::from)
            .map(|app_data| app_data.join("safecc/backups"));
    }

    #[cfg(not(any(target_os = "macos", target_os = "windows")))]
    {
        std::env::var_os("XDG_DATA_HOME")
            .map(PathBuf::from)
            .or_else(|| {
                std::env::var_os("HOME").map(|home| PathBuf::from(home).join(".local/share"))
            })
            .map(|root| root.join("safecc/backups"))
    }
}

fn io_error(action: &str, error: io::Error) -> String {
    format!("{action}失败：{}", error.kind())
}

fn protect_directory(path: &Path) -> io::Result<()> {
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        fs::set_permissions(path, fs::Permissions::from_mode(0o700))?;
    }
    let _ = path;
    Ok(())
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

#[cfg(test)]
mod tests {
    use super::{create_clash_backup, is_safe_backup_id, restore_clash_backup};

    #[test]
    fn backup_requires_explicit_confirmation() {
        let error = create_clash_backup(false).expect_err("未经确认不得创建备份");
        assert!(error.contains("明确确认"));
    }

    #[test]
    fn restore_requires_confirmation_and_safe_id() {
        let error =
            restore_clash_backup("../other".to_string(), false).expect_err("未经确认不得恢复");
        assert!(error.contains("明确确认"));
        assert!(is_safe_backup_id("1760000000-123"));
        assert!(!is_safe_backup_id("../other"));
    }
}
