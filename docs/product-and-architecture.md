# 产品与技术方案

## 1. 产品定位

safecc 是一个 macOS / Windows 双平台桌面应用，帮助用户建立“可检查、可配置、可恢复”的 Claude 使用环境。三种线路来源统一交给 Clash Verge Rev 管理，应用负责生成配置、调用受支持的导入入口、引导必要的系统授权并核验最终结果。

对外命名应避免“防封”“养号神器”等保证性表达。建议定位为：

> safecc：统一接入网络订阅，一键配置本机与浏览器环境，并完成可复核的泄露检测。

### 明确不做

- 不保证账号不会被限制、暂停或封禁；
- 不绕过地区资格、付款限制、验证码或平台风控；
- 不批量注册、共享账号、自动登录或模拟用户行为；
- 不把 Claude 账号接入非官方 API 转换服务；
- 不修改或伪造设备硬件指纹来绕过风控；
- 不收集用户密码、验证码、Cookie、会话或完整客户资料；
- 不在未确认的情况下修改系统代理、DNS、防火墙或远端服务器。

## 2. 首版用户流程

1. **选择线路来源**：自建 VPS、静态 IP 订阅、机场订阅三选一，也可保存多个方案但一次只激活一个；第一步提供搬瓦工（BandwagonHost）和 DMIT 官方 VPS 入口，仅打开官方页面，不代购、不填写付款信息。
2. **预检订阅**：本地解析并验证格式、节点、规则和更新地址；界面和日志不展示完整订阅 URL、UUID 或口令。
3. **统一导入 Clash Verge Rev**：创建应用托管的基础配置与扩展配置，选择规则模式，安装/启用服务模式和 TUN；修改前先创建可恢复快照。
4. **一键配置本机**：按目标地区设置时区、首选语言和地区；执行系统变更时由原生层触发管理员授权或 Windows UAC，定位权限单独展示并确认。
5. **配置默认浏览器**：识别 Safari、Chrome、Edge 或 Firefox，配置可自动化的语言/权限项；不可稳定自动化的项目打开准确设置页并要求用户确认。
6. **清理缓存**：按预览清理系统临时缓存、DNS 缓存和浏览器网络缓存；默认保留 Cookie、登录、密码、书签、历史、扩展和站点数据。
7. **网络与环境验收**：打开 `https://ip.net.coffee/claude/`，检查出口、Claude 可用性、DNS、WebRTC UDP、时区、语言和设备信息；用户确认后只保存目标地区、时间、固定检查项和状态，不保存 IP、订阅地址或页面原文。
8. **最后配置 Claude 隐私**：网络与环境验收通过后，先由用户在官方 Claude 页面完成登录，再关闭 macOS/Windows 系统层的 Claude 定位权限；“协助改进 AI / 帮助改进模型”仅在官方设置页完成，无法通过稳定公开接口自动修改时采用引导确认。safecc 不接收或保存账号、密码、验证码和页面内容。

在线路预检完成后，UI 提供“按此地区一键配置本机”编排入口，顺序固定为 Clash 备份/导入、系统地区、默认浏览器和标准缓存。它不会包含 VPS 远端部署、真实检测页确认或 Claude 隐私步骤；每个会产生系统写入的阶段仍由原生层触发操作系统权限提示，任一阶段失败都会停止，不会跳过后续步骤。

部署完成后进入 **维护中心**，管理订阅更新、线路切换、只读体检、证书/服务状态、备份状态和可选择的修复。恢复历史 Clash 快照前，会先自动备份当前配置，并在恢复失败时保留这个恢复前快照。

## 3. 线路来源与 Clash Verge Rev 配置矩阵

| 来源 | 用户输入 | 工具输出 | 限制与验证 |
| --- | --- | --- | --- |
| 自建 VPS | IP、SSH 用户/端口、密钥或安全密码输入 | 部署节点、生成私有订阅、导入 Clash Verge Rev | 远端变更必须单独确认；只支持声明过的系统版本 |
| 静态 IP 订阅 | 供应商订阅 URL，或 SOCKS5/HTTP 主机、端口、认证 | 转换为仅本机可读的 Mihomo 配置并导入 | 必须确认协议与认证方式；不把凭据发送到第三方转换站 |
| 机场订阅 | Clash/Mihomo 兼容订阅 URL | 原样拉取到本机缓存，再叠加应用托管规则 | 不篡改供应商节点；更新失败保留最后一份已验证配置 |

