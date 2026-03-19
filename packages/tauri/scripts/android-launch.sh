#!/usr/bin/env bash
set -e

DEVICE_TYPE=$1
APK="src-tauri/gen/android/app/build/outputs/apk/universal/release/app-universal-release.apk"
PACKAGE="com.arktosmos.mhaol"

if [ -z "$DEVICE_TYPE" ]; then
  echo "Usage: $0 <device-type> (e.g. tv, phone, tablet)"
  exit 1
fi

SERIAL=""
for s in $(adb devices | grep emulator | awk '{print $1}'); do
  avd_name=$(adb -s "$s" emu avd name 2>/dev/null | head -1 | tr -d '\r')
  if echo "$avd_name" | grep -qi "$DEVICE_TYPE"; then
    SERIAL="$s"
    break
  fi
done

if [ -z "$SERIAL" ]; then
  echo "No running $DEVICE_TYPE emulator found. Start an AVD with '$DEVICE_TYPE' in its name and try again."
  exit 1
fi

AVD_NAME=$(adb -s "$SERIAL" emu avd name 2>/dev/null | head -1 | tr -d '\r')
echo "Deploying to $DEVICE_TYPE emulator: $SERIAL ($AVD_NAME)"

adb -s "$SERIAL" install -r "$APK" || (adb -s "$SERIAL" uninstall "$PACKAGE" && adb -s "$SERIAL" install "$APK")
adb -s "$SERIAL" shell am start -n "$PACKAGE/.MainActivity"
