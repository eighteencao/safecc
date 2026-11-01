import { useEffect, useMemo, useState } from "react";
import { applyBrowserEnvironment, applyClashExtension, applyRegionChange, checkVpsReadOnly, clearSafeCache, createClashBackup, deployVps, getBrowserChangePlan, getCachePlan, getClashChangePlan, getPlatformSnapshot, getRegionChangePlan, getRuntimeInfo, importClashSubscription, importLocalClashSubscription, listClashBackups, openSafeTarget, preflightSource, recordVerification, restoreClashBackup, restoreRegionSnapshot, type BrowserApplyResult, type BrowserChangePlan, type CacheCleanupResult, type CachePlan, type ClashApplyResult, type ClashBackupResult, type ClashBackupSummary, type ClashChangePlan, type ClashImportResult, type ClashRestoreResult, type OpenTarget, type PlatformSnapshot, type RegionApplyResult, type RegionChangePlan, type RegionRestoreResult, type SourcePreflightResult, type VerificationRecord, type VpsCheckResult, type VpsDeployResult } from "./native";

type SourceKind = "vps" | "static" | "airport";
type StepId = "source" | "clash" | "local" | "browser" | "cache" | "verify" | "privacy";

type Step = {
  id: StepId;
  index: string;
  eyebrow: string;
  title: string;
  description: string;
};

const steps: Step[] = [
  { id: "source", index: "01", eyebrow: "线路来源", title: "先选你的出口", description: "自建 VPS、静态 IP 或机场订阅，最终都由 Clash Verge Rev 接管。" },
  { id: "clash", index: "02", eyebrow: "Clash Verge Rev", title: "让配置真正生效", description: "在已有订阅配置上写入托管扩展，启用规则模式、TUN、DNS 接管和 UDP 路由。" },
  { id: "local", index: "03", eyebrow: "本机环境", title: "对齐系统环境", description: "设置目标地区的时区、语言与地区格式，保留可回滚的原值。" },
  { id: "browser", index: "04", eyebrow: "默认浏览器", title: "检查浏览器边界", description: "处理语言、定位权限和时区继承；不能稳定自动化的项目会明确引导。" },
  { id: "cache", index: "05", eyebrow: "安全清理", title: "只清理可重建缓存", description: "清除网络与应用缓存，不触碰登录、Cookie、密码、书签和历史。" },
  { id: "verify", index: "06", eyebrow: "真实验收", title: "确认出口没有绕行", description: "打开指定检测页，核对 Claude、DNS、WebRTC UDP、时区和语言。" },
  { id: "privacy", index: "07", eyebrow: "最后一步", title: "登录并关闭 Claude 隐私开关", description: "网络验收通过后，先登录 Claude，再处理定位权限与“协助改进 AI”。" },
];

const sourceOptions: Array<{ id: SourceKind; name: string; caption: string; detail: string }> = [
  { id: "vps", name: "自建 VPS", caption: "你自己的固定出口", detail: "通过 SSH 分阶段部署，生成私有订阅。远端变更需要单独确认。" },
  { id: "static", name: "静态 IP 订阅", caption: "供应商提供的固定线路", detail: "订阅只在本机拉取和解析，不上传到公共转换服务。" },
  { id: "airport", name: "机场订阅", caption: "已有的 Clash / Mihomo 订阅", detail: "保留供应商节点，叠加本机托管的防泄露扩展配置。" },
];

const checks = [
  { label: "Claude / Anthropic 可用性", value: "待检测" },
  { label: "DNS 出口", value: "待检测" },
  { label: "WebRTC UDP 出口", value: "待检测" },
  { label: "时区与语言", value: "待检测" },
];

