#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
TARGET="x86_64-pc-windows-gnu"
OUTPUT_DIR="$ROOT_DIR/artifacts"
OUTPUT_FILE="$OUTPUT_DIR/safecc-windows-x64-setup.exe"

# Prefer the active rustup toolchain when Homebrew's Rust is also installed.
# Homebrew rustc can report a target-libdir that does not contain rustup's
# installed Windows standard library, which makes cargo-zigbuild fail later.
if command -v rustup >/dev/null 2>&1; then
  rustup_cargo="$(rustup which cargo 2>/dev/null || true)"
  if [[ -n "$rustup_cargo" ]]; then
    rustup_bin_dir="$(dirname "$rustup_cargo")"
    PATH="$rustup_bin_dir:$PATH"
    export PATH
  fi
fi

require_command() {
  local command_name="$1"
  if ! command -v "$command_name" >/dev/null 2>&1; then
    printf 'Missing required command: %s\n' "$command_name" >&2
    exit 1
  fi
}

require_command cargo
require_command rustc
require_command npm
require_command cargo-zigbuild
require_command makensis
require_command x86_64-w64-mingw32-windres

target_libdir="$(rustc --print target-libdir --target "$TARGET" 2>/dev/null || true)"
if [[ ! -d "$target_libdir" ]]; then
  if ! command -v rustup >/dev/null 2>&1; then
    printf 'Rust target %s is missing and rustup is not installed.\n' "$TARGET" >&2
    exit 1
  fi
  rustup target add "$TARGET"
fi

printf 'safecc Windows cross-build (%s)\n' "$TARGET"
npm --prefix "$ROOT_DIR" run build
cargo zigbuild \
  --manifest-path "$ROOT_DIR/src-tauri/Cargo.toml" \
  --release \
  --features tauri/custom-protocol \
  --target "$TARGET"

binary="$ROOT_DIR/src-tauri/target/$TARGET/release/claude_environment_assistant.exe"
webview_loader="$ROOT_DIR/src-tauri/target/$TARGET/release/WebView2Loader.dll"
if [[ ! -f "$binary" || ! -f "$webview_loader" ]]; then
  printf 'Windows executable or WebView2Loader.dll was not produced.\n' >&2
  exit 1
fi

stage_dir="$(mktemp -d "${TMPDIR:-/tmp}/safecc-windows.XXXXXX")"
cleanup() {
  rm -rf "$stage_dir"
}
trap cleanup EXIT

mkdir -p "$stage_dir/deployment/ccbaga-network" "$OUTPUT_DIR"
cp "$binary" "$stage_dir/claude_environment_assistant.exe"
cp "$webview_loader" "$stage_dir/WebView2Loader.dll"
cp -R "$ROOT_DIR/deployment/ccbaga-network/." "$stage_dir/deployment/ccbaga-network/"

makensis \
  "-DAPP_ROOT=$stage_dir" \
  "-DICON_FILE=$ROOT_DIR/src-tauri/icons/icon.ico" \
  "-DOUT_FILE=$OUTPUT_FILE" \
  "$ROOT_DIR/scripts/safecc-windows.nsi"

(cd "$OUTPUT_DIR" && shasum -a 256 "$(basename "$OUTPUT_FILE")") | tee "$OUTPUT_FILE.sha256"
printf 'Installer: %s\n' "$OUTPUT_FILE"
