# Installs the toolchain and system libraries needed to build the Mhaol Cloud
# Tauri shell + the mhaol-cloud backend bin on Windows 10 / 11.
#
# Run from an *elevated* PowerShell:
#   Set-ExecutionPolicy -Scope Process Bypass -Force
#   .\scripts\setup-windows.ps1
#
# Idempotent: safe to re-run. Only installs what is missing.

#Requires -Version 5.1
$ErrorActionPreference = "Stop"

function Test-Cmd($name) { return [bool](Get-Command $name -ErrorAction SilentlyContinue) }

function Install-WingetPackage($id, [string]$override = "") {
    Write-Host "[setup-windows] winget install $id"
    $args = @("install", "--id", $id, "-e", "--accept-package-agreements", "--accept-source-agreements", "--silent")
    if ($override -ne "") { $args += @("--override", $override) }
    & winget @args
    if ($LASTEXITCODE -ne 0 -and $LASTEXITCODE -ne -1978335189) {
        # -1978335189 = APPINSTALLER_CLI_ERROR_NO_APPLICABLE_UPGRADE (already installed). Treat as success.
        Write-Warning "winget exit $LASTEXITCODE for $id — verify manually if the install failed."
    }
}

if (-not (Test-Cmd winget)) {
    throw "winget not found. Install 'App Installer' from the Microsoft Store, then re-run."
}

# 1. Visual Studio 2022 Build Tools + the 'Desktop development with C++' workload (gives us MSVC + Win SDK).
Install-WingetPackage "Microsoft.VisualStudio.2022.BuildTools" `
    "--quiet --wait --add Microsoft.VisualStudio.Workload.VCTools --includeRecommended"

# 2. WebView2 runtime — usually preinstalled on Win11; harmless on Win10.
Install-WingetPackage "Microsoft.EdgeWebView2Runtime"

# 3. Rust via rustup (defaults to the MSVC toolchain, which is what we want).
if (-not (Test-Cmd rustup)) { Install-WingetPackage "Rustlang.Rustup" }

# 4. Node LTS + pnpm via Corepack.
if (-not (Test-Cmd node)) { Install-WingetPackage "OpenJS.NodeJS.LTS" }

# 5. NSIS — Tauri's default Windows installer bundler.
Install-WingetPackage "NSIS.NSIS"

# 6. GStreamer MSVC (runtime + development) — required by packages/ipfs-stream.
#    The winget package installs the MSVC variant. If it fails, grab the two MSVC MSIs from
#    https://gstreamer.freedesktop.org/download/#windows manually (runtime + development).
Install-WingetPackage "GStreamer.GStreamer"
Install-WingetPackage "GStreamer.GStreamer.Development"

Write-Host ""
Write-Host "[setup-windows] reload PATH so the new toolchain is visible to this shell..."
$env:Path = [System.Environment]::GetEnvironmentVariable("Path", "Machine") + ";" + `
            [System.Environment]::GetEnvironmentVariable("Path", "User")

# Corepack + pnpm.
if (Test-Cmd corepack) {
    & corepack enable
    & corepack prepare pnpm@latest --activate
} else {
    Write-Warning "corepack not on PATH yet — open a new shell and run 'corepack enable; corepack prepare pnpm@latest --activate' manually."
}

# Tauri CLI.
if (-not (Test-Cmd cargo-tauri)) {
    if (Test-Cmd cargo) {
        & cargo install tauri-cli --locked --version "^2"
    } else {
        Write-Warning "cargo not on PATH yet — open a new shell and run 'cargo install tauri-cli --locked --version ^2' manually."
    }
}

Write-Host ""
Write-Host "[setup-windows] done."
Write-Host "  - Open a fresh PowerShell so PATH / GSTREAMER_1_0_ROOT_MSVC_X86_64 are picked up."
Write-Host "  - Then run: pnpm install; pnpm build:dist"