function App() {
  const [activeStep, setActiveStep] = useState<StepId>("source");
  const [source, setSource] = useState<SourceKind>("airport");
  const [subscription, setSubscription] = useState("");
  const [vpsKeyPath, setVpsKeyPath] = useState("");
  const [vpsPpkPath, setVpsPpkPath] = useState("");
  const [vpsPubPath, setVpsPubPath] = useState("");
  const [vpsProxyHost, setVpsProxyHost] = useState("");
  const [vpsProxyPort, setVpsProxyPort] = useState("1080");
  const [vpsProxyUser, setVpsProxyUser] = useState("");
  const [vpsProxyPassword, setVpsProxyPassword] = useState("");
  const [vpsExpectedIp, setVpsExpectedIp] = useState("");
  const [vpsFingerprintConfirmed, setVpsFingerprintConfirmed] = useState(false);
  const [vpsCheckConsent, setVpsCheckConsent] = useState(false);
  const [vpsCheckResult, setVpsCheckResult] = useState<VpsCheckResult | null>(null);
  const [vpsCheckError, setVpsCheckError] = useState<string | null>(null);
  const [isCheckingVps, setIsCheckingVps] = useState(false);
  const [vpsDeployConsent, setVpsDeployConsent] = useState(false);
  const [vpsDeployResult, setVpsDeployResult] = useState<VpsDeployResult | null>(null);
  const [vpsDeployError, setVpsDeployError] = useState<string | null>(null);
  const [isDeployingVps, setIsDeployingVps] = useState(false);
  const [sourceConsent, setSourceConsent] = useState(false);
  const [sourcePreflight, setSourcePreflight] = useState<SourcePreflightResult | null>(null);
  const [sourcePreflightError, setSourcePreflightError] = useState<string | null>(null);
  const [isPreflightingSource, setIsPreflightingSource] = useState(false);
  const [targetRegion, setTargetRegion] = useState("美国 · 洛杉矶");
  const [isConfigured, setIsConfigured] = useState(false);
  const [isVerified, setIsVerified] = useState(false);
  const [verificationRecord, setVerificationRecord] = useState<VerificationRecord | null>(null);
  const [verificationError, setVerificationError] = useState<string | null>(null);
  const [isRecordingVerification, setIsRecordingVerification] = useState(false);
  const [claudeLoginConfirmed, setClaudeLoginConfirmed] = useState(false);
  const [privacyConfirmed, setPrivacyConfirmed] = useState(false);
  const [configurationCompleted, setConfigurationCompleted] = useState(false);
  const [cacheConfirmed, setCacheConfirmed] = useState(false);
  const [cacheApproval, setCacheApproval] = useState(false);
  const [cachePlan, setCachePlan] = useState<CachePlan | null>(null);
  const [cacheResult, setCacheResult] = useState<CacheCleanupResult | null>(null);
  const [cacheError, setCacheError] = useState<string | null>(null);
  const [isCleaningCache, setIsCleaningCache] = useState(false);
  const [externalOpenError, setExternalOpenError] = useState<string | null>(null);
  const [expanded, setExpanded] = useState<string | null>(null);
  const [runtimeLabel, setRuntimeLabel] = useState("浏览器预览");
  const [platformSnapshot, setPlatformSnapshot] = useState<PlatformSnapshot | null>(null);
  const [clashPlan, setClashPlan] = useState<ClashChangePlan | null>(null);
  const [backupConfirmed, setBackupConfirmed] = useState(false);
  const [backupResult, setBackupResult] = useState<ClashBackupResult | null>(null);
  const [backupError, setBackupError] = useState<string | null>(null);
  const [isBackingUp, setIsBackingUp] = useState(false);
  const [clashApplyResult, setClashApplyResult] = useState<ClashApplyResult | null>(null);
  const [clashApplyError, setClashApplyError] = useState<string | null>(null);
  const [clashImportResult, setClashImportResult] = useState<ClashImportResult | null>(null);
  const [clashImportError, setClashImportError] = useState<string | null>(null);
  const [backups, setBackups] = useState<ClashBackupSummary[]>([]);
  const [selectedBackupId, setSelectedBackupId] = useState("");
  const [restoreConfirmed, setRestoreConfirmed] = useState(false);
  const [restoreResult, setRestoreResult] = useState<ClashRestoreResult | null>(null);
  const [restoreError, setRestoreError] = useState<string | null>(null);
  const [isRestoring, setIsRestoring] = useState(false);
  const [regionPlan, setRegionPlan] = useState<RegionChangePlan | null>(null);
  const [regionConfirmed, setRegionConfirmed] = useState(false);
  const [regionResult, setRegionResult] = useState<RegionApplyResult | null>(null);
  const [regionError, setRegionError] = useState<string | null>(null);
  const [isApplyingRegion, setIsApplyingRegion] = useState(false);
  const [localApplied, setLocalApplied] = useState(false);
  const [regionRestoreConfirmed, setRegionRestoreConfirmed] = useState(false);
  const [regionRestoreResult, setRegionRestoreResult] = useState<RegionRestoreResult | null>(null);
  const [regionRestoreError, setRegionRestoreError] = useState<string | null>(null);
  const [isRestoringRegion, setIsRestoringRegion] = useState(false);
  const [browserPlan, setBrowserPlan] = useState<BrowserChangePlan | null>(null);
  const [browserConfirmed, setBrowserConfirmed] = useState(false);
  const [browserResult, setBrowserResult] = useState<BrowserApplyResult | null>(null);
  const [browserError, setBrowserError] = useState<string | null>(null);
  const [isApplyingBrowser, setIsApplyingBrowser] = useState(false);
  const [browserApplied, setBrowserApplied] = useState(false);
  const [quickConfigConfirmed, setQuickConfigConfirmed] = useState(false);
  const [isQuickConfiguring, setIsQuickConfiguring] = useState(false);
  const [quickConfigStage, setQuickConfigStage] = useState<string | null>(null);
  const [quickConfigError, setQuickConfigError] = useState<string | null>(null);

  const refreshBackups = async () => {
    try {
      setBackups(await listClashBackups());
    } catch {
      setBackups([]);
    }
  };

  useEffect(() => {
    getRuntimeInfo()
      .then((info) => setRuntimeLabel(info.runtime === "tauri" ? `Tauri 本机 · ${info.platform}` : "浏览器预览"))
      .catch(() => setRuntimeLabel("浏览器预览"));
    getPlatformSnapshot().then(setPlatformSnapshot).catch(() => setPlatformSnapshot(null));
    getCachePlan().then(setCachePlan).catch(() => setCachePlan(null));
    refreshBackups();
  }, []);

  useEffect(() => {
    getClashChangePlan(source, targetRegion).then(setClashPlan).catch(() => setClashPlan(null));
  }, [source, targetRegion]);

  useEffect(() => {
    getRegionChangePlan(targetRegion).then(setRegionPlan).catch(() => setRegionPlan(null));
  }, [targetRegion]);

  useEffect(() => {
    getBrowserChangePlan(targetRegion, platformSnapshot?.defaultBrowser ?? null).then(setBrowserPlan).catch(() => setBrowserPlan(null));
  }, [platformSnapshot, targetRegion]);

  useEffect(() => {
    setSourceConsent(false);
    setSourcePreflight(null);
    setSourcePreflightError(null);
    setVpsFingerprintConfirmed(false);
    setVpsCheckConsent(false);
    setVpsCheckResult(null);
    setVpsCheckError(null);
    setVpsDeployConsent(false);
    setVpsDeployResult(null);
    setVpsDeployError(null);
  }, [source, subscription]);

  useEffect(() => {
    setVpsCheckResult(null);
    setVpsCheckError(null);
    setVpsDeployConsent(false);
    setVpsDeployResult(null);
    setVpsDeployError(null);
  }, [vpsExpectedIp, vpsKeyPath, vpsPpkPath, vpsProxyHost, vpsProxyPassword, vpsProxyPort, vpsProxyUser, vpsPubPath]);

  useEffect(() => {
    setIsConfigured(false);
    setBackupConfirmed(false);
    setBackupResult(null);
    setBackupError(null);
    setClashApplyResult(null);
    setClashApplyError(null);
    setClashImportResult(null);
    setClashImportError(null);
    setIsVerified(false);
    setVerificationRecord(null);
    setVerificationError(null);
    setClaudeLoginConfirmed(false);
    setPrivacyConfirmed(false);
    setConfigurationCompleted(false);
    setLocalApplied(false);
    setRegionConfirmed(false);
    setRegionResult(null);
    setRegionError(null);
    setRegionRestoreConfirmed(false);
    setRegionRestoreResult(null);
    setRegionRestoreError(null);
    setBrowserApplied(false);
    setBrowserConfirmed(false);
    setBrowserResult(null);
    setBrowserError(null);
    setCacheConfirmed(false);
    setCacheApproval(false);
    setCacheResult(null);
    setCacheError(null);
    setQuickConfigConfirmed(false);
    setQuickConfigStage(null);
    setQuickConfigError(null);
  }, [source, targetRegion]);

  const handleClashConfigure = async () => {
    if (!backupConfirmed || isBackingUp) return;
    setBackupError(null);
    setIsBackingUp(true);
    try {
      const result = await createClashBackup(true);
      setBackupResult(result);
      let importCompleted = Boolean(clashImportResult);
      try {
        if (sourcePreflight?.status === "valid" && sourcePreflight.sourceId && !clashImportResult) {
          const importResult = await importClashSubscription(source, targetRegion, sourcePreflight.sourceId, subscription, true);
          setClashImportResult(importResult);
          importCompleted = true;
        } else if (source === "vps" && vpsDeployResult?.subscriptionSnapshotId && !clashImportResult) {
          const importResult = await importLocalClashSubscription(targetRegion, vpsDeployResult.subscriptionSnapshotId, true);
          setClashImportResult(importResult);
          importCompleted = true;
        }
        const applyResult = await applyClashExtension(source, targetRegion, true);
        setClashApplyResult(applyResult);
        setIsConfigured(true);
      } catch (error) {
        if ((sourcePreflight?.status === "valid" || (source === "vps" && vpsDeployResult?.subscriptionSnapshotId)) && !importCompleted) {
          setClashImportError(error instanceof Error ? error.message : "导入订阅到 Clash Verge Rev 失败，请恢复或检查权限。");
        } else {
          setClashApplyError(error instanceof Error ? error.message : "写入 Clash 托管扩展失败，请恢复或检查权限。");
        }
      }
      await refreshBackups();
    } catch (error) {
      setBackupError(error instanceof Error ? error.message : "创建本地备份失败，请检查权限和 Clash 数据目录。");
    } finally {
      setIsBackingUp(false);
    }
  };

  const handleSourcePreflight = async () => {
    if (!sourceConsent || !subscription.trim() || isPreflightingSource) return;
    setSourcePreflightError(null);
    setIsPreflightingSource(true);
    try {
      const result = await preflightSource(source, subscription, true, source === "vps" ? (vpsKeyPath.trim() || null) : null);
      setSourcePreflight(result);
    } catch (error) {
      setSourcePreflightError(error instanceof Error ? error.message : "线路预检失败，请检查地址和网络。");
    } finally {
      setIsPreflightingSource(false);
    }
  };

  const handleVpsCheck = async () => {
    if (source !== "vps" || !sourcePreflight?.sourceId || !vpsFingerprintConfirmed || !vpsCheckConsent || !vpsKeyPath.trim() || isCheckingVps) return;
    setVpsCheckError(null);
    setIsCheckingVps(true);
    try {
      setVpsCheckResult(await checkVpsReadOnly(subscription, vpsKeyPath.trim(), sourcePreflight.sourceId, true));
    } catch (error) {
      setVpsCheckError(error instanceof Error ? error.message : "VPS 只读检查失败，未执行远端变更。");
    } finally {
      setIsCheckingVps(false);
    }
  };

  const handleVpsDeploy = async () => {
    if (source !== "vps" || !sourcePreflight?.sourceId || !vpsCheckResult || !vpsDeployConsent || !vpsKeyPath.trim() || isDeployingVps) return;
    setVpsDeployError(null);
    setIsDeployingVps(true);
    try {
      setVpsDeployResult(await deployVps({
        targetInput: subscription.trim(),
        pemPath: vpsKeyPath.trim(),
        ppkPath: vpsPpkPath.trim(),
        pubPath: vpsPubPath.trim(),
        proxyHost: vpsProxyHost.trim(),
        proxyPort: Number(vpsProxyPort) || 0,
        proxyUser: vpsProxyUser.trim(),
        proxyPassword: vpsProxyPassword,
        expectedIp: vpsExpectedIp.trim(),
        hostKeyId: sourcePreflight.sourceId,
        confirm: true,
      }));
    } catch (error) {
      setVpsDeployError(error instanceof Error ? error.message : "VPS 部署失败；未确认远端结果。请检查参数和权限。");
    } finally {
      setIsDeployingVps(false);
    }
  };

  const handleRestore = async () => {
    if (!selectedBackupId || !restoreConfirmed || isRestoring) return;
    setRestoreError(null);
    setIsRestoring(true);
    try {
      const result = await restoreClashBackup(selectedBackupId, true);
      setRestoreResult(result);
      setIsConfigured(true);
      setRestoreConfirmed(false);
      await refreshBackups();
    } catch (error) {
      setRestoreError(error instanceof Error ? error.message : "恢复配置失败，请保留当前状态并检查权限。");
    } finally {
      setIsRestoring(false);
    }
  };

  const handleApplyRegion = async () => {
    if (!regionConfirmed || isApplyingRegion) return;
    setRegionError(null);
    setIsApplyingRegion(true);
    try {
      const result = await applyRegionChange(targetRegion, true);
      setRegionResult(result);
      setLocalApplied(true);
    } catch (error) {
      setRegionError(error instanceof Error ? error.message : "系统环境配置失败，请检查权限。");
    } finally {
      setIsApplyingRegion(false);
    }
  };

  const handleRestoreRegion = async () => {
    if (!regionResult?.snapshotId || !regionRestoreConfirmed || isRestoringRegion) return;
    setRegionRestoreError(null);
    setIsRestoringRegion(true);
    try {
      const result = await restoreRegionSnapshot(regionResult.snapshotId, true);
      setRegionRestoreResult(result);
      setRegionRestoreConfirmed(false);
    } catch (error) {
      setRegionRestoreError(error instanceof Error ? error.message : "恢复地区快照失败，请检查权限。");
    } finally {
      setIsRestoringRegion(false);
    }
  };

  const handleCacheCleanup = async () => {
    if (!cacheApproval || isCleaningCache) return;
    setCacheError(null);
    setIsCleaningCache(true);
    try {
      const result = await clearSafeCache(true);
      setCacheResult(result);
      setCacheConfirmed(true);
    } catch (error) {
      setCacheError(error instanceof Error ? error.message : "安全缓存清理失败，请检查权限。");
    } finally {
      setIsCleaningCache(false);
    }
  };

  const handleApplyBrowser = async () => {
    if (!browserConfirmed || isApplyingBrowser) return;
    if (!browserPlan?.automatic) {
      setBrowserResult({
        status: "preview-only",
        browser: browserPlan?.browser ?? "unknown",
        changed: [],
        backupId: null,
        requiresRestart: false,
        warnings: ["当前浏览器没有安全的自动写入入口，已记录用户完成人工检查"],
      });
      setBrowserApplied(true);
      return;
    }
    setBrowserError(null);
    setIsApplyingBrowser(true);
    try {
      const result = await applyBrowserEnvironment(targetRegion, platformSnapshot?.defaultBrowser ?? null, true);
      setBrowserResult(result);
      setBrowserApplied(true);
    } catch (error) {
      setBrowserError(error instanceof Error ? error.message : "浏览器环境配置失败，请检查浏览器是否已完全退出。");
    } finally {
      setIsApplyingBrowser(false);
    }
  };

  const handleQuickConfigure = async () => {
    const sourceReady = Boolean(
      sourcePreflight
        && (sourcePreflight.status === "preview-only"
          || (sourcePreflight.status === "valid"
            && sourcePreflight.sourceId
            && (source !== "vps" || vpsDeployResult?.subscriptionSnapshotId))
          || (source === "vps"
            && sourcePreflight.status === "manual"
            && sourcePreflight.sourceId
            && vpsDeployResult?.subscriptionSnapshotId)),
    );
    if (!quickConfigConfirmed || !sourceReady || isQuickConfiguring) return;

    setQuickConfigError(null);
    setIsQuickConfiguring(true);
    try {
      setQuickConfigStage("正在备份 Clash Verge Rev 配置");
      const backup = await createClashBackup(true);
      setBackupResult(backup);

      let importCompleted = Boolean(clashImportResult);
      const sourceId = sourcePreflight?.sourceId;
      const canImportQuickConfigSource = Boolean(
        sourceId
          && (sourcePreflight.status === "valid"
            || (source === "vps" && sourcePreflight.status === "manual" && vpsDeployResult?.subscriptionSnapshotId)),
      );
      if (!importCompleted && sourceId && canImportQuickConfigSource) {
        setQuickConfigStage("正在登记订阅到 Clash Verge Rev");
        const imported = source === "vps" && vpsDeployResult?.subscriptionSnapshotId
          ? await importLocalClashSubscription(targetRegion, vpsDeployResult.subscriptionSnapshotId, true)
          : await importClashSubscription(source, targetRegion, sourceId, subscription, true);
        setClashImportResult(imported);
        importCompleted = true;
      }

      setQuickConfigStage("正在写入 Clash 防泄露托管扩展");
      const clash = await applyClashExtension(source, targetRegion, true);
      setClashApplyResult(clash);
      setIsConfigured(true);

      setQuickConfigStage("正在请求系统权限并配置地区");
      const region = await applyRegionChange(targetRegion, true);
      setRegionResult(region);
      setLocalApplied(true);

      setQuickConfigStage("正在配置默认浏览器");
      if (!browserPlan?.automatic) {
        setBrowserResult({
          status: "preview-only",
          browser: browserPlan?.browser ?? "unknown",
          changed: [],
          backupId: null,
          requiresRestart: false,
          warnings: ["当前浏览器没有安全的自动写入入口，已保留人工确认步骤"],
        });
        setQuickConfigStage("默认浏览器需要手动确认，请完成浏览器步骤后继续");
        setActiveStep("browser");
        await refreshBackups();
        return;
      }
      const browser = await applyBrowserEnvironment(targetRegion, platformSnapshot?.defaultBrowser ?? null, true);
      setBrowserResult(browser);
      setBrowserApplied(true);

      setQuickConfigStage("正在清理标准缓存并刷新 DNS");
      const cache = await clearSafeCache(true);
      setCacheResult(cache);
      setCacheConfirmed(true);
      setQuickConfigStage("本机配置完成，请进入真实验收");
      setActiveStep("verify");
      await refreshBackups();
    } catch (error) {
      setQuickConfigError(error instanceof Error ? error.message : "一键配置在当前阶段停止，请按提示修复后重试。未跳过后续步骤。" );
    } finally {
      setIsQuickConfiguring(false);
    }
  };

  const handleOpenTarget = async (target: OpenTarget) => {
    setExternalOpenError(null);
    try {
      await openSafeTarget(target);
    } catch (error) {
      setExternalOpenError(error instanceof Error ? error.message : "无法打开对应设置页面。");
    }
  };

  const handleVerify = async () => {
    if (isRecordingVerification) return;
    setVerificationError(null);
    setIsRecordingVerification(true);
    try {
      const routeAlias = source === "vps"
        ? "VPS 私有线路"
        : source === "static"
          ? "静态 IP 线路"
          : "机场订阅线路";
      const record = await recordVerification(targetRegion, `${routeAlias} · ${targetRegion}`, ["claude", "dns", "webrtc", "locale"], true);
      setVerificationRecord(record);
      setIsVerified(true);
    } catch (error) {
      setVerificationError(error instanceof Error ? error.message : "无法保存验收记录，请保留页面结果并重试。" );
    } finally {
      setIsRecordingVerification(false);
    }
  };

  const activeIndex = steps.findIndex((step) => step.id === activeStep);
  const selectedSource = sourceOptions.find((item) => item.id === source)!;
  const quickConfigReady = Boolean(
    sourcePreflight
      && (sourcePreflight.status === "preview-only"
        || (sourcePreflight.status === "valid"
          && sourcePreflight.sourceId
          && (source !== "vps" || vpsDeployResult?.subscriptionSnapshotId))
        || (source === "vps"
          && sourcePreflight.status === "manual"
          && sourcePreflight.sourceId
          && vpsDeployResult?.subscriptionSnapshotId)),
  );

  const progress = useMemo(() => {
    const completed = [isConfigured, localApplied, browserApplied, cacheConfirmed, isVerified, privacyConfirmed].filter(Boolean).length;
    return Math.round((completed / 6) * 100);
  }, [browserApplied, cacheConfirmed, isConfigured, isVerified, localApplied, privacyConfirmed]);

  const canAdvance = (id: StepId) => {
    if (id === "source") return Boolean(source && subscription.trim() && sourcePreflight && (source !== "vps" || sourcePreflight.status === "preview-only" || (vpsFingerprintConfirmed && vpsCheckResult && vpsDeployResult)));
    if (id === "clash") return isConfigured;
    if (id === "local") return localApplied;
    if (id === "browser") return browserApplied;
    if (id === "cache") return cacheConfirmed;
    if (id === "verify") return isVerified;
    if (id === "privacy") return claudeLoginConfirmed && privacyConfirmed;
    return true;
  };

  const move = (direction: -1 | 1) => {
    if (direction === 1 && activeIndex === steps.length - 1) {
      if (canAdvance(activeStep)) setConfigurationCompleted(true);
      return;
    }
    const next = Math.min(steps.length - 1, Math.max(0, activeIndex + direction));
    setActiveStep(steps[next].id);
  };

  return (
    <main className="app-shell">
      <aside className="sidebar">
        <div className="brand-lockup">
          <div className="brand-mark">C</div>
          <div>
            <strong>safecc</strong>
            <span>Claude 环境安全助手</span>
          </div>
        </div>

        <div className="sidebar-label">配置流程</div>
        <nav className="step-list" aria-label="配置流程">
          {steps.map((step, index) => {
            const done = step.id === "clash" ? isConfigured : step.id === "local" ? localApplied : step.id === "browser" ? browserApplied : step.id === "cache" ? cacheConfirmed : step.id === "verify" ? isVerified : step.id === "privacy" ? privacyConfirmed : index < activeIndex;
            const locked = step.id === "privacy" && !isVerified;
            return (
              <button className={`step-item ${activeStep === step.id ? "active" : ""} ${locked ? "locked" : ""}`} key={step.id} onClick={() => setActiveStep(step.id)} disabled={locked} aria-label={locked ? `${step.title}（完成网络验收后解锁）` : step.title}>
                <span className={`step-dot ${done ? "done" : ""}`}>{done ? "✓" : locked ? "·" : step.index}</span>
                <span className="step-copy"><b>{step.eyebrow}</b><small>{step.title}</small></span>
                {activeStep === step.id && <span className="step-arrow">↗</span>}
              </button>
            );
          })}
        </nav>

        <div className="sidebar-footer">
          <div className="progress-label"><span>当前进度</span><strong>{progress}%</strong></div>
          <div className="progress-track"><span style={{ width: `${Math.max(8, progress)}%` }} /></div>
          <p>隐私配置始终放在网络验收之后。</p>
        </div>
      </aside>

      <section className="workspace">
        <header className="topbar">
          <div className="breadcrumb"><span>工作区</span><i>/</i><strong>{steps[activeIndex].eyebrow}</strong></div>
          <div className="topbar-actions"><span className="platform-pill"><span className="status-dot" /> {runtimeLabel}</span><button className="icon-button" aria-label="帮助">?</button></div>
        </header>

        <div className="content-wrap">
          <div className="hero-row">
            <div><p className="kicker">{steps[activeIndex].index} / {String(steps.length).padStart(2, "0")} · {steps[activeIndex].eyebrow}</p><h1>{steps[activeIndex].title}</h1><p className="hero-description">{steps[activeIndex].description}</p></div>
            <div className="hero-note"><span className="note-icon">i</span><span>本工具不会保证账号不受限制；它只报告可测量的环境事实。</span></div>
          </div>

          {activeStep === "source" && <SourceStep source={source} setSource={setSource} selectedSource={selectedSource} subscription={subscription} setSubscription={setSubscription} vpsKeyPath={vpsKeyPath} setVpsKeyPath={setVpsKeyPath} vpsPpkPath={vpsPpkPath} setVpsPpkPath={setVpsPpkPath} vpsPubPath={vpsPubPath} setVpsPubPath={setVpsPubPath} vpsProxyHost={vpsProxyHost} setVpsProxyHost={setVpsProxyHost} vpsProxyPort={vpsProxyPort} setVpsProxyPort={setVpsProxyPort} vpsProxyUser={vpsProxyUser} setVpsProxyUser={setVpsProxyUser} vpsProxyPassword={vpsProxyPassword} setVpsProxyPassword={setVpsProxyPassword} vpsExpectedIp={vpsExpectedIp} setVpsExpectedIp={setVpsExpectedIp} fingerprintConfirmed={vpsFingerprintConfirmed} setFingerprintConfirmed={setVpsFingerprintConfirmed} vpsCheckConsent={vpsCheckConsent} setVpsCheckConsent={setVpsCheckConsent} vpsCheckResult={vpsCheckResult} vpsCheckError={vpsCheckError} isCheckingVps={isCheckingVps} onVpsCheck={handleVpsCheck} vpsDeployConsent={vpsDeployConsent} setVpsDeployConsent={setVpsDeployConsent} vpsDeployResult={vpsDeployResult} vpsDeployError={vpsDeployError} isDeployingVps={isDeployingVps} onVpsDeploy={handleVpsDeploy} targetRegion={targetRegion} setTargetRegion={setTargetRegion} consent={sourceConsent} setConsent={setSourceConsent} preflight={sourcePreflight} error={sourcePreflightError} isPreflighting={isPreflightingSource} onPreflight={handleSourcePreflight} quickConfigReady={quickConfigReady} quickConfigConfirmed={quickConfigConfirmed} setQuickConfigConfirmed={setQuickConfigConfirmed} isQuickConfiguring={isQuickConfiguring} quickConfigStage={quickConfigStage} quickConfigError={quickConfigError} onQuickConfigure={handleQuickConfigure} onOpenProvider={(target) => handleOpenTarget(target)} />}
          {activeStep === "clash" && <ClashStep isConfigured={isConfigured} plan={clashPlan} backupConfirmed={backupConfirmed} setBackupConfirmed={setBackupConfirmed} backupResult={backupResult} backupError={backupError} clashImportResult={clashImportResult} clashImportError={clashImportError} clashApplyResult={clashApplyResult} clashApplyError={clashApplyError} isBackingUp={isBackingUp} backups={backups} selectedBackupId={selectedBackupId} setSelectedBackupId={setSelectedBackupId} restoreConfirmed={restoreConfirmed} setRestoreConfirmed={setRestoreConfirmed} restoreResult={restoreResult} restoreError={restoreError} isRestoring={isRestoring} onConfigure={handleClashConfigure} onRestore={handleRestore} onOpenClash={() => handleOpenTarget("clash")} />}
          {activeStep === "local" && <LocalStep targetRegion={targetRegion} snapshot={platformSnapshot} plan={regionPlan} confirmed={regionConfirmed} setConfirmed={setRegionConfirmed} result={regionResult} error={regionError} isApplying={isApplyingRegion} applied={localApplied} onApply={handleApplyRegion} restoreConfirmed={regionRestoreConfirmed} setRestoreConfirmed={setRegionRestoreConfirmed} restoreResult={regionRestoreResult} restoreError={regionRestoreError} isRestoring={isRestoringRegion} onRestore={handleRestoreRegion} />}
          {activeStep === "browser" && <BrowserStep plan={browserPlan} confirmed={browserConfirmed} setConfirmed={setBrowserConfirmed} result={browserResult} error={browserError} isApplying={isApplyingBrowser} applied={browserApplied} expanded={expanded} setExpanded={setExpanded} onApply={handleApplyBrowser} />}
          {activeStep === "cache" && <CacheStep plan={cachePlan} approval={cacheApproval} setApproval={setCacheApproval} result={cacheResult} error={cacheError} isCleaning={isCleaningCache} confirmed={cacheConfirmed} onClean={handleCacheCleanup} />}
          {externalOpenError && <p className="error-note global-error">{externalOpenError}</p>}
          {activeStep === "verify" && <VerifyStep targetRegion={targetRegion} verified={isVerified} record={verificationRecord} error={verificationError} isRecording={isRecordingVerification} onOpen={() => handleOpenTarget("verification")} onVerify={handleVerify} />}
          {activeStep === "privacy" && <PrivacyStep loggedIn={claudeLoginConfirmed} confirmed={privacyConfirmed} completed={configurationCompleted} onOpenLogin={() => handleOpenTarget("claude-login")} onConfirmLogin={(value) => { setClaudeLoginConfirmed(value); if (!value) { setPrivacyConfirmed(false); setConfigurationCompleted(false); } }} onOpenLocation={() => handleOpenTarget("location")} onOpenClaude={() => handleOpenTarget("claude-settings")} onConfirm={(value) => { if (claudeLoginConfirmed) { setPrivacyConfirmed(value); if (!value) setConfigurationCompleted(false); } }} />}

          <footer className="action-bar">
            <button className="text-button" onClick={() => move(-1)} disabled={activeIndex === 0}>← 上一步</button>
            <div className="action-right"><span className="save-hint">{configurationCompleted ? "safecc 配置流程已完成" : "快照与验收记录保存在本机"}</span><button className="primary-button" onClick={() => move(1)} disabled={!canAdvance(activeStep) || configurationCompleted}>{activeStep === "privacy" ? configurationCompleted ? "已完成" : "完成配置" : "继续"}<span>→</span></button></div>
          </footer>
        </div>
      </section>
    </main>
  );
}

