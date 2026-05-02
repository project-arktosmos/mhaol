#!/usr/bin/env bash
# Installs the toolchain and system libraries needed to build the Mhaol Cloud
# Tauri shell + the mhaol-cloud backend bin on Ubuntu/Debian.
#
# Idempotent: safe to re-run. Only installs what is missing.

set -euo pipefail

if ! command -v apt-get >/dev/null 2>&1; then
  echo "[setup-linux] this script targets Ubuntu/Debian (apt). Adapt the package list for your distro." >&2
  exit 1
fi

sudo apt-get update
sudo apt-get install -y \
  build-essential pkg-config curl wget file ca-certificates \
  libssl-dev libonig-dev \
  libwebkit2gtk-4.1-dev libxdo-dev \
  libayatana-appindicator3-dev librsvg2-dev libsoup-3.0-dev \
  libgstreamer1.0-dev libgstreamer-plugins-base1.0-dev libgstreamer-plugins-bad1.0-dev \
  gstreamer1.0-plugins-base gstreamer1.0-plugins-good gstreamer1.0-plugins-bad \
  gstreamer1.0-plugins-ugly gstreamer1.0-libav

# Rust toolchain.
if ! command -v cargo >/dev/null 2>&1; then
  echo "[setup-linux] installing Rust via rustup"
  curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y
  # shellcheck disable=SC1091
  source "$HOME/.cargo/env"
fi

# Node 20 LTS via NodeSource (apt's nodejs is usually too old for pnpm + Vite).
if ! command -v node >/dev/null 2>&1; then
  curl -fsSL https://deb.nodesource.com/setup_20.x | sudo -E bash -
  sudo apt-get install -y nodejs
fi

# Corepack ships with Node 20; sudo only needed if Node was installed system-wide.
sudo corepack enable
corepack prepare pnpm@latest --activate

# Tauri CLI (cargo subcommand).
if ! cargo tauri --version >/dev/null 2>&1; then
  cargo install tauri-cli --locked --version "^2"
fi

echo "[setup-linux] done — run 'pnpm install && pnpm build:dist' to produce the Tauri bundle"
