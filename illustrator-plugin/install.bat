@echo off
REM install.bat — Per-user CEP extension installer (no admin required)
REM Extracts .zxp to APPDATA and enables debug mode

setlocal EnableExtensions EnableDelayedExpansion

echo ============================================
echo  egui_expressive Illustrator Plugin Installer
echo ============================================
echo.

REM --- Resolve ZXP path (outside blocks) ---
set "SCRIPT_DIR=%~dp0"
set "ZXP_FILE=!SCRIPT_DIR!egui_expressive_export-1.0.0.zxp"
if not exist "!ZXP_FILE!" (
    set "ZXP_FILE=!SCRIPT_DIR!..\dist\egui_expressive_export-1.0.0.zxp"
)
if not exist "!ZXP_FILE!" (
    echo [ERROR] egui_expressive_export-1.0.0.zxp not found.
    echo [ERROR] Place the .zxp next to this script, or run build_zxp.bat first.
    pause
    exit /b 1
)

echo [INFO] Found: !ZXP_FILE!

REM --- Extension identity ---
set "EXT_ID=com.egui-expressive.illustrator-exporter"
set "EXT_NAME=egui_expressive Exporter"

REM --- Uninstall: discover and remove from ALL known locations ---
echo [INFO] Removing previous version...
set "FOUND_OLD=0"

REM Check all known CEP extension paths — skip if no write permission
for %%L in (
    "!APPDATA!\Adobe\CEP\extensions"
    "!ProgramData!\Adobe\CEP\extensions"
    "!ProgramFiles!\Common Files\Adobe\CEP\extensions"
    "!ProgramFiles(x86)!\Common Files\Adobe\CEP\extensions"
) do (
    set "OLD_DIR=%%~L\!EXT_ID!"
    if exist "!OLD_DIR!" (
        echo [INFO]   Found old install at: !OLD_DIR!
        rmdir /s /q "!OLD_DIR!" > nul 2>&1
        if !errorlevel! neq 0 (
            echo [WARN]   Could not delete !OLD_DIR! (may need admin rights or Illustrator is running).
        ) else (
            set "FOUND_OLD=1"
            echo [INFO]   Deleted.
        )
    )
)

if !FOUND_OLD! equ 0 (
    echo [INFO] No previous installation found.
)

REM Also try UPIA unregister if available (optional cleanup)
set "UPIA_PATH="
for %%P in (
    "!ProgramFiles!\Common Files\Adobe\Adobe Desktop Common\RemoteComponents\UPI\UnifiedPluginInstallerAgent\UnifiedPluginInstallerAgent.exe"
    "!ProgramFiles(x86)!\Common Files\Adobe\Adobe Desktop Common\RemoteComponents\UPI\UnifiedPluginInstallerAgent\UnifiedPluginInstallerAgent.exe"
    "!LOCALAPPDATA!\Adobe\Adobe Desktop Common\RemoteComponents\UPI\UnifiedPluginInstallerAgent\UnifiedPluginInstallerAgent.exe"
    "!APPDATA!\Adobe\Adobe Desktop Common\RemoteComponents\UPI\UnifiedPluginInstallerAgent\UnifiedPluginInstallerAgent.exe"
) do (
    if exist "%%~P" (
        set "UPIA_PATH=%%~P"
        goto :upia_found
    )
)
:upia_found

if defined UPIA_PATH (
    echo [INFO] Unregistering from Adobe extension database...
    "!UPIA_PATH!" /remove "!EXT_ID!"
    if !errorlevel! neq 0 (
        echo [WARN] UPIA unregister failed (non-critical, extension folder already deleted).
    ) else (
        echo [INFO] Unregister done.
    )
)

REM --- Install: per-user location (no admin) ---
set "EXT_DIR=!APPDATA!\Adobe\CEP\extensions\!EXT_ID!"
echo [INFO] Installing to: !EXT_DIR!
if not exist "!EXT_DIR!" mkdir "!EXT_DIR!"

REM Pass paths via env vars to avoid PowerShell quoting issues with apostrophes
set "ZXP_SRC=!ZXP_FILE!"
set "ZXP_DST=!EXT_DIR!"
powershell -NoProfile -Command "$src = $env:ZXP_SRC; $dst = $env:ZXP_DST; Expand-Archive -LiteralPath $src -DestinationPath $dst -Force"
if !errorlevel! neq 0 (
    echo [ERROR] Extraction failed.
    echo [ERROR] If the path contains special characters, move the .zxp to a simple path like C:\temp\
    pause
    exit /b 1
)
echo [INFO] Extraction complete.

REM --- Enable CEP debug mode (HKCU = no admin) ---
echo [INFO] Enabling CEP debug mode for self-signed extensions...
for %%V in (10 11 12 13 14 15) do (
    reg add "HKCU\SOFTWARE\Adobe\CSXS.%%V" /v PlayerDebugMode /t REG_SZ /d 1 /f
    if !errorlevel! equ 0 (
        echo [INFO]   CSXS.%%V debug mode enabled.
    ) else (
        echo [WARN]   CSXS.%%V registry write failed.
    )
)

echo.
echo ============================================
echo  Installation complete!
echo.
echo  Restart Illustrator and open:
echo  Window - Extensions - egui_expressive Export
echo ============================================
pause

endlocal
