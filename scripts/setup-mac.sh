#!/usr/bin/env bash
# Installs the toolchain and system libraries needed to build the Mhaol Cloud
# Tauri shell + the mhaol-cloud backend bin on macOS.
#
# Idempotent: safe to re-run. Only installs what is missing.

set -euo pipefail

if [[ "$(uname -s)" != "Darwin" ]]; then
  echo "[setup-mac] this script is for macOS; use scripts/setup-linux.sh or scripts/setup-windows.ps1" >&2
  exit 1
fi

# Xcode Command Line Tools — Tauri's macOS requirement, also gives us clang for cargo.
if ! xcode-select -p >/dev/null 2>&1; then
  echo "[setup-mac] installing Xcode Command Line Tools (a GUI prompt will appear)"
  xcode-select --install || true
fi

if ! command -v brew >/dev/null 2>&1; then
  echo "[setup-mac] Homebrew is required. Install from https://brew.sh and re-run this script." >&2
  exit 1
fi

# GStreamer — required by packages/ipfs-stream (mhaol-cloud bin), not by the Tauri shell itself.
brew list --formula gstreamer >/dev/null 2>&1 || brew install gstreamer

# Rust toolchain.
if ! command -v cargo >/dev/null 2>&1; then
  echo "[setup-mac] installing Rust via rustup"
  curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y
  # shellcheck disable=SC1091
  source "$HOME/.cargo/env"
fi

# Node + pnpm via Corepack.
if ! command -v node >/dev/null 2>&1; then
  brew install node
fi
corepack enable
corepack prepare pnpm@latest --activate

# Tauri CLI (cargo subcommand).
if ! cargo tauri --version >/dev/null 2>&1; then
  cargo install tauri-cli --locked --version "^2"
fi

echo "[setup-mac] done — run 'pnpm install && pnpm build:dist' to produce the Tauri bundle"
