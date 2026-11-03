import { invoke } from "@tauri-apps/api/core";
import { getRegionConfig } from "./regions";

export type RuntimeInfo = {
  runtime: "tauri" | "browser";
  platform: string;
  architecture: string;
  locale: string | null;
  timezone: string | null;
};

export type ChangePlan = {
  source: "vps" | "static" | "airport";
  target_region: string;
  actions: string[];
  requires_confirmation: string[];
  side_effects: string[];
};

export type PlatformSnapshot = {
  platform: string;
  architecture: string;
  locale: string | null;
  timezone: string | null;
  defaultBrowser: string | null;
  claudeDesktopInstalled: boolean;
  clashVergeInstalled: boolean;
  locationPermission: string;
  safeCacheCleanupAvailable: boolean;
};

export type PlannedFile = {
  label: string;
  relativePath: string;
  exists: boolean;
  containsSecrets: boolean;
};

export type ClashChangePlan = {
  platform: string;
  dataDirExists: boolean;
  profileFiles: PlannedFile[];
  managedExtension: string;
  targetRegion: string;
  sourceKind: string;
  claudeRuleCount: number;
  dnsAndTunFields: string[];
  warnings: string[];
  requiresConfirmation: string[];
};

export type ClashApplyResult = {
  targetRegion: string;
  sourceKind: string;
  status: "written" | "preview-only";
  extensionRelativePath: string;
  writtenFields: string[];
  warnings: string[];
  requiresReload: boolean;
};

export type ClashBackupResult = {
  backupId: string;
  status: "created" | "preview-only";
  copiedFiles: number;
  skippedFiles: number;
  backupLocation: string | null;
  warnings: string[];
};

export type ClashBackupSummary = {
  backupId: string;
  copiedFiles: number;
  totalBytes: number;
};

export type ClashRestoreResult = {
  backupId: string;
  preRestoreBackupId: string;
  status: "restored" | "preview-only";
  restoredFiles: number;
  warnings: string[];
};

export type RegionChangePlan = {
  platform: string;
  targetRegion: string;
  timezone: string;
  locale: string;
  language: string;
  actions: string[];
  permissionPrompts: string[];
  restartRequirements: string[];
  manualSteps: string[];
};

export type RegionApplyResult = {
  targetRegion: string;
  status: "applied" | "preview-only";
  snapshotId: string;
  changed: string[];
  permissionRequested: boolean;
  restartRequirements: string[];
  warnings: string[];
};

export type RegionRestoreResult = {
  snapshotId: string;
  preRestoreSnapshotId: string;
  status: "restored" | "preview-only";
  restored: string[];
  permissionRequested: boolean;
  warnings: string[];
};

export type BrowserChangePlan = {
  platform: string;
  browser: string;
  browserLabel: string;
  targetRegion: string;
  language: string;
  profileExists: boolean;
  automatic: boolean;
  actions: string[];
  permissionPrompts: string[];
  restartRequirements: string[];
  manualSteps: string[];
  warnings: string[];
};

export type BrowserApplyResult = {
  status: "applied" | "preview-only";
  browser: string;
  changed: string[];
  backupId: string | null;
  requiresRestart: boolean;
  warnings: string[];
};

export type CacheTarget = {
  label: string;
  exists: boolean;
};

export type CachePlan = {
  platform: string;
  targets: CacheTarget[];
  dnsFlushAvailable: boolean;
  preserved: string[];
  warnings: string[];
};

export type CacheCleanupResult = {
  status: "cleaned" | "preview-only";
  cleanedEntries: number;
  cleanedBytes: number;
  dnsFlushRequested: boolean;
  warnings: string[];
};

export type SourcePreflightResult = {
  status: "valid" | "manual" | "preview-only";
  sourceKind: string;
  sourceId: string | null;
  format: string;
  nodeCount: number;
  bytes: number;
  warnings: string[];
  actions: string[];
  fingerprint: string | null;
};

export type ClashImportResult = {
  status: string;
  profileUid: string;
  profileRelativePath: string;
  sourceKind: string;
  targetRegion: string;
  activated: boolean;
  warnings: string[];
};

export type VpsCheckResult = {
  status: string;
  host: string;
  user: string;
  port: number;
  operatingSystem: string;
  remoteChanges: boolean;
  warnings: string[];
  actions: string[];
};