function SourceStep({ source, setSource, selectedSource, subscription, setSubscription, vpsKeyPath, setVpsKeyPath, vpsPpkPath, setVpsPpkPath, vpsPubPath, setVpsPubPath, vpsProxyHost, setVpsProxyHost, vpsProxyPort, setVpsProxyPort, vpsProxyUser, setVpsProxyUser, vpsProxyPassword, setVpsProxyPassword, vpsExpectedIp, setVpsExpectedIp, fingerprintConfirmed, setFingerprintConfirmed, vpsCheckConsent, setVpsCheckConsent, vpsCheckResult, vpsCheckError, isCheckingVps, onVpsCheck, vpsDeployConsent, setVpsDeployConsent, vpsDeployResult, vpsDeployError, isDeployingVps, onVpsDeploy, targetRegion, setTargetRegion, consent, setConsent, preflight, error, isPreflighting, onPreflight, quickConfigReady, quickConfigConfirmed, setQuickConfigConfirmed, isQuickConfiguring, quickConfigStage, quickConfigError, onQuickConfigure, onOpenProvider }: { source: SourceKind; setSource: (value: SourceKind) => void; selectedSource: (typeof sourceOptions)[number]; subscription: string; setSubscription: (value: string) => void; vpsKeyPath: string; setVpsKeyPath: (value: string) => void; vpsPpkPath: string; setVpsPpkPath: (value: string) => void; vpsPubPath: string; setVpsPubPath: (value: string) => void; vpsProxyHost: string; setVpsProxyHost: (value: string) => void; vpsProxyPort: string; setVpsProxyPort: (value: string) => void; vpsProxyUser: string; setVpsProxyUser: (value: string) => void; vpsProxyPassword: string; setVpsProxyPassword: (value: string) => void; vpsExpectedIp: string; setVpsExpectedIp: (value: string) => void; fingerprintConfirmed: boolean; setFingerprintConfirmed: (value: boolean) => void; vpsCheckConsent: boolean; setVpsCheckConsent: (value: boolean) => void; vpsCheckResult: VpsCheckResult | null; vpsCheckError: string | null; isCheckingVps: boolean; onVpsCheck: () => void; vpsDeployConsent: boolean; setVpsDeployConsent: (value: boolean) => void; vpsDeployResult: VpsDeployResult | null; vpsDeployError: string | null; isDeployingVps: boolean; onVpsDeploy: () => void; targetRegion: string; setTargetRegion: (value: string) => void; consent: boolean; setConsent: (value: boolean) => void; preflight: SourcePreflightResult | null; error: string | null; isPreflighting: boolean; onPreflight: () => void; quickConfigReady: boolean; quickConfigConfirmed: boolean; setQuickConfigConfirmed: (value: boolean) => void; isQuickConfiguring: boolean; quickConfigStage: string | null; quickConfigError: string | null; onQuickConfigure: () => void; onOpenProvider: (target: Extract<OpenTarget, "bandwagon" | "dmit">) => void }) {
  return <div className="grid-layout">
    <section className="card card-large"><div className="card-heading"><div><span className="section-number">A</span><div><h2>选择线路来源</h2><p>一次只激活一条线路，之后可以随时切换。</p></div></div><span className="safe-badge">仅本机处理</span></div>
      <div className="provider-links"><div className="provider-links-heading"><span>购买 VPS / 获取线路</span><small>打开官方页面，safecc 不代购、不填写付款信息</small></div><div className="provider-link-grid"><button className="provider-link-card" onClick={() => onOpenProvider("bandwagon")}><span className="provider-link-mark">B</span><span><b>搬瓦工 · BandwagonHost</b><small>官方 VPS 入口</small></span><span className="provider-link-arrow">↗</span></button><button className="provider-link-card" onClick={() => onOpenProvider("dmit")}><span className="provider-link-mark">D</span><span><b>DMIT</b><small>官方 VPS 入口</small></span><span className="provider-link-arrow">↗</span></button></div></div>
      <div className="source-grid">{sourceOptions.map((item) => <button key={item.id} onClick={() => setSource(item.id)} className={`source-card ${source === item.id ? "selected" : ""}`}><span className="source-icon">{item.id === "vps" ? "⌁" : item.id === "static" ? "▣" : "◌"}</span><span><b>{item.name}</b><small>{item.caption}</small></span><span className="radio">{source === item.id ? "●" : "○"}</span></button>)}</div>
      <div className="form-divider" />
      <label className="field-label">{source === "vps" ? "服务器信息" : "订阅地址"}<span>必填</span></label>
      <div className="input-with-status"><input type={source === "vps" ? "text" : "password"} autoComplete="off" value={subscription} onChange={(event) => setSubscription(event.target.value)} placeholder={source === "vps" ? "输入服务器 IP 或 SSH 别名" : "粘贴 Clash / Mihomo 订阅 URL"} /><span className="input-state">{subscription ? "已填写" : "等待输入"}</span></div>
      {source === "vps" && <div className="input-with-status"><input type="text" autoComplete="off" value={vpsKeyPath} onChange={(event) => setVpsKeyPath(event.target.value)} placeholder="本机 PEM 私钥路径" /><span className="input-state">{vpsKeyPath ? "已填写" : "待补充"}</span></div>}
      {source === "vps" && <>
        <div className="input-with-status"><input type="text" autoComplete="off" value={vpsPpkPath} onChange={(event) => setVpsPpkPath(event.target.value)} placeholder="部署所需 .ppk 文件绝对路径" /><span className="input-state">{vpsPpkPath ? "已填写" : "待补充"}</span></div>
        <div className="input-with-status"><input type="text" autoComplete="off" value={vpsPubPath} onChange={(event) => setVpsPubPath(event.target.value)} placeholder="部署所需 .pub 文件绝对路径" /><span className="input-state">{vpsPubPath ? "已填写" : "待补充"}</span></div>
        <div className="form-divider" />
        <label className="field-label">静态住宅 / ISP SOCKS5 出口<span>部署必填</span></label>
        <div className="input-with-status"><input type="text" autoComplete="off" value={vpsProxyHost} onChange={(event) => setVpsProxyHost(event.target.value)} placeholder="代理主机或 IP" /><span className="input-state">{vpsProxyHost ? "已填写" : "待补充"}</span></div>
        <div className="split-fields"><input type="number" min="1" max="65535" value={vpsProxyPort} onChange={(event) => setVpsProxyPort(event.target.value)} placeholder="端口" /><input type="text" autoComplete="off" value={vpsProxyUser} onChange={(event) => setVpsProxyUser(event.target.value)} placeholder="代理用户名" /></div>
        <div className="input-with-status"><input type="password" autoComplete="new-password" value={vpsProxyPassword} onChange={(event) => setVpsProxyPassword(event.target.value)} placeholder="代理密码（仅在本机受保护配置中使用）" /><span className="input-state">{vpsProxyPassword ? "已填写" : "待补充"}</span></div>
        <div className="input-with-status"><input type="text" autoComplete="off" value={vpsExpectedIp} onChange={(event) => setVpsExpectedIp(event.target.value)} placeholder="供应商承诺的固定出口 IP" /><span className="input-state">{vpsExpectedIp ? "已填写" : "待补充"}</span></div>
      </>}
      <p className="field-help">{selectedSource.detail}</p>
      <label className="confirm-line"><input type="checkbox" checked={consent} onChange={(event) => setConsent(event.target.checked)} /><span>{source === "vps" ? "我确认只读取 SSH 主机指纹，不登录服务器、不执行远端命令。" : "我确认只在本机通过 HTTPS 预检订阅，并保存受保护的本地快照。"}</span></label>
      <button className="secondary-button" onClick={onPreflight} disabled={!subscription.trim() || !consent || isPreflighting}>{isPreflighting ? "正在本机预检线路…" : preflight ? "重新预检线路" : "确认并预检线路"}<span>↗</span></button>
      {preflight && <div className="backup-result"><span className="check-icon passed">✓</span><div><b>{preflight.status === "valid" ? "线路预检通过" : preflight.status === "manual" ? "VPS 主机指纹已读取" : "浏览器预览：未请求订阅"}</b><small>{preflight.status === "valid" ? `${preflight.format} · ${preflight.nodeCount || "未知"} 个节点 · ${preflight.bytes} bytes；已保存本地快照 ${preflight.sourceId}` : preflight.warnings[0]}</small>{preflight.fingerprint && <code className="fingerprint-value">SSH {preflight.fingerprint}</code>}</div></div>}
      {source === "vps" && preflight?.fingerprint && <label className="confirm-line"><input type="checkbox" checked={fingerprintConfirmed} onChange={(event) => setFingerprintConfirmed(event.target.checked)} /><span>我已把上面的 SHA-256 指纹与可信 VPS 控制台或既有主机记录比对。</span></label>}
      {source === "vps" && preflight?.fingerprint && <><label className="confirm-line"><input type="checkbox" checked={vpsCheckConsent} onChange={(event) => setVpsCheckConsent(event.target.checked)} /><span>我确认允许一次 SSH 只读检查；不会安装软件、修改防火墙或写入远端文件。</span></label><button className="secondary-button" onClick={onVpsCheck} disabled={!fingerprintConfirmed || !vpsCheckConsent || !vpsKeyPath.trim() || isCheckingVps}>{isCheckingVps ? "正在执行 SSH 只读检查…" : vpsCheckResult ? "重新执行 SSH 只读检查" : "执行 SSH 只读检查"}<span>↗</span></button></>}
      {vpsCheckResult && <div className="backup-result"><span className="check-icon passed">✓</span><div><b>VPS 只读检查通过</b><small>{vpsCheckResult.operatingSystem} · {vpsCheckResult.user}@{vpsCheckResult.host}:{vpsCheckResult.port}；未执行远端变更</small></div></div>}
      {source === "vps" && vpsCheckResult && <>
        <label className="confirm-line"><input type="checkbox" checked={vpsDeployConsent} onChange={(event) => setVpsDeployConsent(event.target.checked)} /><span>我已核对上面的部署参数，并明确批准远端变更：安装系统依赖、配置 SSH / 防火墙 / Xray，失败时可能留下部分改动。</span></label>
        <button className="secondary-button" onClick={onVpsDeploy} disabled={!vpsDeployConsent || !vpsPpkPath.trim() || !vpsPubPath.trim() || !vpsProxyHost.trim() || !vpsProxyUser.trim() || !vpsProxyPassword || !vpsExpectedIp.trim() || isDeployingVps}>{isDeployingVps ? "正在执行已批准的 VPS 部署…" : vpsDeployResult ? "重新执行 VPS 部署" : "批准并部署 VPS"}<span>↗</span></button>
      </>}
      {vpsDeployResult && <div className="backup-result"><span className="check-icon passed">✓</span><div><b>VPS 部署阶段已完成</b><small>本机已取回私有 Clash YAML 快照 {vpsDeployResult.subscriptionSnapshotId ?? "—"}；请在 Clash Verge Rev 步骤导入并复核运行态</small></div></div>}
      {vpsDeployError && <p className="error-note">{vpsDeployError}</p>}
      {vpsCheckError && <p className="error-note">{vpsCheckError}</p>}
      {error && <p className="error-note">{error}</p>}
      <div className="permission-box quick-config-box"><b>一键配置本机</b><p>线路预检通过后，safecc 会按“Clash → 系统地区 → 默认浏览器 → 标准缓存”的顺序执行。系统时区/UAC 权限仍会在真正写入时由系统弹窗请求；不会自动执行 VPS 远端部署，也不会跳过真实验收和最后的 Claude 隐私步骤。</p><label className="confirm-line"><input type="checkbox" checked={quickConfigConfirmed} onChange={(event) => setQuickConfigConfirmed(event.target.checked)} /><span>我确认按“{targetRegion}”执行以上本机配置，并允许逐项出现系统权限提示。</span></label>{quickConfigStage && <small className="quick-config-stage">{quickConfigStage}</small>}{quickConfigError && <p className="error-note">{quickConfigError}</p>}<button className="secondary-button" onClick={onQuickConfigure} disabled={!quickConfigConfirmed || !quickConfigReady || isQuickConfiguring}>{isQuickConfiguring ? "正在按顺序配置本机…" : "按此地区一键配置本机"}<span>→</span></button></div>
      <p className="field-help">预检只验证来源并保护本地副本；不会替你上传订阅，也不会把完整地址写入日志。</p>
    </section>
    <aside className="card side-card"><span className="mini-label">配置预览</span><h3>目标环境</h3><label className="field-label">地区与出口</label><select value={targetRegion} onChange={(event) => setTargetRegion(event.target.value)}><option>美国 · 洛杉矶</option><option>美国 · 纽约</option><option>日本 · 东京</option><option>新加坡</option><option>英国 · 伦敦</option></select><div className="preview-list"><div><span>接管工具</span><b>Clash Verge Rev</b></div><div><span>模式</span><b>规则 + TUN</b></div><div><span>隐私设置</span><b>最后一步</b></div></div><div className="warning-box"><span>↳</span><p>订阅凭据不会显示在界面、日志或验收记录中。</p></div></aside>
  </div>;
}