三种来源统一使用 Clash Verge Rev。工具不直接编辑供应商远程订阅，而是维护两个可审计层：

当前首版的订阅接入边界是：静态 IP / 机场地址只在本机通过 HTTPS 预检，识别 Clash/Mihomo、节点列表或编码格式，并将原始响应保存到受保护的 safecc 快照目录；用户确认并完成备份后，safecc 会原子更新 Clash Verge Rev 的 `profiles.yaml` 与 remote profile 缓存文件，将新 profile 设为当前配置。VPS 首阶段会读取 SSH 主机公钥并展示 SHA-256 指纹，随后可在用户再次确认、提供本机 PEM 且核对指纹后执行 Debian 12/13 root SSH 只读检查；只读检查不安装、不改防火墙、不写远端文件。真正部署仍需单独的远端变更审批。预检和导入不会把订阅上传到第三方转换站；导入后仍需在 Clash Verge Rev 中刷新并复核最终运行态。

1. **来源层**：节点和供应商规则；
2. **托管扩展层**：TUN、DNS、Claude/Anthropic 分流、直连保留网段与泄露防护。

推荐运行态：

- Clash Verge Rev 规则模式；
- TUN 开启，服务模式已安装；
- 系统代理关闭，避免和 TUN 双重接管；
- `auto-route` 与 `auto-detect-interface` 开启；
- DNS 由 Mihomo 接管，UDP 53/TCP 53 劫持到内核；
- 托管扩展显式写入规则模式与 `dns.respect-rules`，避免依赖 Clash 默认值；
- Claude/Anthropic 相关域名固定走当前选中的线路组；
- 局域网、回环、系统更新和必要的国内服务按明确规则直连；
- 默认不允许代理出口 IPv4 与直连 IPv6 混用，IPv6 策略由预检结果决定。

Clash Verge Rev 的 GUI 全局设置可能覆盖订阅内的 `tun` 配置，因此实现时必须使用其受支持的配置/扩展机制，并对最终生效配置做二次读取，不能只检查写入文件。

Clash Verge Rev 的应用数据目录可能包含订阅 URL、节点地址、密码和 UUID。工具可以探测目录是否存在；只有用户确认导入后，才读取 `profiles.yaml` 的结构字段来生成新的 remote profile，绝不打印或上传其值，也不读取现有订阅内容。需要可回滚备份时必须明确确认，原始副本只保存到当前用户的本机受保护目录，不会输出或上传。

原生接入先生成 `ClashChangePlan`：列出待备份文件、Claude 域名规则数量、TUN/DNS 字段、警告和确认项；只有用户明确确认后，才会创建本地快照并写入 `profiles/Merge.yaml` 托管扩展。safecc 不读取供应商现有内容，生成的 Claude 规则默认指向 `PROXY` 策略组，若当前订阅没有同名策略组，必须在 Clash Verge Rev 中选择有效策略组后再验收。

按地区配置本机时，safecc 先把当前时区、地区格式和首选语言保存为受保护的地区快照，再由 macOS `systemsetup` 管理员授权或 Windows UAC 执行；本机步骤提供“恢复本次地区快照”，恢复前会再次保存当前状态。浏览器步骤对已识别且已退出的 Chrome / Edge 在备份 `Preferences` 后修改语言偏好、网站定位默认值并关闭浏览器 DoH，让 DNS 回到 Clash TUN；Safari、Firefox、未识别浏览器或缺少配置文件时只记录人工确认，不强行改写内部文件，一键流程会停在浏览器步骤而不会把人工步骤误标记为完成。

## 4. 本机与浏览器配置边界