export type VpsDeployInput = {
  targetInput: string;
  pemPath: string;
  ppkPath: string;
  pubPath: string;
  proxyHost: string;
  proxyPort: number;
  proxyUser: string;
  proxyPassword: string;
  expectedIp: string;
  hostKeyId: string;
  confirm: boolean;
};

export type VpsDeployResult = {
  status: string;
  workspaceId: string;
  subscriptionSnapshotId: string;
  remoteChangesRequested: boolean;
  warnings: string[];
  actions: string[];
};

export type VerificationRecord = {
  recordId: string;
  targetRegion: string;
  routeAlias: string;
  clashConfigVersion: string;
  recordedAtUnixMs: number;
  checks: string[];
  results: Array<{ check: string; status: string }>;
  status: string;
};

export type EnvironmentProfile = {
  id: string;
  name: string;
  targetRegion: string;
  createdAtUnixMs: number;
  claudeConfigDir: string;
  browserProfileDir: string;
  launcherPath: string;
  browser: string;
  browserLabel: string;
  browserAvailable: boolean;
  previewOnly: boolean;
  isolation: string[];
  warnings: string[];
};

const inTauri = () => typeof window !== "undefined" && "__TAURI_INTERNALS__" in window;

export async function getRuntimeInfo(): Promise<RuntimeInfo> {
  if (inTauri()) return invoke<RuntimeInfo>("get_runtime_info");
  return {
    runtime: "browser",
    platform: navigator.platform || "browser",
    architecture: "unknown",
    locale: navigator.language || null,
    timezone: Intl.DateTimeFormat().resolvedOptions().timeZone || null,
  };
}

export async function preflightSource(sourceKind: string, input: string, confirm: boolean, keyPath: string | null = null): Promise<SourcePreflightResult> {
  if (inTauri()) return invoke<SourcePreflightResult>("preflight_source", { sourceKind, input, keyPath, confirm });
  return {
    status: "preview-only",
    sourceKind,
    sourceId: null,
    format: sourceKind === "vps" ? "ssh-endpoint" : "browser-preview",
    nodeCount: 0,
    bytes: 0,
    warnings: ["浏览器预览不会请求或保存订阅内容", "请在 Tauri 本机窗口中确认后进行本地预检"],
    actions: ["下一步会先备份 Clash Verge Rev 配置"],
    fingerprint: null,
  };
}

export async function importClashSubscription(sourceKind: string, targetRegion: string, snapshotId: string, input: string, confirm: boolean): Promise<ClashImportResult> {
  if (inTauri()) return invoke<ClashImportResult>("import_clash_subscription", { sourceKind, targetRegion, snapshotId, input, confirm });
  return {
    status: "preview-only",
    profileUid: "safecc-preview",
    profileRelativePath: "profiles/safecc-preview.yaml",
    sourceKind,
    targetRegion,
    activated: false,
    warnings: ["浏览器预览不会修改 Clash Verge Rev 文件"],
  };
}

export async function importLocalClashSubscription(targetRegion: string, snapshotId: string, confirm: boolean): Promise<ClashImportResult> {
  if (inTauri()) return invoke<ClashImportResult>("import_local_clash_subscription", { targetRegion, snapshotId, confirm });
  return {
    status: "preview-only",
    profileUid: "safecc-preview-vps",
    profileRelativePath: "profiles/safecc-preview-vps.yaml",
    sourceKind: "vps",
    targetRegion,
    activated: false,
    warnings: ["浏览器预览不会修改 Clash Verge Rev 文件"],
  };
}

export async function checkVpsReadOnly(targetInput: string, keyPath: string, hostKeyId: string, confirm: boolean): Promise<VpsCheckResult> {
  if (inTauri()) return invoke<VpsCheckResult>("check_vps_read_only", { targetInput, keyPath, hostKeyId, confirm });
  return {
    status: "preview-only",
    host: "preview",
    user: "root",
    port: 22,
    operatingSystem: "浏览器预览",
    remoteChanges: false,
    warnings: ["浏览器预览不会连接 VPS 或执行远端命令"],
    actions: ["Tauri 本机窗口会在明确确认后使用严格 SSH 只读检查"],
  };
}