function ClashStep({ isConfigured, plan, backupConfirmed, setBackupConfirmed, backupResult, backupError, clashImportResult, clashImportError, clashApplyResult, clashApplyError, isBackingUp, backups, selectedBackupId, setSelectedBackupId, restoreConfirmed, setRestoreConfirmed, restoreResult, restoreError, isRestoring, onConfigure, onRestore, onOpenClash }: { isConfigured: boolean; plan: ClashChangePlan | null; backupConfirmed: boolean; setBackupConfirmed: (value: boolean) => void; backupResult: ClashBackupResult | null; backupError: string | null; clashImportResult: ClashImportResult | null; clashImportError: string | null; clashApplyResult: ClashApplyResult | null; clashApplyError: string | null; isBackingUp: boolean; backups: ClashBackupSummary[]; selectedBackupId: string; setSelectedBackupId: (value: string) => void; restoreConfirmed: boolean; setRestoreConfirmed: (value: boolean) => void; restoreResult: ClashRestoreResult | null; restoreError: string | null; isRestoring: boolean; onConfigure: () => void; onRestore: () => void; onOpenClash: () => void }) {
  const items = [["规则模式", "明确写入 mode: rule，Claude、Anthropic 与常用服务按规则分流"], ["TUN 服务", "接管 TCP / UDP，避免仅代理浏览器 HTTP"], ["DNS 劫持", "接管 UDP 53 与 TCP 53，并按规则解析，阻止系统绕行"], ["WebRTC UDP", "让检测页看到与代理一致的 UDP 出口"]];
  return <div className="grid-layout">
    <section className="card card-large">
      <div className="card-heading"><div><span className="section-number">B</span><div><h2>Clash Verge Rev 生效配置</h2><p>先备份现有配置，再进入托管扩展写入阶段。</p></div></div><span className={`state-badge ${isConfigured ? "ready" : ""}`}>{isConfigured ? "已写入" : "待备份"}</span></div>
      <div className="config-list">{items.map(([title, detail]) => <div className="config-row" key={title}><span className="check-circle">{isConfigured ? "✓" : ""}</span><div><b>{title}</b><small>{detail}</small></div><span className="row-state">{isConfigured ? "待复核" : "预览"}</span></div>)}</div>
      <div className="preview-list clash-plan-list"><div><span>配置目录</span><b>{plan?.dataDirExists ? "已发现（不读取内容）" : "未发现 / 待接入"}</b></div><div><span>Claude 规则</span><b>{plan?.claudeRuleCount ?? "—"} 个域名</b></div><div><span>托管方式</span><b>{plan?.managedExtension ?? "Global Merge"}</b></div></div>
      <label className="confirm-line"><input type="checkbox" checked={backupConfirmed} onChange={(event) => setBackupConfirmed(event.target.checked)} /><span>我确认先备份，并允许替换 profiles/Merge.yaml；备份可能包含订阅地址、节点认证信息或 UUID。</span></label>
      {backupResult && <div className="backup-result"><span className="check-icon passed">✓</span><div><b>{backupResult.status === "created" ? "本地备份已创建" : "浏览器预览：未写入文件"}</b><small>{backupResult.status === "created" ? `已保存 ${backupResult.copiedFiles} 个配置文件` : backupResult.warnings[1]}</small></div></div>}
      {backupError && <p className="error-note">{backupError}</p>}
      {clashImportResult && <div className="backup-result"><span className="check-icon passed">✓</span><div><b>订阅已导入 Clash Verge Rev</b><small>{clashImportResult.profileRelativePath} · 已设为当前配置；请在 Clash Verge Rev 中刷新并确认运行态</small></div></div>}
      {clashImportError && <p className="error-note">{clashImportError}</p>}
      {clashApplyResult && <div className="backup-result"><span className="check-icon passed">✓</span><div><b>{clashApplyResult.status === "written" ? "托管扩展已写入" : "浏览器预览：未写入文件"}</b><small>{clashApplyResult.status === "written" ? `${clashApplyResult.extensionRelativePath}；请重新加载 Clash Verge Rev` : clashApplyResult.warnings[0]}</small></div></div>}
      {clashApplyError && <p className="error-note">{clashApplyError}</p>}
      <div className="verify-actions"><button className="secondary-button" onClick={onConfigure} disabled={!backupConfirmed || isBackingUp}>{isBackingUp ? "正在备份并写入…" : isConfigured ? "重新备份并写入托管扩展" : "确认、备份并写入托管扩展"}<span>↗</span></button><button className="link-button" onClick={onOpenClash}>打开 Clash Verge Rev<span>↗</span></button></div>
    </section>
    <aside className="card side-card">
      <span className="mini-label">推荐运行态</span><div className="mode-visual"><div className="mode-ring">⌁</div><strong>规则模式</strong><span>TUN ON · 系统代理 OFF</span></div>
      <div className="tip-box"><b>备份边界</b><p>只复制 verge.yaml、config.yaml、profiles.yaml 和 profiles/，不复制日志、缓存，也不会输出文件内容。</p></div>
      <div className="backup-manager">
        <div className="backup-manager-heading"><span>备份管理</span><b>{backups.length} 个快照</b></div>
        {backups.length === 0 ? <p className="side-copy">还没有可恢复的本地快照。</p> : <select value={selectedBackupId} onChange={(event) => setSelectedBackupId(event.target.value)}><option value="">选择恢复快照</option>{backups.map((backup) => <option key={backup.backupId} value={backup.backupId}>{backup.backupId} · {backup.copiedFiles} 个文件</option>)}</select>}
        <label className="confirm-line"><input type="checkbox" checked={restoreConfirmed} onChange={(event) => setRestoreConfirmed(event.target.checked)} /><span>恢复前先自动备份当前配置</span></label>
        {restoreResult && <div className="backup-result"><span className="check-icon passed">✓</span><div><b>{restoreResult.status === "restored" ? "配置已恢复" : "浏览器预览：未恢复文件"}</b><small>{restoreResult.status === "restored" ? `恢复 ${restoreResult.restoredFiles} 个文件；恢复前快照：${restoreResult.preRestoreBackupId}` : restoreResult.warnings[0]}</small></div></div>}
        {restoreError && <p className="error-note">{restoreError}</p>}
        <button className="secondary-button" onClick={onRestore} disabled={!selectedBackupId || !restoreConfirmed || isRestoring}>{isRestoring ? "正在恢复…" : "恢复所选配置"}<span>↗</span></button>
      </div>
    </aside>
  </div>;
}

