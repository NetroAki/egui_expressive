@echo off
REM install_zxp.bat — Install egui_expressive Exporter into Adobe Illustrator
REM
REM Usage:
REM   install_zxp.bat                          Install from default dist/ location
REM   install_zxp.bat C:\path\to\plugin.zxp   Install specific .zxp file
REM   set EGUI_EXPRESSIVE_ENABLE_CEP_DEBUG=1   Opt in to CEP debug-mode registry writes for self-signed/internal installs only

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
    set "EGUI_EXPRESSIVE_ZXP_FILE=%ZXP_FILE%"
    set "EGUI_EXPRESSIVE_EXT_DIR=!EXT_DIR!"
    powershell -NoProfile -Command "Expand-Archive -LiteralPath $env:EGUI_EXPRESSIVE_ZXP_FILE -DestinationPath $env:EGUI_EXPRESSIVE_EXT_DIR -Force"
    if errorlevel 1 (
        echo [ERROR] Failed to extract .zxp file.
        exit /b 1
    )
    
    echo [SUCCESS] Extension extracted successfully.
    echo [INFO] Restart Illustrator to load the extension.
)

if "%EGUI_EXPRESSIVE_ENABLE_CEP_DEBUG%"=="1" goto enable_debug_modes
echo [INFO] CEP debug mode was not changed.
echo [INFO] Set EGUI_EXPRESSIVE_ENABLE_CEP_DEBUG=1 before running this helper only when a self-signed/internal CEP install requires it.
goto debug_modes_done

:enable_debug_modes
echo [WARN] Enabling CEP debug mode because EGUI_EXPRESSIVE_ENABLE_CEP_DEBUG=1.
echo [WARN] Use this only for explicitly approved self-signed/internal installs.
call :enable_debug 9
call :enable_debug 10
call :enable_debug 11
call :enable_debug 12
call :enable_debug 13
call :enable_debug 14
call :enable_debug 15
call :enable_debug 16
call :enable_debug 17
call :enable_debug 18
call :enable_debug 19
call :enable_debug 20

:debug_modes_done
endlocal
exit /b 0

:enable_debug
reg add "HKCU\SOFTWARE\Adobe\CSXS.%~1" /v PlayerDebugMode /t REG_SZ /d 1 /f >nul 2>&1
if %ERRORLEVEL% equ 0 (
    echo [INFO]   CSXS.%~1 debug mode enabled.
) else (
    echo [WARN]   CSXS.%~1 registry write failed.
)
goto :eof