| 项目 | macOS | Windows | 自动化策略 |
| --- | --- | --- | --- |
| 时区 | 系统时区，需管理员授权 | 系统时区，可能需管理员权限 | 预览后修改，记录原值用于回滚 |
| 语言/地区 | 系统首选语言与区域 | Windows 显示/区域格式 | 仅设置已安装/支持的语言，可能要求注销 |
| 系统定位 | Claude 的定位权限关闭 | Claude 的位置权限关闭 | 优先系统隐私设置；不绕过系统授权数据库保护 |
| 默认浏览器语言 | Safari/Chromium/Firefox 分适配 | Chromium/Firefox 分适配 | 只通过公开偏好或企业策略接口；否则引导用户操作 |
| 浏览器时区 | 默认跟随系统 | 默认跟随系统 | 不注入网页脚本伪造时区 |
| 浏览器地区 | Accept-Language 与搜索/内容地区分开 | 同左 | 只配置可验证项，不声称等同账号地区 |
| 浏览器定位 | 网站权限设为询问/阻止 | 网站权限设为询问/阻止 | 不写入伪造坐标；测试页验证暴露结果 |

“默认浏览器环境”不等于改变浏览器账号或 Claude 服务端地区。对无法稳定自动化的浏览器设置，应用应打开对应设置页，展示一步操作，并读取结果重新检查。

## 5. 缓存清理等级

默认使用安全的 **标准清理**：

- 刷新系统 DNS 缓存；
- 清理 Clash Verge Rev/Mihomo 可重建的网络缓存；
- 清理默认浏览器 HTTP 缓存、Service Worker 缓存和临时网络缓存；
- 清理 Claude Desktop 的 HTTP/GPU/Code Cache；
- 保留 Cookie、登录会话、Local Storage、IndexedDB、密码、书签、历史、扩展和用户文件。

另设需要二次确认的 **深度清理**，它可能让网站和 Claude 退出登录。执行前必须列出将删除的数据类型和影响，并创建可恢复项的备份。不得提供一个含糊的“清除全部”按钮。

## 6. DNS 与 WebRTC UDP 防泄露

### DNS

- 由 Clash Verge Rev/Mihomo 在 TUN 中接管 DNS；
- 劫持 UDP 53 与 TCP 53，处理系统和浏览器常规解析；
- 明确选择加密上游，并按代理/直连策略决定解析路径；
- 对已识别且已退出的 Chrome / Edge，在备份 Preferences 后关闭浏览器 DoH，让解析回到 Clash TUN；若策略覆盖或浏览器不支持自动写入，打开设置页并复验；
- 验收读取检测页实际展示的 DNS 出口，而不是仅检查配置文件。

### WebRTC UDP

- 不通过禁用 WebRTC 或篡改浏览器 API 来“通过检测”；
- 用 Clash Verge Rev TUN 接管 UDP 路由，确保 STUN/UDP 流量从当前出口发送；
- 系统代理保持关闭，避免只代理 HTTP(S) 而让 UDP 直连；
- 验收要求检测页的 UDP 出口与当前代理出口一致，且不出现本地公网地址。

## 7. 推荐架构

采用单仓库、双平台、一个 UI 与一个原生核心：

```text
React + TypeScript UI
        |
        | typed Tauri commands/events
        v
Rust / Tauri 2 core
  |-- local environment probes
  |-- secret storage abstraction
  |-- SSH deployment state machine
  |-- rollback and audit journal
  |-- companion browser lifecycle
  `-- platform adapters
       |-- macOS: Keychain / AppKit / system settings
       `-- Windows: Credential Manager / Win32 / PowerShell

Remote VPS
  |-- idempotent, versioned deployment bundle
  |-- private proxy endpoint
  |-- HTTPS subscription endpoint
  |-- health checks and backups
  `-- uninstall / rollback manifest