function LocalStep({ targetRegion, snapshot, plan, confirmed, setConfirmed, result, error, isApplying, applied, onApply, restoreConfirmed, setRestoreConfirmed, restoreResult, restoreError, isRestoring, onRestore }: { targetRegion: string; snapshot: PlatformSnapshot | null; plan: RegionChangePlan | null; confirmed: boolean; setConfirmed: (value: boolean) => void; result: RegionApplyResult | null; error: string | null; isApplying: boolean; applied: boolean; onApply: () => void; restoreConfirmed: boolean; setRestoreConfirmed: (value: boolean) => void; restoreResult: RegionRestoreResult | null; restoreError: string | null; isRestoring: boolean; onRestore: () => void }) {
  return <div className="grid-layout">
    <section className="card card-large">
      <div className="card-heading"><div><span className="section-number">C</span><div><h2>按目标地区配置本机</h2><p>执行前展示具体变更；系统权限只在用户确认后请求。</p></div></div><span className={`state-badge ${applied ? "ready" : ""}`}>{applied ? "已执行" : "待执行"}</span></div>
      <div className="setting-grid"><SettingRow label="时区" value={plan?.timezone ?? "读取中…"} /><SettingRow label="系统语言" value={plan?.language ?? "读取中…"} /><SettingRow label="地区格式" value={plan?.locale ?? targetRegion} /><SettingRow label="系统定位" value="保持由系统授权控制" /></div>
      <div className="permission-box"><b>权限提示</b><p>{plan?.permissionPrompts[0] ?? "执行时可能需要管理员授权"}</p></div>
      <label className="confirm-line"><input type="checkbox" checked={confirmed} onChange={(event) => setConfirmed(event.target.checked)} /><span>我确认按“{targetRegion}”修改系统时区、语言和地区格式。</span></label>
      {result && <div className="backup-result"><span className="check-icon passed">✓</span><div><b>{result.status === "applied" ? "系统配置已执行" : "浏览器预览：未修改系统"}</b><small>{result.status === "applied" ? `${result.changed.join("；")}；快照 ${result.snapshotId}` : result.warnings[0]}</small></div></div>}
      {error && <p className="error-note">{error}</p>}
      <button className="secondary-button" onClick={onApply} disabled={!confirmed || isApplying}>{isApplying ? "正在请求系统权限…" : applied ? "重新按此地区配置" : "确认并配置本机"}<span>→</span></button>
      {result?.snapshotId && <><label className="confirm-line"><input type="checkbox" checked={restoreConfirmed} onChange={(event) => setRestoreConfirmed(event.target.checked)} /><span>恢复前先再次保存当前系统地区。</span></label>{restoreResult && <div className="backup-result"><span className="check-icon passed">✓</span><div><b>{restoreResult.status === "restored" ? "地区快照已恢复" : "浏览器预览：未恢复系统"}</b><small>{restoreResult.status === "restored" ? `已恢复 ${restoreResult.restored.join("、")}；恢复前快照：${restoreResult.preRestoreSnapshotId}` : restoreResult.warnings[0]}</small></div></div>}{restoreError && <p className="error-note">{restoreError}</p>}<button className="secondary-button" onClick={onRestore} disabled={!restoreConfirmed || isRestoring}>{isRestoring ? "正在恢复地区快照…" : "恢复本次地区快照"}<span>↶</span></button></>}
    </section>
    <aside className="card side-card"><span className="mini-label">只读探测</span><h3>{snapshot ? snapshot.platform : "等待原生层"}</h3><div className="probe-list"><ProbeRow label="默认浏览器" value={snapshot?.defaultBrowser ?? "待接入原生探测"} /><ProbeRow label="Claude Desktop" value={snapshot ? (snapshot.claudeDesktopInstalled ? "已发现" : "未发现") : "待读取"} /><ProbeRow label="Clash Verge Rev" value={snapshot ? (snapshot.clashVergeInstalled ? "已发现" : "未发现") : "待读取"} /></div><p className="side-copy">系统时区、语言和地区会按目标地区执行；定位权限与浏览器权限继续单独确认。</p><div className="side-line"><span className="status-dot muted" />{plan?.restartRequirements[0] ?? "可回滚"}</div></aside>
  </div>;
}

