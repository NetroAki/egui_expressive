@echo off
REM install.bat - Install egui_expressive Exporter into Adobe Illustrator
REM Self-contained: place this .bat next to the .zxp file

setlocal EnableExtensions EnableDelayedExpansion

echo ============================================
echo  egui_expressive Illustrator Plugin Installer
echo ============================================
echo.

REM Resolve paths OUTSIDE any blocks to avoid %~dp0 expansion issues
set "SCRIPT_DIR=%~dp0"
set "ZXP_FILE=!SCRIPT_DIR!egui_expressive_export-1.0.0.zxp"

REM Look for ZXP next to script first, then in dist/
if not exist "!ZXP_FILE!" (
    set "ZXP_FILE=!SCRIPT_DIR!..\dist\egui_expressive_export-1.0.0.zxp"
)
if not exist "!ZXP_FILE!" (
    echo [ERROR] egui_expressive_export-1.0.0.zxp not found.
    echo [ERROR] Run build_zxp.bat first, or place the .zxp next to this script.
    exit /b 1
)

echo [INFO] Found: !ZXP_FILE!

REM Find UPIA - check each path one at a time, no nesting
set "UPIA_PATH="
set "UPIA_TEST=!ProgramFiles!\Common Files\Adobe\Adobe Desktop Common\RemoteComponents\UPI\UnifiedPluginInstallerAgent\UnifiedPluginInstallerAgent.exe"
if exist "!UPIA_TEST!" set "UPIA_PATH=!UPIA_TEST!"

if not defined UPIA_PATH (
    set "UPIA_TEST=!ProgramFiles(x86)!\Common Files\Adobe\Adobe Desktop Common\RemoteComponents\UPI\UnifiedPluginInstallerAgent\UnifiedPluginInstallerAgent.exe"
    if exist "!UPIA_TEST!" set "UPIA_PATH=!UPIA_TEST!"
)

if not defined UPIA_PATH (
    set "UPIA_TEST=!LOCALAPPDATA!\Adobe\Adobe Desktop Common\RemoteComponents\UPI\UnifiedPluginInstallerAgent\UnifiedPluginInstallerAgent.exe"
    if exist "!UPIA_TEST!" set "UPIA_PATH=!UPIA_TEST!"
)

if not defined UPIA_PATH (
    set "UPIA_TEST=!APPDATA!\Adobe\Adobe Desktop Common\RemoteComponents\UPI\UnifiedPluginInstallerAgent\UnifiedPluginInstallerAgent.exe"
    if exist "!UPIA_TEST!" set "UPIA_PATH=!UPIA_TEST!"
)

REM Remove previous version — always force remove folder, then UPIA unregister
set "EXT_ID=com.egui-expressive.illustrator-exporter"
echo [INFO] Removing previous version...

REM Try UPIA remove first (unregisters from Adobe's extension database)
if defined UPIA_PATH (
    "!UPIA_PATH!" /remove "!EXT_ID!" 2>nul
)

REM Always manually delete the extension folder (guaranteed cleanup)
set "EXT_DIR=!APPDATA!\Adobe\CEP\extensions\!EXT_ID!"
if exist "!EXT_DIR!" (
    rmdir /s /q "!EXT_DIR!"
    echo [INFO] Old extension folder deleted.
) else (
    echo [INFO] No previous extension folder found.
)

REM Also check per-user ProgramData location
set "EXT_DIR2=!ProgramData!\Adobe\CEP\extensions\!EXT_ID!"
if exist "!EXT_DIR2!" (
    rmdir /s /q "!EXT_DIR2!"
    echo [INFO] Old system extension folder deleted.
)

echo [INFO] Previous version removed.

REM Install fresh
set "EXT_DIR=!APPDATA!\Adobe\CEP\extensions\!EXT_ID!"
if defined UPIA_PATH (
    echo [INFO] Installing via UPIA...
    "!UPIA_PATH!" /install "!ZXP_FILE!"
    if !errorlevel! neq 0 (
        echo [WARN] UPIA install failed, trying manual extraction...
        if not exist "!EXT_DIR!" mkdir "!EXT_DIR!"
        echo [INFO] Extracting to: !EXT_DIR!
        powershell -Command "Expand-Archive -Path '!ZXP_FILE!' -DestinationPath '!EXT_DIR!' -Force"
        if !errorlevel! neq 0 (
            echo [ERROR] Failed to extract .zxp file.
            exit /b 1
        )
    )
) else (
    echo [INFO] UPIA not found. Installing via manual extraction...
    if not exist "!EXT_DIR!" mkdir "!EXT_DIR!"
    echo [INFO] Extracting to: !EXT_DIR!
    powershell -Command "Expand-Archive -Path '!ZXP_FILE!' -DestinationPath '!EXT_DIR!' -Force"
    if !errorlevel! neq 0 (
        echo [ERROR] Failed to extract .zxp file.
        exit /b 1
    )
)
echo [SUCCESS] Extension installed.

REM Enable CEP debug mode for self-signed extensions
echo [INFO] Enabling CEP debug mode for self-signed extensions...
reg add "HKCU\SOFTWARE\Adobe\CSXS.10" /v PlayerDebugMode /t REG_SZ /d 1 /f >nul 2>&1
reg add "HKCU\SOFTWARE\Adobe\CSXS.11" /v PlayerDebugMode /t REG_SZ /d 1 /f >nul 2>&1
reg add "HKCU\SOFTWARE\Adobe\CSXS.12" /v PlayerDebugMode /t REG_SZ /d 1 /f >nul 2>&1
reg add "HKCU\SOFTWARE\Adobe\CSXS.13" /v PlayerDebugMode /t REG_SZ /d 1 /f >nul 2>&1
reg add "HKCU\SOFTWARE\Adobe\CSXS.14" /v PlayerDebugMode /t REG_SZ /d 1 /f >nul 2>&1
reg add "HKCU\SOFTWARE\Adobe\CSXS.15" /v PlayerDebugMode /t REG_SZ /d 1 /f >nul 2>&1
echo [INFO] CEP debug mode enabled.

echo.
echo ============================================
echo  Installation complete!
echo  Restart Illustrator and open:
echo  Window - Extensions - egui_expressive Export
echo ============================================
pause

endlocal