export async function deployVps(input: VpsDeployInput): Promise<VpsDeployResult> {
  if (inTauri()) return invoke<VpsDeployResult>("deploy_vps", { input });
  return {
    status: "preview-only",
    workspaceId: "preview",
    subscriptionSnapshotId: "preview",
    remoteChangesRequested: false,
    warnings: ["浏览器预览不会连接或修改 VPS"],
    actions: ["Tauri 本机窗口会在单独确认后启动分阶段部署"],
  };
}

export async function createChangePlan(source: ChangePlan["source"], targetRegion: string): Promise<ChangePlan> {
  if (inTauri()) return invoke<ChangePlan>("create_change_plan", { source, targetRegion });
  return {
    source,
    target_region: targetRegion,
    actions: ["写入 Clash Verge Rev 托管扩展配置", "启用规则模式与 TUN", "准备 DNS / UDP 验收"],
    requires_confirmation: ["系统设置可能需要管理员授权", "清理缓存前展示具体范围"],
    side_effects: [],
  };
}

export async function getPlatformSnapshot(): Promise<PlatformSnapshot> {
  if (inTauri()) return invoke<PlatformSnapshot>("get_platform_snapshot");
  return {
    platform: navigator.platform || "browser",
    architecture: "unknown",
    locale: navigator.language || null,
    timezone: Intl.DateTimeFormat().resolvedOptions().timeZone || null,
    defaultBrowser: null,
    claudeDesktopInstalled: false,
    clashVergeInstalled: false,
    locationPermission: "manual-check-required",
    safeCacheCleanupAvailable: false,
  };
}

export async function getClashChangePlan(sourceKind: string, targetRegion: string): Promise<ClashChangePlan> {
  if (inTauri()) return invoke<ClashChangePlan>("get_clash_change_plan", { sourceKind, targetRegion });
  return {
    platform: "browser",
    dataDirExists: false,
    profileFiles: [],
    managedExtension: "Clash Verge Rev · Global Merge（profiles/Merge.yaml）",
    targetRegion,
    sourceKind,
    claudeRuleCount: 9,
    dnsAndTunFields: ["tun.enable", "tun.auto-route", "tun.auto-detect-interface", "tun.dns-hijack", "dns.enhanced-mode"],
    warnings: ["浏览器预览不会读取或写入 Clash 配置", "Claude 规则默认指向 PROXY 策略组", "当前为 dry-run"],
    requiresConfirmation: ["创建本地备份", "写入托管扩展配置", "重新生成并验证运行配置"],
  };
}

export async function applyClashExtension(sourceKind: string, targetRegion: string, confirm: boolean): Promise<ClashApplyResult> {
  if (inTauri()) return invoke<ClashApplyResult>("apply_clash_extension", { sourceKind, targetRegion, confirm });
  return {
    targetRegion,
    sourceKind,
    status: "preview-only",
    extensionRelativePath: "profiles/Merge.yaml",
    writtenFields: [],
    warnings: ["浏览器预览不会读取或写入 Clash 配置", "请在 Tauri 本机窗口中确认后写入托管扩展"],
    requiresReload: true,
  };
}

export async function createClashBackup(confirm: boolean): Promise<ClashBackupResult> {
  if (inTauri()) return invoke<ClashBackupResult>("create_clash_backup", { confirm });
  return {
    backupId: "browser-preview",
    status: "preview-only",
    copiedFiles: 0,
    skippedFiles: 0,
    backupLocation: null,
    warnings: ["浏览器预览不会读取或写入 Clash 文件", "请在 Tauri 本机窗口中确认后创建真实备份"],
  };
}

export async function listClashBackups(): Promise<ClashBackupSummary[]> {
  if (inTauri()) return invoke<ClashBackupSummary[]>("list_clash_backups");
  return [];
}

export async function restoreClashBackup(backupId: string, confirm: boolean): Promise<ClashRestoreResult> {
  if (inTauri()) return invoke<ClashRestoreResult>("restore_clash_backup", { backupId, confirm });
  return {
    backupId,
    preRestoreBackupId: "browser-preview",
    status: "preview-only",
    restoredFiles: 0,
    warnings: ["浏览器预览不会读取或写入 Clash 文件", "请在 Tauri 本机窗口中恢复真实配置"],
  };
}