function BrowserStep({ plan, confirmed, setConfirmed, result, error, isApplying, applied, expanded, setExpanded, onApply }: { plan: BrowserChangePlan | null; confirmed: boolean; setConfirmed: (value: boolean) => void; result: BrowserApplyResult | null; error: string | null; isApplying: boolean; applied: boolean; expanded: string | null; setExpanded: (value: string | null) => void; onApply: () => void }) {
  const rows = [["语言", plan?.language ?? "跟随系统首选语言", plan?.automatic ? "可自动写入" : "需要确认"], ["时区", "跟随系统时区", "自动检查"], ["定位权限", "默认阻止网站读取位置", plan?.automatic ? "可自动写入" : "打开设置页"], ["安全 DNS / WebRTC", "由 Clash TUN 接管并在验收页复核", "需要验收"]];
  return <div className="grid-layout"><section className="card card-large"><div className="card-heading"><div><span className="section-number">D</span><div><h2>按地区配置默认浏览器</h2><p>Chrome / Edge 默认配置会先备份 Preferences；Safari、Firefox 或未识别浏览器只做人工引导。</p></div></div><span className={`state-badge ${applied ? "ready" : ""}`}>{applied ? "已执行" : "待检查"}</span></div><div className="browser-banner"><span className="browser-orb">◎</span><div><b>当前默认浏览器</b><strong>{plan?.browserLabel ?? "识别中…"}</strong></div><span className="browser-mode">{plan?.automatic ? "可自动配置" : "手动确认"}</span></div><div className="browser-list">{rows.map(([label, value, action]) => <button className="browser-row" key={label} onClick={() => setExpanded(expanded === label ? null : label)}><span className={`browser-check ${applied && label !== "安全 DNS / WebRTC" ? "passed" : ""}`}>{applied && label !== "安全 DNS / WebRTC" ? "✓" : "○"}</span><span><b>{label}</b><small>{value}</small></span><em>{action}</em><span>＋</span></button>)}</div>{expanded && <div className="inline-guide"><b>{expanded}的处理边界</b><p>{plan?.manualSteps[0] ?? "请在真实浏览器设置页确认，safecc 不注入脚本伪造时区或 WebRTC 结果。"}</p></div>}<div className="permission-box"><b>执行前确认</b><p>{plan?.permissionPrompts[0] ?? "会先备份浏览器偏好，浏览器必须完全退出；safecc 不会强制结束进程。"}</p></div><label className="confirm-line"><input type="checkbox" checked={confirmed} onChange={(event) => setConfirmed(event.target.checked)} /><span>{plan?.automatic ? "我确认已退出默认浏览器，并允许 safecc 备份和修改可自动化的偏好。" : "我已按页面提示检查默认浏览器的语言、定位和安全 DNS 设置。"}</span></label>{result && <div className="backup-result"><span className="check-icon passed">✓</span><div><b>{result.status === "applied" ? "浏览器环境已写入" : "人工检查已记录"}</b><small>{result.status === "applied" ? `${result.changed.join("；")}；本地快照 ${result.backupId ?? "未生成"}；请重启浏览器。` : result.warnings[0]}</small></div></div>}{error && <p className="error-note">{error}</p>}<button className="secondary-button" onClick={onApply} disabled={!confirmed || isApplying}>{isApplying ? "正在备份并写入浏览器偏好…" : plan?.automatic ? applied ? "重新备份并配置浏览器" : "确认、备份并配置浏览器" : "我已按提示检查浏览器"}<span>→</span></button></section><aside className="card side-card"><span className="mini-label">不做的事</span><h3>不伪造网页指纹</h3><p className="side-copy">不注入脚本伪造时区、Canvas 或 WebRTC 结果。安全 DNS、UDP 和最终出口必须以真实检测页面为准。</p><div className="preview-list"><div><span>目标地区</span><b>{plan?.targetRegion ?? "读取中…"}</b></div><div><span>语言偏好</span><b>{plan?.language ?? "—"}</b></div><div><span>配置文件</span><b>{plan?.profileExists ? "已发现" : "未发现"}</b></div></div>{plan?.manualSteps.map((step) => <p className="side-copy" key={step}>· {step}</p>)}</aside></div>;
}

