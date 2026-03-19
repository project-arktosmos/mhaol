#!/usr/bin/env bash
set -e

APP_DIR="$1"
if [ -z "$APP_DIR" ]; then
  echo "Usage: $0 <app-dir> (e.g., apps/tube)"
  exit 1
fi

ASSETS_DIR="$(cd "$(dirname "$0")/../assets" && pwd)"

cp -r "$ASSETS_DIR/icons" "$APP_DIR/src-tauri/"
cp -r "$ASSETS_DIR/capabilities" "$APP_DIR/src-tauri/"
cp -r "$ASSETS_DIR/loading" "$APP_DIR/src-tauri/"
