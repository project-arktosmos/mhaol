#!/usr/bin/env bash
# Stage MinGW GStreamer/GLib runtime DLLs alongside the Windows .exe so the
# NSIS / MSI installer ships a self-contained runtime. Without this step the
# released .exe fails to launch on clean Windows machines with
# "the code execution cannot proceed because libglib-2.0-0.dll was not found".
#
# CI invokes this between `cargo tauri build --no-bundle` and
# `cargo tauri bundle`, in an MSYS2 shell so ntldd + /mingw64 are available.
#
# Outputs:
#   apps/cloud/runtime-windows/*.dll       — copied DLLs
#   apps/cloud/tauri.windows.conf.json     — bundle.resources map merged on
#                                            top of tauri.conf.json at bundle
#                                            time so each DLL lands at
#                                            $INSTDIR\<dllname>.

set -euo pipefail

case "$(uname -s)" in
  MINGW*|MSYS*|CYGWIN*) ;;
  *) echo "[stage-windows-runtime] non-Windows host, skipping"; exit 0 ;;
esac

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
APP_DIR="$(cd "$SCRIPT_DIR/.." && pwd)"
RUNTIME_DIR="$APP_DIR/runtime-windows"
TARGET_TRIPLE="${TAURI_TARGET:-x86_64-pc-windows-gnu}"
EXE="$APP_DIR/../../target/$TARGET_TRIPLE/release/mhaol-cloud-shell.exe"

if [[ ! -f "$EXE" ]]; then
  EXE="$APP_DIR/../../target/release/mhaol-cloud-shell.exe"
fi
if [[ ! -f "$EXE" ]]; then
  echo "::error::stage-windows-runtime: missing mhaol-cloud-shell.exe — run 'cargo tauri build --no-bundle' first" >&2
  exit 1
fi

if ! command -v ntldd >/dev/null 2>&1; then
  pacman -S --needed --noconfirm mingw-w64-x86_64-ntldd-git
fi
if ! command -v node >/dev/null 2>&1; then
  echo "::error::stage-windows-runtime: node is required to emit tauri.windows.conf.json" >&2
  exit 1
fi

mkdir -p "$RUNTIME_DIR"
rm -f "$RUNTIME_DIR"/*.dll

declare -A SEEN
queue=("$EXE")
while [[ ${#queue[@]} -gt 0 ]]; do
  cur="${queue[0]}"
  queue=("${queue[@]:1}")
  while IFS= read -r line; do
    path=$(echo "$line" | awk '/=>/ { print $3 }')
    [[ -z "$path" || "$path" == "not" ]] && continue
    case "$path" in
      */mingw64/bin/*)
        name=$(basename "$path")
        if [[ -z "${SEEN[$name]:-}" ]]; then
          SEEN[$name]=1
          cp "$path" "$RUNTIME_DIR/"
          queue+=("$path")
        fi
        ;;
    esac
  done < <(ntldd -R "$cur" 2>/dev/null || true)
done

CONF_PATH="$APP_DIR/tauri.windows.conf.json"
RUNTIME_DIR_ABS="$RUNTIME_DIR" CONF_PATH_ABS="$CONF_PATH" node <<'JS'
const fs = require('node:fs');
const path = require('node:path');
const dir = process.env.RUNTIME_DIR_ABS;
const conf = process.env.CONF_PATH_ABS;
const dlls = fs.readdirSync(dir).filter(f => f.toLowerCase().endsWith('.dll')).sort();
if (dlls.length === 0) {
  console.error('::error::stage-windows-runtime: no DLLs were staged');
  process.exit(1);
}
const resources = Object.fromEntries(dlls.map(f => [`runtime-windows/${f}`, f]));
fs.writeFileSync(conf, JSON.stringify({ bundle: { resources } }, null, 2) + '\n');
console.log(`[stage-windows-runtime] wrote ${path.basename(conf)} with ${dlls.length} DLL entries`);
JS

echo "[stage-windows-runtime] staged $(ls "$RUNTIME_DIR" | wc -l | tr -d ' ') DLLs into $RUNTIME_DIR"
