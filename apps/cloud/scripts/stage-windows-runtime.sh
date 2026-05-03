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
  pacman -S --needed --noconfirm mingw-w64-x86_64-ntldd
fi
if ! command -v ntldd >/dev/null 2>&1; then
  echo "::error::stage-windows-runtime: ntldd is required and could not be installed" >&2
  exit 1
fi
if ! command -v node >/dev/null 2>&1; then
  echo "::error::stage-windows-runtime: node is required to emit tauri.windows.conf.json" >&2
  exit 1
fi

# Resolve the build host's mingw64 prefix in unix form so we can match it
# against ntldd's output regardless of whether ntldd emits backslash-style
# Windows paths or forward-slash MSYS paths.
MINGW_PREFIX_UNIX=$(cygpath -u "${MINGW_PREFIX:-/mingw64}" 2>/dev/null || echo "/mingw64")
MINGW_BIN_UNIX="${MINGW_PREFIX_UNIX%/}/bin"

mkdir -p "$RUNTIME_DIR"
rm -f "$RUNTIME_DIR"/*.dll

declare -A SEEN
queue=("$EXE")
NTLDD_LINES=0
while [[ ${#queue[@]} -gt 0 ]]; do
  cur="${queue[0]}"
  queue=("${queue[@]:1}")
  while IFS= read -r line; do
    NTLDD_LINES=$((NTLDD_LINES + 1))
    raw=$(echo "$line" | awk '/=>/ { print $3 }')
    [[ -z "$raw" || "$raw" == "not" ]] && continue
    # Normalize: backslashes → forward slashes, then convert to MSYS path
    # so both `D:\a\_temp\msys64\mingw64\bin\foo.dll` and
    # `/mingw64/bin/foo.dll` collapse to the same comparable form.
    norm="${raw//\\//}"
    unix_path=$(cygpath -u "$norm" 2>/dev/null || echo "$norm")
    case "$unix_path" in
      "$MINGW_BIN_UNIX"/*|*/mingw64/bin/*)
        name=$(basename "$unix_path")
        if [[ -z "${SEEN[$name]:-}" ]]; then
          SEEN[$name]=1
          cp "$unix_path" "$RUNTIME_DIR/"
          queue+=("$unix_path")
        fi
        ;;
    esac
  done < <(ntldd -R "$cur" 2>/dev/null || true)
done

if [[ -z "$(ls -A "$RUNTIME_DIR" 2>/dev/null)" ]]; then
  echo "::error::stage-windows-runtime: ntldd produced $NTLDD_LINES lines but no DLLs matched $MINGW_BIN_UNIX" >&2
  echo "::error::sample ntldd output for $EXE:" >&2
  ntldd -R "$EXE" 2>&1 | head -20 >&2 || true
  exit 1
fi

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