function CacheStep({ plan, approval, setApproval, result, error, isCleaning, confirmed, onClean }: { plan: CachePlan | null; approval: boolean; setApproval: (value: boolean) => void; result: CacheCleanupResult | null; error: string | null; isCleaning: boolean; confirmed: boolean; onClean: () => void }) {
  const availableTargets = plan?.targets.filter((target) => target.exists) ?? [];
  return <div className="grid-layout"><section className="card card-large"><div className="card-heading"><div><span className="section-number">E</span><div><h2>安全缓存清理</h2><p>只清理可重建内容，避免意外退出账号。</p></div></div><span className={`state-badge ${confirmed ? "ready" : ""}`}>{confirmed ? "已清理" : "待确认"}</span></div><div className="cleanup-columns"><div><span className="column-label">将清理</span><ul className="clean-list"><li>系统 DNS 缓存{plan?.dnsFlushAvailable ? "（可用）" : "（待原生层）"}</li>{availableTargets.length > 0 ? availableTargets.map((target) => <li key={target.label}>{target.label}</li>) : <li>浏览器 / Clash 可重建缓存（按实际存在路径处理）</li>}</ul></div><div><span className="column-label muted-text">默认保留</span><ul className="clean-list keep">{(plan?.preserved ?? ["Cookie 与登录会话", "密码、书签和历史", "扩展与用户文件", "Local Storage / IndexedDB"]).map((item) => <li key={item}>{item}</li>)}</ul></div></div><label className="confirm-line"><input type="checkbox" checked={approval} onChange={(event) => setApproval(event.target.checked)} /><span>我确认执行标准清理，不执行深度清理。</span></label>{result && <div className="backup-result"><span className="check-icon passed">✓</span><div><b>{result.status === "cleaned" ? "标准缓存已清理" : "浏览器预览：未清理文件"}</b><small>{result.status === "cleaned" ? `清理 ${result.cleanedEntries} 个缓存项${result.dnsFlushRequested ? "，已请求刷新 DNS" : ""}` : result.warnings[0]}</small></div></div>}{error && <p className="error-note">{error}</p>}<button className="secondary-button" onClick={onClean} disabled={!approval || isCleaning}>{isCleaning ? "正在清理可重建缓存…" : confirmed ? "重新执行标准清理" : "确认并执行标准清理"}<span>→</span></button></section><aside className="card side-card"><span className="mini-label">深度清理</span><h3>需要另一次确认</h3><p className="side-copy">若要清除登录和站点数据，必须先备份并列出影响范围。safecc 不提供含糊的“一键全清”。</p></aside></div>;
}

