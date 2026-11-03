# safecc

safecc 是面向 macOS 与 Windows 的本地环境检查和私人网络部署工具。

本项目的目标是帮助用户建立一致、可验证、可恢复的 Claude 使用环境，包括：

- 检查并配置本机与默认浏览器的时区、语言、地区和定位权限；Chrome/Edge 自动配置时会在备份后关闭浏览器 DoH，让 DNS 回到 Clash TUN；
- 接入自建 VPS、静态 IP 订阅或机场订阅，并统一导入 Clash Verge Rev；
- 第一步提供搬瓦工（BandwagonHost）和 DMIT 官方 VPS 入口，仅跳转访问，不代购或填写付款信息；
- 配置 Clash Verge Rev 的规则模式、TUN、DNS 劫持和 UDP 路由；
- 为不同 Claude 账号建立独立本机档案：每份档案各有 `CLAUDE_CONFIG_DIR`、Chromium `--user-data-dir` 与单独启动入口，不复制默认 `~/.claude` 的凭证；
- 清理经过用户确认的本机与浏览器缓存，不删除登录和个人资料；
- 在最后一步先由用户登录 Claude，再关闭客户端定位权限，并协助关闭“改进 AI / 帮助改进模型”设置；safecc 不接收账号密码；
- 在导入和配置后使用指定检测页重新验收真实出口；
- 对所有高风险操作给出变更预览、明确确认和回滚路径。

本项目不会承诺“防封”或保证账号可用。本地设置无法改变服务端风控记录，账号状态还受到服务条款、付款、地区资格、账号历史和使用行为等因素影响。

## 当前状态

当前已具备可运行的 React/Vite 原型和 Tauri 2 原生壳；已接入本机 HTTPS 订阅预检与受保护快照、订阅确认后自动登记为 Clash Verge Rev remote profile、VPS SSH 主机指纹预检与 Debian 12/13 只读检查、网络部署脚本分阶段部署与私有 Clash YAML 取回、Clash 配置快照/恢复、按地区申请系统权限与地区快照恢复、默认浏览器逐项设置页引导与 Chrome/Edge `Preferences` 只读识别、标准缓存清理、按所选地区一键编排本机步骤、真实验收页确认及本地脱敏验收记录（含线路别名、托管版本和逐项确认状态）。部署阶段不会在未确认时执行远端变更；真实 VPS 部署、Clash 运行态和检测页仍必须由用户在目标环境中复核。Safari/Firefox 没有安全稳定的统一只读入口，流程会明确进入人工确认模式；订阅后台自动更新仍需后续接入。Windows x64 同时提供 MSVC/CI 构建入口和 macOS 本地 GNU 交叉构建入口，安装包仍需在真实 Windows 机器上启动验收。首版范围、配置矩阵和验收条件见 [产品与技术方案](docs/product-and-architecture.md)。

## 本地开发

```bash
npm install
npm run dev          # 浏览器预览
npm run tauri:dev    # Tauri 本机窗口（需要 Rust 与平台桌面依赖）
npm run build
cargo check --manifest-path src-tauri/Cargo.toml
```

## 当前可安装产物

在 Apple Silicon macOS 上，当前可直接解压产物位于 [`artifacts/safecc-macos-arm64.app.zip`](artifacts/safecc-macos-arm64.app.zip)；解压后将 `safecc.app` 拖到“应用程序”即可。当前包为本机 ad-hoc 签名，首次打开若被 Gatekeeper 拦截，请在“系统设置 → 隐私与安全性”中手动允许。DMG 在当前机器上因磁盘映像设备不可用未生成。

当前包 SHA-256：`4f2e03b229edb898a8e335bd9be7ba01f7da824bb1e5eb6818815200dcb482dc`

## 多账号档案的边界

safecc 的账号档案用于隔离同一台设备上的 Claude CLI 配置目录和 Chromium 浏览器 Profile。创建时只生成空目录与启动脚本，不读取、复制或展示任何已有 Token、Cookie、密码或 Claude 页面内容。档案不是独立设备：系统时区、硬件指纹、浏览器引擎特征和同一时刻的 Clash 活动出口仍由整机共享。不同地区的档案需要切换 Clash 出口，并重新执行本机地区和验收步骤；safecc 不把档案隔离表述为账号安全保证。

账号档案页还会检测本机是否已安装兼容的 Fingerprint Chromium 引擎（包括已安装的 CCBaga-B）。检测到时，每个档案可以用独立 `user-data-dir`、稳定指纹种子、目标地区语言/时区和 WebRTC UDP 限制参数启动；普通 Chrome/Edge 独立 Profile 不会被标成指纹浏览器。当前 safecc 自身安装包不分发第三方 Chromium 二进制，Windows 指纹引擎尚未打包；引擎缺失时仍使用普通独立 Chromium Profile 隔离 Cookie 和站点数据。

Windows x64 构建入口已固定为 `scripts/build-windows.ps1`（也可双击 `scripts/build-windows.cmd`）。它会检查 Rust MSVC 目标、执行 `npm ci`、前端构建和 Tauri NSIS 打包，最后输出安装包 SHA-256。Windows 机器上运行：

```powershell
./scripts/build-windows.ps1
```

同样的流程已放入 `.github/workflows/build-windows.yml`，可手动触发或推送 `v*` 标签后生成 `safecc-windows-x64` 构建产物。当前工作区还提供了本机交叉构建产物，已生成但尚未在真实 Windows 机器启动验收：[`artifacts/safecc-windows-x64-setup.exe`](artifacts/safecc-windows-x64-setup.exe)。

如果需要在 macOS 主机本地生成 Windows x64 安装包，可先安装 `rustup`、`cargo-zigbuild`、`zig`、`makensis` 和 `mingw-w64`，再运行：

```bash
./scripts/build-windows-cross.sh
```

该本地交叉构建使用 `x86_64-pc-windows-gnu`，输出 `artifacts/safecc-windows-x64-setup.exe` 和对应的 SHA-256 文件；当前 SHA-256 为 `11fa742668e5ac6b784fb425a34d49c0cdb1b05b379da4468c34c4fff5da779b`。Windows 机器与 CI 仍优先使用 MSVC/NSIS 流程。安装包需要在真实 Windows 机器上启动并完成系统权限、Clash 运行态和检测页复核。