```

### 技术选型

| 层 | 选择 | 原因 |
| --- | --- | --- |
| 桌面壳 | Tauri 2 + Rust | 安装包较小，原生权限和进程边界清楚，便于实现跨平台权限确认 |
| UI | React + TypeScript | 跨平台共享，适合向导、进度和诊断报告 |
| 本地数据库 | SQLite | 保存任务状态、非秘密元数据、审计记录和恢复点 |
| 秘密存储 | macOS Keychain / Windows Credential Manager | 不把密码、密钥或订阅地址写进日志和普通配置 |
| SSH | 系统 OpenSSH 优先，Rust 封装参数 | 复用成熟客户端，同时避免把秘密放进命令行 |
| 远端部署 | 版本化脚本包 + manifest | 可审计、可重入、可验证 SHA-256、可回滚 |
| 独立浏览器 | 固定版本 Chromium + 独立 profile | 真实核验浏览器环境，不污染用户默认浏览器 |

Windows 首版不要求 WSL。若目标机缺少 OpenSSH Client，应用只给出系统功能安装入口，不静默安装。

## 8. 模块边界

```text
apps/desktop/                 React UI
crates/core/                  领域模型、任务状态机、审计与错误类型
crates/platform-macos/        Keychain、系统设置、权限桥接
crates/platform-windows/      Credential Manager、Win32、系统设置
crates/network-check/         出口、DNS、WebRTC 会话与结果归一化
crates/clash-verge/           安装探测、订阅导入、扩展配置与生效态读取
crates/deployer/              SSH、上传、阶段执行、验收与回滚
crates/browser-adapters/       Safari、Chrome、Edge、Firefox 适配器
crates/claude-desktop/         Claude 权限探测、设置页引导和缓存边界
deployment/                   远端脚本、模板、版本锁和哈希清单
browser-runtime/              浏览器元数据与打包入口，不提交大型二进制
docs/                         用户边界、威胁模型、发布和运维文档
tests/fixtures/               虚构数据与本地假 SSH 服务
```

UI 只负责收集输入、展示预览和进度。所有路径校验、权限判断、秘密处理、进程识别和远端命令生成都在 Rust 核心完成。

## 9. 部署与配置状态机

```text
Draft
  -> Preflight
  -> SourceValidated
  -> ClashConfigured
  -> ConnectionVerified
  -> ChangePlanReady
  -> AwaitingApproval
  -> BaseHardened
  -> CertificateReady
  -> ProxyReady
  -> SubscriptionReady
  -> LocalEnvironmentConfigured
  -> Verified
  -> PrivacySettingsConfirmed
  -> Delivered

任一执行阶段 -> FailedRecoverable -> 重跑同阶段
任一已变更阶段 -> RollbackPreview -> AwaitingApproval -> RolledBack
```

每个阶段必须定义：前置条件、将变更的资源、幂等检测、成功证据、失败信息和回滚动作。应用崩溃或断网后从审计 journal 恢复，不依赖 UI 内存状态。

## 10. 安全与隐私要求

- 密码只允许用户在原生安全输入框或终端提示中输入，永不进入聊天、命令参数或应用日志；
- 私钥不复制到项目目录；临时副本使用最小权限并在任务结束后清理；
- 日志统一脱敏 URL、Authorization、Cookie、UUID、订阅路径和 Key；
- 首次连接必须显示并固定 SSH 主机指纹；指纹变化默认阻断；
- 远端脚本和二进制必须有固定版本与 SHA-256；
- 关闭密码登录前，必须用新密钥完成第二次独立登录验证；
- 修改防火墙前先确认当前 SSH 端口已放行；
- 所有删除、清空身份、全清浏览器和远端卸载都使用单独确认；
- 本地环境检查与远端部署分开授权；一次确认不能覆盖后续高风险动作；
- 机场/静态 IP 订阅只在本机拉取和解析，禁止上传到公共订阅转换服务；
- Clash Verge Rev 的安装、服务模式授权、配置覆盖和线路切换分别记录结果；
- 修改系统语言、地区、时区、定位权限和清理缓存前展示逐项预览；
- 检测结果标注数据来源与时间，IP 地理库只能作为估计。

## 11. MVP 范围

### M0：只读体检

- macOS Apple Silicon、Windows 10/11 x86_64；
- 系统时区、语言、地区、公网 IPv4/IPv6、DNS、WebRTC；
- Tauri 原生层只读探测默认浏览器、Claude Desktop、Clash Verge Rev 和定位权限边界；
- 结果导出为不含秘密的 JSON / Markdown；
- 不修改系统，不部署 VPS。

### M1：Clash Verge Rev 与本机一键配置

- 探测或引导安装受支持版本的 Clash Verge Rev；
- 支持机场订阅和静态 IP 订阅；
- 应用托管扩展配置、规则模式、TUN、DNS 与 UDP 路由；
- 系统/默认浏览器地区设置、标准缓存清理；
- Claude 定位权限关闭和“协助改进 AI”设置确认；
- 指定检测页的辅助验收与本机结构化报告。

### M2：私人 VPS 部署

- Debian 12 / 13（与当前只读检查和部署脚本一致）；
- 自定义 SSH 用户和端口；
- 密钥登录、主机指纹固定、部署预览；
- 分阶段部署、验收、备份和回滚；
- 生成并导入 Clash Verge Rev/Mihomo 配置；
- 不包含 CLIProxyAPI，不转换 Claude 登录为 API。

### M3：独立浏览器与身份管理评估

- 独立浏览器 profile 与 Claude CLI `CLAUDE_CONFIG_DIR`；每账号档案只生成空目录和本地入口；
- 可选地区设置修复；
- 身份备份/切换需另立威胁模型和恢复测试后再进入；
- 代码签名、自动更新和崩溃恢复。

## 12. 验收标准

指定验收页面：`https://ip.net.coffee/claude/`。

