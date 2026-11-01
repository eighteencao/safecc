$ErrorActionPreference = "Stop"

Write-Host "safecc Windows build"

if ($env:OS -ne "Windows_NT") {
  throw "This script must run on Windows."
}

foreach ($command in @("node", "npm", "cargo", "rustc", "rustup")) {
  if (-not (Get-Command $command -ErrorAction SilentlyContinue)) {
    throw "Missing required command: $command"
  }
}

$rustTarget = "x86_64-pc-windows-msvc"
$installedTargets = rustup target list --installed 2>$null
if ($LASTEXITCODE -ne 0 -or $installedTargets -notcontains $rustTarget) {
  Write-Host "Installing Rust target $rustTarget..."
  rustup target add $rustTarget
}

Write-Host "Installing locked JavaScript dependencies..."
npm ci

Write-Host "Running frontend typecheck and production build..."
npm run build

Write-Host "Building safecc NSIS installer..."
npm run tauri:build -- --target $rustTarget --bundles nsis

$installer = Get-ChildItem -Path "src-tauri/target/$rustTarget/release/bundle/nsis" -Filter "*.exe" -File | Select-Object -First 1
if (-not $installer) {
  throw "The NSIS installer was not produced."
}

Write-Host "Installer: $($installer.FullName)"
Write-Host "SHA256: $((Get-FileHash -Algorithm SHA256 $installer.FullName).Hash)"
