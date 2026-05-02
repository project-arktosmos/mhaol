#!/usr/bin/env bash
# Boots the Google_TV_1080p_API_36 AVD (or reuses an already-running emulator),
# then runs the cloud backend bin, the Vite dev server, and
# `cargo tauri android dev` against the apps/android-tv crate.
#
# apps/android-tv does NOT embed a backend — it is a pure viewer shell. The
# emulator points back at the host's localhost via `10.0.2.2`, so the cloud
# backend strand here gives the TV shell something real to talk to during dev.
# In production the TV shell expects the user to set the backend URL via the
# in-app Settings page.

set -euo pipefail

REPO_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$REPO_ROOT"

AVD_NAME="Google_TV_1080p_API_36"

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

pnpm clean:ports:cloud

if ! "$ADB" devices 2>/dev/null | awk 'NR>1 && $1 ~ /^emulator-/ && $2 == "device" {found=1} END {exit !found}'; then
  echo "[android-tv] booting AVD $AVD_NAME"
  nohup "$EMULATOR" -avd "$AVD_NAME" -no-snapshot-save >logs/android-tv-emulator.log 2>&1 &
  echo "[android-tv] waiting for device"
  "$ADB" wait-for-device
  until [ "$("$ADB" shell getprop sys.boot_completed 2>/dev/null | tr -d '\r')" = "1" ]; do
    sleep 2
  done
  echo "[android-tv] boot complete"
else
  echo "[android-tv] reusing already-running emulator"
fi

if [ ! -d apps/android-tv/gen/android ]; then
  echo "[android-tv] running 'cargo tauri android init' (one-time)"
  (cd apps/android-tv && cargo tauri android init)
fi

CARGO_BUILD_JOBS="${CARGO_BUILD_JOBS:-$(( ( $(sysctl -n hw.ncpu 2>/dev/null || nproc 2>/dev/null || echo 4) + 1 ) / 2 ))}"
export CARGO_BUILD_JOBS
cargo build -p mhaol-backend --bin mhaol-cloud

exec pnpm exec concurrently \
  --kill-others-on-fail \
  --names cloud,web,android-tv \
  --prefix-colors blue,green,magenta \
  "pnpm run app:cloud:bin" \
  "pnpm run app:cloud:web" \
  "pnpm run app:tauri:android:tv"
