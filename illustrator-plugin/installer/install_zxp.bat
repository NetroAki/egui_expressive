@echo off
REM install_zxp.bat — Install egui_expressive Exporter into Adobe Illustrator
REM
REM Usage:
REM   install_zxp.bat                          Install from default dist/ location
REM   install_zxp.bat C:\path\to\plugin.zxp   Install specific .zxp file

setlocal enabledelayedexpansion

set "SCRIPT_DIR=%~dp0"
set "PLUGIN_DIR=%SCRIPT_DIR%.."
set "PROJECT_ROOT=%PLUGIN_DIR%\.."
set "OUTPUT_DIR=%PROJECT_ROOT%\dist"

set "ZXP_FILE="
if not "%~1"=="" (
    set "ZXP_FILE=%~1"
) else (
    for %%F in ("%OUTPUT_DIR%\egui_expressive_export-*.zxp") do (
        set "ZXP_FILE=%%F"
    )
)

if not defined ZXP_FILE (
    echo [ERROR] No .zxp file found in %OUTPUT_DIR%
    echo Please run build_zxp.bat first or provide a path to a .zxp file.
    exit /b 1
)

if not exist "%ZXP_FILE%" (
    echo [ERROR] File not found: %ZXP_FILE%
    exit /b 1
)

echo [INFO] Installing: %ZXP_FILE%

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

if defined UPIA_PATH (
    echo [INFO] Found UPIA: !UPIA_PATH!
    "!UPIA_PATH!" /install "%ZXP_FILE%"
    if errorlevel 1 (
        echo [ERROR] UPIA installation failed.
        exit /b 1
    )
    echo [SUCCESS] Extension installed successfully.
) else (
    echo [WARN] UnifiedPluginInstallerAgent ^(UPIA^) not found.
    echo [INFO] Falling back to manual extraction...
    
    set "EXT_DIR=%APPDATA%\Adobe\CEP\extensions\com.egui-expressive.illustrator-exporter"
    
    if not exist "!EXT_DIR!" (
        mkdir "!EXT_DIR!"
    )
    
    echo [INFO] Extracting to: !EXT_DIR!
    powershell -Command "Expand-Archive -Path '%ZXP_FILE%' -DestinationPath '!EXT_DIR!' -Force"
    if errorlevel 1 (
        echo [ERROR] Failed to extract .zxp file.
        exit /b 1
    )
    
    echo [SUCCESS] Extension extracted successfully.
    echo [INFO] Restart Illustrator to load the extension.
)

REM Enable CEP debug mode for self-signed extensions (no admin required)
echo [INFO] Enabling CEP debug mode for self-signed extensions...
reg add "HKCU\SOFTWARE\Adobe\CSXS.10" /v PlayerDebugMode /t REG_SZ /d 1 /f >nul 2>&1
reg add "HKCU\SOFTWARE\Adobe\CSXS.11" /v PlayerDebugMode /t REG_SZ /d 1 /f >nul 2>&1
reg add "HKCU\SOFTWARE\Adobe\CSXS.12" /v PlayerDebugMode /t REG_SZ /d 1 /f >nul 2>&1
reg add "HKCU\SOFTWARE\Adobe\CSXS.13" /v PlayerDebugMode /t REG_SZ /d 1 /f >nul 2>&1
reg add "HKCU\SOFTWARE\Adobe\CSXS.14" /v PlayerDebugMode /t REG_SZ /d 1 /f >nul 2>&1
reg add "HKCU\SOFTWARE\Adobe\CSXS.15" /v PlayerDebugMode /t REG_SZ /d 1 /f >nul 2>&1
echo [INFO] CEP debug mode enabled (CSXS.10-15). Restart Illustrator for changes to take effect.

endlocal