页面当前覆盖：Claude 出口 IP 与属性、信任评分、安全标签、Claude/Anthropic 可用性、DNS 出口、WebRTC UDP 出口、时区、语言、操作系统/浏览器和若干浏览器信息。

网络与环境验收通过必须同时满足：

1. Claude AI 出口 IP 等于当前 Clash Verge Rev 选中线路的预期出口；
2. `claude.ai` 与 `anthropic.com` 可用性检测通过；
3. DNS 检测不出现用户本地网络的 DNS 出口，且地区不与目标出口明显冲突；
4. WebRTC UDP 出口等于当前代理出口，且不出现用户本地公网 IP；
5. 页面报告的时区和语言与用户选择的目标地区一致；
6. 应用同步保存一份脱敏验收记录，包括时间、Clash 配置版本、线路别名和各项结果，不保存完整 IP、订阅 URL 或凭据。

网络与环境验收通过后才进入最后的 Claude 隐私配置。整个一键流程交付完成还必须满足：

1. Claude 系统定位权限已确认关闭；
2. “协助改进 AI”已由用户在官方设置页确认关闭；
3. 两项结果独立记录，不因隐私设置未完成而篡改前面的网络验收结果。

信任评分属于第三方页面的动态评价，不由本工具控制。它应展示为证据而非唯一硬门槛；默认建议阈值可以设为 70，但不得把达到阈值表述为账号安全保证。

该页面是最终用户验收入口，但应用还必须保留本地结构化检查，因为第三方页面可能变更、不可用或产生误判。

### 工程验收

- 同一代码库可生成 macOS arm64 `.app.zip`（DMG 需要构建机提供可用的磁盘映像设备）和 Windows x64 NSIS；MSI 不在当前构建入口内；
- 所有秘密字段都有脱敏测试，错误日志不含秘密值；
- SSH 主机指纹首次固定、变化阻断有自动化测试；
- 每个部署阶段重复运行不会重复创建资源或破坏现有配置；
- 断网、SSH 超时、证书失败、端口冲突和磁盘不足都有明确失败路径；
- 关闭密码登录前后都有可验证的防锁死门槛；
- 本地假服务器集成测试通过；
- 真实 VPS 验收必须使用专用测试机，并与本地自动化结果分开记录；
- UI 不出现“绝对安全”“保证不封”等表述。

## 13. 回滚

- 本地修改前保存带时间戳的设置快照；
- 远端每阶段写入资源清单和前置状态，不接管未知同名服务；
- 卸载只删除本工具创建且 manifest 可证明归属的资源；
- 证书、SSH 和防火墙回滚按依赖倒序执行；
- 如果无法证明某资源由本工具创建，停止并要求用户决定。

## 14. 开工顺序

1. 建立 Tauri 2 双平台空壳和 CI；
2. 完成 M0 只读体检及脱敏报告；
3. 完成机场订阅 → Clash Verge Rev → TUN/DNS/UDP → 检测页验收的首个纵向切片；
4. 增加静态 IP 订阅、本机/浏览器设置和安全缓存清理；
5. 建立 deployment manifest、状态机和本地假 SSH 测试；
6. 在一次性测试 VPS 上完成自建 VPS 纵向切片；
7. 打包、签名、更新和真实 macOS / Windows 验收；
8. 最后评估是否引入独立浏览器与身份管理。

