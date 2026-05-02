#!/usr/bin/env bash
# Boots the Medium_Phone_API_36.1 AVD (or reuses an already-running emulator),
# then runs `cargo tauri android dev` against the apps/android-mobile crate.
#
# apps/android-mobile embeds the backend itself (`mhaol_backend::run()` in the
# Tauri setup hook), so there is no separate cloud bin / Vite dev server in
# this strand — the SPA is served from `tauri://` inside the WebView and
# talks to `http://127.0.0.1:9898` (the embedded backend).
#
# However, `cargo tauri android dev` still needs `frontendDist` reachable for
# the dev WebView refresh, so we run the Vite dev server alongside.

set -euo pipefail

REPO_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$REPO_ROOT"

AVD_NAME="Medium_Phone_API_36.1"

export ANDROID_HOME="${ANDROID_HOME:-$HOME/Library/Android/sdk}"
export ANDROID_SDK_ROOT="${ANDROID_SDK_ROOT:-$ANDROID_HOME}"
if [ -z "${NDK_HOME:-}" ]; then
  NDK_HOME="$(ls -d "$ANDROID_HOME"/ndk/* 2>/dev/null | sort -V | tail -1 || true)"
  export NDK_HOME
fi
export ANDROID_NDK_ROOT="${ANDROID_NDK_ROOT:-$NDK_HOME}"

if [ -z "${JAVA_HOME:-}" ]; then
  if [ -d "/Applications/Android Studio.app/Contents/jbr/Contents/Home" ]; then
    export JAVA_HOME="/Applications/Android Studio.app/Contents/jbr/Contents/Home"
  fi
fi

ADB="$ANDROID_HOME/platform-tools/adb"
EMULATOR="$ANDROID_HOME/emulator/emulator"

mkdir -p logs

if ! "$ADB" devices 2>/dev/null | awk 'NR>1 && $1 ~ /^emulator-/ && $2 == "device" {found=1} END {exit !found}'; then
  echo "[android-mobile] booting AVD $AVD_NAME"
  nohup "$EMULATOR" -avd "$AVD_NAME" -no-snapshot-save >logs/android-mobile-emulator.log 2>&1 &
  echo "[android-mobile] waiting for device"
  "$ADB" wait-for-device
  until [ "$("$ADB" shell getprop sys.boot_completed 2>/dev/null | tr -d '\r')" = "1" ]; do
    sleep 2
  done
  echo "[android-mobile] boot complete"
else
  echo "[android-mobile] reusing already-running emulator"
fi

if [ ! -d apps/android-mobile/gen/android ]; then
  echo "[android-mobile] running 'cargo tauri android init' (one-time)"
  (cd apps/android-mobile && cargo tauri android init)
fi

CARGO_BUILD_JOBS="${CARGO_BUILD_JOBS:-$(( ( $(sysctl -n hw.ncpu 2>/dev/null || nproc 2>/dev/null || echo 4) + 1 ) / 2 ))}"
export CARGO_BUILD_JOBS

exec pnpm exec concurrently \
  --kill-others-on-fail \
  --names web,android-mobile \
  --prefix-colors green,magenta \
  "pnpm run app:cloud:web" \
  "pnpm run app:tauri:android:mobile"
