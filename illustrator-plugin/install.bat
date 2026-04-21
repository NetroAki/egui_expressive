@echo off
REM install.bat — Install egui_expressive Exporter into Adobe Illustrator
REM Self-contained: place this .bat next to the .zxp file

setlocal enabledelayedexpansion

echo ============================================
echo  egui_expressive Illustrator Plugin Installer
echo ============================================
echo.

set "ZXP_FILE=%~dp0egui_expressive_export-1.0.0.zxp"
if not exist "%ZXP_FILE%" (
    echo [ERROR] egui_expressive_export-1.0.0.zxp not found in this folder.
    echo Please extract the ZIP and keep all files together.
    exit /b 1
)

echo [INFO] Found: %ZXP_FILE%

REM Find UPIA in common locations
set "UPIA_PATH="
if exist "%ProgramFiles%\Common Files\Adobe\Adobe Desktop Common\RemoteComponents\UPI\UnifiedPluginInstallerAgent\UnifiedPluginInstallerAgent.exe" (
    set "UPIA_PATH=%ProgramFiles%\Common Files\Adobe\Adobe Desktop Common\RemoteComponents\UPI\UnifiedPluginInstallerAgent\UnifiedPluginInstallerAgent.exe"
) else if exist "%ProgramFiles(x86)%\Common Files\Adobe\Adobe Desktop Common\RemoteComponents\UPI\UnifiedPluginInstallerAgent\UnifiedPluginInstallerAgent.exe" (
    set "UPIA_PATH=%ProgramFiles(x86)%\Common Files\Adobe\Adobe Desktop Common\RemoteComponents\UPI\UnifiedPluginInstallerAgent\UnifiedPluginInstallerAgent.exe"
) else if exist "%LOCALAPPDATA%\Adobe\Adobe Desktop Common\RemoteComponents\UPI\UnifiedPluginInstallerAgent\UnifiedPluginInstallerAgent.exe" (
    set "UPIA_PATH=%LOCALAPPDATA%\Adobe\Adobe Desktop Common\RemoteComponents\UPI\UnifiedPluginInstallerAgent\UnifiedPluginInstallerAgent.exe"
) else if exist "%APPDATA%\Adobe\Adobe Desktop Common\RemoteComponents\UPI\UnifiedPluginInstallerAgent\UnifiedPluginInstallerAgent.exe" (
    set "UPIA_PATH=%APPDATA%\Adobe\Adobe Desktop Common\RemoteComponents\UPI\UnifiedPluginInstallerAgent\UnifiedPluginInstallerAgent.exe"
)

REM Remove previous version first
if defined UPIA_PATH (
    echo [INFO] Removing previous version...
    "!UPIA_PATH!" /remove "com.egui-expressive.illustrator-exporter" >nul 2>&1
    echo [INFO] Previous version removed (if present).
) else (
    echo [INFO] Removing previous version manually...
    rmdir /s /q "%APPDATA%\Adobe\CEP\extensions\com.egui-expressive.illustrator-exporter" >nul 2>&1
    echo [INFO] Previous version removed (if present).
)

REM Install
if defined UPIA_PATH (
    echo [INFO] Installing via UPIA...
    "!UPIA_PATH!" /install "%ZXP_FILE%"
    if errorlevel 1 (
        echo [ERROR] UPIA installation failed.
        exit /b 1
    )
    echo [SUCCESS] Extension installed successfully.
) else (
    echo [WARN] UPIA not found. Falling back to manual extraction...
    
    set "EXT_DIR=%APPDATA%\Adobe\CEP\extensions\com.egui-expressive.illustrator-exporter"
    if not exist "!EXT_DIR!" mkdir "!EXT_DIR!"
    
    echo [INFO] Extracting to: !EXT_DIR!
    powershell -Command "Expand-Archive -Path '%ZXP_FILE%' -DestinationPath '!EXT_DIR!' -Force"
    if errorlevel 1 (
        echo [ERROR] Failed to extract .zxp file.
        exit /b 1
    )
    
    echo [SUCCESS] Extension extracted successfully.
)

REM Enable CEP debug mode for self-signed extensions (no admin required)
echo [INFO] Enabling CEP debug mode for self-signed extensions...
reg add "HKCU\SOFTWARE\Adobe\CSXS.10" /v PlayerDebugMode /t REG_SZ /d 1 /f >nul 2>&1
reg add "HKCU\SOFTWARE\Adobe\CSXS.11" /v PlayerDebugMode /t REG_SZ /d 1 /f >nul 2>&1
reg add "HKCU\SOFTWARE\Adobe\CSXS.12" /v PlayerDebugMode /t REG_SZ /d 1 /f >nul 2>&1
reg add "HKCU\SOFTWARE\Adobe\CSXS.13" /v PlayerDebugMode /t REG_SZ /d 1 /f >nul 2>&1
reg add "HKCU\SOFTWARE\Adobe\CSXS.14" /v PlayerDebugMode /t REG_SZ /d 1 /f >nul 2>&1
reg add "HKCU\SOFTWARE\Adobe\CSXS.15" /v PlayerDebugMode /t REG_SZ /d 1 /f >nul 2>&1
echo [INFO] CEP debug mode enabled (CSXS.10-15).

echo.
echo ============================================
echo  Installation complete!
echo  Restart Illustrator and open:
echo  Window ^> Extensions ^> egui_expressive Export
echo ============================================
pause

endlocal