export async function getRegionChangePlan(targetRegion: string): Promise<RegionChangePlan> {
  if (inTauri()) return invoke<RegionChangePlan>("get_region_change_plan", { targetRegion });
  const selected = getRegionConfig(targetRegion);
  return {
    platform: "browser",
    targetRegion,
    ...selected,
    actions: ["保存当前系统设置快照", "设置时区、地区格式与首选语言", "重新读取并报告生效状态"],
    permissionPrompts: ["真实执行会请求系统管理员权限或 UAC"],
    restartRequirements: ["部分应用需要重启或注销后才会完全生效"],
    manualSteps: ["定位权限和浏览器权限继续在后续步骤确认"],
  };
}

export async function applyRegionChange(targetRegion: string, confirm: boolean): Promise<RegionApplyResult> {
  if (inTauri()) return invoke<RegionApplyResult>("apply_region_change", { targetRegion, confirm });
  return {
    targetRegion,
    status: "preview-only",
    snapshotId: "browser-preview",
    changed: [],
    permissionRequested: false,
    restartRequirements: ["浏览器预览不会修改系统设置"],
    warnings: ["请在 Tauri 本机窗口中确认后申请系统权限并执行"],
  };
}

export async function restoreRegionSnapshot(snapshotId: string, confirm: boolean): Promise<RegionRestoreResult> {
  if (inTauri()) return invoke<RegionRestoreResult>("restore_region_snapshot", { snapshotId, confirm });
  return {
    snapshotId,
    preRestoreSnapshotId: "browser-preview",
    status: "preview-only",
    restored: [],
    permissionRequested: false,
    warnings: ["浏览器预览不会修改系统设置", "请在 Tauri 本机窗口中确认后恢复地区快照"],
  };
}

export async function getBrowserChangePlan(targetRegion: string, defaultBrowser: string | null): Promise<BrowserChangePlan> {
  if (inTauri()) return invoke<BrowserChangePlan>("get_browser_change_plan", { targetRegion, defaultBrowser });
  const browserLabel = defaultBrowser ?? "未识别浏览器";
  const selected = getRegionConfig(targetRegion);
  return {
    platform: "browser",
    browser: "unknown",
    browserLabel,
    targetRegion,
    language: selected.browserLanguage,
    profileExists: false,
    automatic: false,
    actions: ["让浏览器语言偏好与目标地区一致", "将网站定位默认值设为阻止", "时区跟随系统"],
    permissionPrompts: ["真实执行会先备份浏览器偏好并要求确认"],
    restartRequirements: ["需要重启浏览器"],
    manualSteps: ["浏览器预览不会修改浏览器，请在 Tauri 本机窗口中确认"],
    warnings: ["浏览器预览不会读取或写入浏览器偏好"],
  };
}

export async function applyBrowserEnvironment(targetRegion: string, defaultBrowser: string | null, confirm: boolean): Promise<BrowserApplyResult> {
  if (inTauri()) return invoke<BrowserApplyResult>("apply_browser_environment", { targetRegion, defaultBrowser, confirm });
  return {
    status: "preview-only",
    browser: "unknown",
    changed: [],
    backupId: null,
    requiresRestart: true,
    warnings: ["浏览器预览不会读取或写入浏览器偏好", "请在 Tauri 本机窗口中确认后执行"],
  };
}

export async function getCachePlan(): Promise<CachePlan> {
  if (inTauri()) return invoke<CachePlan>("get_cache_plan");
  return {
    platform: "browser",
    targets: [],
    dnsFlushAvailable: false,
    preserved: ["Cookie 与登录会话", "密码、书签和历史", "Local Storage / IndexedDB", "浏览器扩展与用户文件"],
    warnings: ["浏览器预览不会读取或删除本机缓存"],
  };
}

export async function clearSafeCache(confirm: boolean): Promise<CacheCleanupResult> {
  if (inTauri()) return invoke<CacheCleanupResult>("clear_safe_cache", { confirm });
  return {
    status: "preview-only",
    cleanedEntries: 0,
    cleanedBytes: 0,
    dnsFlushRequested: false,
    warnings: ["浏览器预览不会读取或删除本机缓存", "请在 Tauri 本机窗口中确认后执行真实清理"],
  };
}

export async function recordVerification(targetRegion: string, routeAlias: string, checks: string[], confirm: boolean): Promise<VerificationRecord> {
  if (inTauri()) return invoke<VerificationRecord>("record_verification", { targetRegion, routeAlias, checks, confirm });
  return {
    recordId: "safecc-preview",
    targetRegion,
    routeAlias,
    clashConfigVersion: "safecc-managed-v1",
    recordedAtUnixMs: Date.now(),
    checks,
    results: checks.map((check) => ({ check, status: "confirmed" })),
    status: "preview-only",
  };
}