function VerifyStep({ targetRegion, verified, record, error, isRecording, onOpen, onVerify }: { targetRegion: string; verified: boolean; record: VerificationRecord | null; error: string | null; isRecording: boolean; onOpen: () => void; onVerify: () => void }) {
  return <div className="grid-layout"><section className="card card-large"><div className="card-heading"><div><span className="section-number">F</span><div><h2>真实出口验收</h2><p>先打开指定检测页，完成真实检查后再由用户确认记录。</p></div></div><span className={`state-badge ${verified ? "ready" : ""}`}>{verified ? "验收已记录" : "待检测"}</span></div><div className="verify-target"><div><span className="mini-label">检测地址</span><strong>ip.net.coffee / claude</strong></div><div className="verify-actions"><button className="secondary-button compact" onClick={onOpen}>打开检测页<span>↗</span></button><button className="link-button" onClick={onVerify} disabled={isRecording}>{isRecording ? "正在保存" : verified ? "重新记录" : "我已确认结果"}</button></div></div><div className="check-grid">{checks.map((check) => <div className="check-card" key={check.label}><span className={`check-icon ${verified ? "passed" : ""}`}>{verified ? "✓" : "○"}</span><div><b>{check.label}</b><small>{verified ? "用户已确认（未自动读取页面）" : check.value}</small></div></div>)}</div>{record && <div className="backup-result"><span className="check-icon passed">✓</span><div><b>脱敏验收记录已保存</b><small>{record.targetRegion} · {record.routeAlias} · {record.clashConfigVersion} · 记录编号 {record.recordId} · 未保存 IP、订阅地址或页面原文</small></div></div>}{error && <p className="error-note">{error}</p>}<p className="tiny-note">safecc 不伪造检测结果；“我已确认结果”表示你已在真实页面核对出口、DNS、WebRTC UDP、时区和语言。</p></section><aside className="card side-card"><span className="mini-label">通过条件</span><h3>先通过网络，再做隐私</h3><p className="side-copy">只有 DNS、WebRTC UDP、时区、语言和 Claude 可用性都被记录后，才会进入最后一步。</p><div className="preview-list"><div><span>目标地区</span><b>{targetRegion}</b></div><div><span>线路别名</span><b>{record?.routeAlias ?? "待记录"}</b></div><div><span>Clash 配置</span><b>{record?.clashConfigVersion ?? "待记录"}</b></div><div><span>本地记录</span><b>{record ? "已保存" : "待保存"}</b></div></div></aside></div>;
}

function PrivacyStep({ loggedIn, confirmed, completed, onOpenLogin, onConfirmLogin, onOpenLocation, onOpenClaude, onConfirm }: { loggedIn: boolean; confirmed: boolean; completed: boolean; onOpenLogin: () => void; onConfirmLogin: (value: boolean) => void; onOpenLocation: () => void; onOpenClaude: () => void; onConfirm: (value: boolean) => void }) {
  return <div className="grid-layout"><section className="card card-large privacy-card"><div className="last-step-ribbon">最后一步 · Claude 隐私</div><div className="card-heading"><div><span className="section-number">G</span><div><h2>登录 Claude 后关闭隐私设置</h2><p>网络环境已经验收；safecc 不接收、不保存你的 Claude 账号或密码。</p></div></div><span className={`state-badge ${confirmed ? "ready" : ""}`}>{completed ? "已完成" : confirmed ? "已确认" : loggedIn ? "已登录，待设置" : "待登录"}</span></div><div className="privacy-row"><span className="privacy-icon">↗</span><div><b>先登录 Claude</b><small>在官方登录页完成登录；safecc 只记录你是否确认已登录，不读取账号信息。</small></div><button className="link-button" onClick={onOpenLogin}>打开登录页</button></div><label className="confirm-line"><input type="checkbox" checked={loggedIn} onChange={(event) => onConfirmLogin(event.target.checked)} /><span>我已在官方 Claude 页面完成登录（不会把账号、密码或验证码交给 safecc）。</span></label><div className={`privacy-row ${!loggedIn ? "privacy-row-locked" : ""}`}><span className="privacy-icon">⌖</span><div><b>关闭定位权限</b><small>登录后打开系统隐私设置，拒绝 Claude 使用位置权限。</small></div><button className="link-button" onClick={onOpenLocation} disabled={!loggedIn}>打开设置</button></div><div className={`privacy-row ${!loggedIn ? "privacy-row-locked" : ""}`}><span className="privacy-icon">◒</span><div><b>关闭“不帮助模型改进”</b><small>登录后打开 Claude 官方设置页，将“帮助改进模型 / 协助改进 AI”关闭。</small></div><button className="link-button" onClick={onOpenClaude} disabled={!loggedIn}>打开 Claude 设置</button></div><label className="confirm-line privacy-confirm"><input type="checkbox" checked={confirmed} onChange={(event) => onConfirm(event.target.checked)} disabled={!loggedIn} /><span>我已登录 Claude，并确认定位权限与“帮助改进模型 / 协助改进 AI”均已关闭。</span></label>{completed && <div className="backup-result"><span className="check-icon passed">✓</span><div><b>safecc 配置流程已完成</b><small>登录、定位和模型改进设置均由你在官方页面确认；safecc 未保存账号或页面内容。</small></div></div>}</section><aside className="card side-card"><span className="mini-label">完成后</span><h3>保存两份状态</h3><div className="evidence-line"><span>网络验收</span><b className="green-text">通过</b></div><div className="evidence-line"><span>Claude 登录</span><b className={loggedIn ? "green-text" : "muted-text"}>{loggedIn ? "已确认" : "待确认"}</b></div><div className="evidence-line"><span>Claude 隐私</span><b className={confirmed ? "green-text" : "muted-text"}>{confirmed ? "已确认" : "待确认"}</b></div><p className="side-copy">safecc 不代填登录信息，也不读取 Claude 页面内容；隐私未完成不会篡改网络验收结果。</p></aside></div>;
}

function SettingRow({ label, value }: { label: string; value: string }) {
  return <div className="setting-row"><span>{label}</span><b>{value}</b><span className="row-state">预览</span></div>;
}

function ProbeRow({ label, value }: { label: string; value: string }) {
  return <div className="probe-row"><span>{label}</span><b>{value}</b></div>;
}

export default App;