const PREVIEW_PROFILES_KEY = "safecc.preview.environmentProfiles";

function readPreviewProfiles(): EnvironmentProfile[] {
  try {
    const value = window.localStorage.getItem(PREVIEW_PROFILES_KEY);
    return value ? JSON.parse(value) as EnvironmentProfile[] : [];
  } catch {
    return [];
  }
}

export async function listEnvironmentProfiles(): Promise<EnvironmentProfile[]> {
  if (inTauri()) return invoke<EnvironmentProfile[]>("list_environment_profiles");
  return readPreviewProfiles();
}

export async function createEnvironmentProfile(name: string, targetRegion: string, confirm: boolean): Promise<EnvironmentProfile> {
  if (inTauri()) return invoke<EnvironmentProfile>("create_environment_profile", { name, targetRegion, confirm });
  if (!confirm) throw new Error("创建账号档案前必须确认隔离范围");
  const profiles = readPreviewProfiles();
  const trimmed = name.trim();
  if (!trimmed || trimmed.length > 32) throw new Error("档案名称需要 1 至 32 个字符");
  if (/[\/\\:*?"<>|]/.test(trimmed)) throw new Error("档案名称不能包含路径分隔符或系统保留字符");
  if (profiles.some((profile) => profile.name.toLocaleLowerCase() === trimmed.toLocaleLowerCase())) {
    throw new Error("已存在同名账号档案，请换一个名称");
  }
  if (profiles.length >= 8) throw new Error("最多保存 8 个本地账号档案");
  const timestamp = Date.now();
  const id = `profile-${timestamp}-preview`;
  const profile: EnvironmentProfile = {
    id,
    name: trimmed,
    targetRegion,
    createdAtUnixMs: timestamp,
    claudeConfigDir: `~/.safecc/profiles/${id}/claude-config`,
    browserProfileDir: `~/.safecc/profiles/${id}/browser-profile`,
    launcherPath: `~/.safecc/profiles/${id}/start-claude.command`,
    browser: "preview",
    browserLabel: "浏览器预览",
    browserAvailable: false,
    previewOnly: true,
    isolation: [
      "Claude CLI 使用独立 CLAUDE_CONFIG_DIR，登录凭证与配置不共享",
      "浏览器使用独立 --user-data-dir，Cookie、站点存储和登录会话不共享",
      "每个档案目录只保存启动元数据，不读取或复制已有凭证内容",
    ],
    warnings: [
      "系统时区是全机设置；不同地区的档案需要切换 Clash 出口并重新配置地区",
      "独立档案不等于独立设备：硬件指纹、浏览器引擎特征和公网 IP 仍可能相关",
      "同一时刻 Clash Verge Rev 只能使用一个活动出口；账号隔离不会创建多个物理网络",
    ],
  };
  window.localStorage.setItem(PREVIEW_PROFILES_KEY, JSON.stringify([...profiles, profile]));
  return profile;
}

export async function openEnvironmentProfile(profileId: string, action: "browser" | "claude" | "folder", confirm: boolean): Promise<void> {
  if (inTauri()) {
    await invoke<void>("open_environment_profile", { profileId, action, confirm });
    return;
  }
  throw new Error("浏览器预览不会启动隔离浏览器或 Claude CLI，请在 Tauri 本机窗口中操作");
}

export type OpenTarget = "verification" | "location" | "claude-settings" | "claude-login" | "clash" | "bandwagon" | "dmit";

export async function openSafeTarget(target: OpenTarget): Promise<void> {
  if (inTauri()) {
    await invoke<void>("open_safe_target", { target });
    return;
  }
  const urls: Record<OpenTarget, string> = {
    verification: "https://ip.net.coffee/claude/",
    location: "https://support.apple.com/guide/mac-help/control-location-information-mac-mh35873/mac",
    "claude-settings": "https://claude.ai/settings",
    "claude-login": "https://claude.ai/login",
    clash: "https://github.com/clash-verge-rev/clash-verge-rev/releases",
    bandwagon: "https://bandwagonhost.com/",
    dmit: "https://www.dmit.io/",
  };
  window.open(urls[target], "_blank", "noopener,noreferrer");
}
