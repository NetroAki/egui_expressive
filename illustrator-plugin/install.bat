@echo off
REM install.bat - Per-user CEP extension installer (no admin required)
REM Extracts .zxp to %%APPDATA%%\Adobe\CEP\extensions\<bundleId>

setlocal EnableExtensions DisableDelayedExpansion

set "EXT_ID=com.egui-expressive.illustrator-exporter"
set "EXT_NAME=egui_expressive Exporter"
set "SCRIPT_DIR=%~dp0"
set "ZXP_FILE=%SCRIPT_DIR%egui_expressive_export-1.0.0.zxp"

echo ============================================
echo  %EXT_NAME% Installer
echo ============================================
echo.

if exist "%ZXP_FILE%" goto zxp_found
set "ZXP_FILE=%SCRIPT_DIR%..\dist\egui_expressive_export-1.0.0.zxp"
if exist "%ZXP_FILE%" goto zxp_found
echo [ERROR] egui_expressive_export-1.0.0.zxp not found.
echo [ERROR] Place the .zxp next to this script, or run installer\build_zxp.bat first.
pause
exit /b 1

:zxp_found
echo [INFO] Found: %ZXP_FILE%

set "UPIA_PATH="
if exist "%ProgramFiles%\Common Files\Adobe\Adobe Desktop Common\RemoteComponents\UPI\UnifiedPluginInstallerAgent\UnifiedPluginInstallerAgent.exe" set "UPIA_PATH=%ProgramFiles%\Common Files\Adobe\Adobe Desktop Common\RemoteComponents\UPI\UnifiedPluginInstallerAgent\UnifiedPluginInstallerAgent.exe"
if defined UPIA_PATH goto upia_found
if exist "%ProgramFiles(x86)%\Common Files\Adobe\Adobe Desktop Common\RemoteComponents\UPI\UnifiedPluginInstallerAgent\UnifiedPluginInstallerAgent.exe" set "UPIA_PATH=%ProgramFiles(x86)%\Common Files\Adobe\Adobe Desktop Common\RemoteComponents\UPI\UnifiedPluginInstallerAgent\UnifiedPluginInstallerAgent.exe"
if defined UPIA_PATH goto upia_found
if exist "%LOCALAPPDATA%\Adobe\Adobe Desktop Common\RemoteComponents\UPI\UnifiedPluginInstallerAgent\UnifiedPluginInstallerAgent.exe" set "UPIA_PATH=%LOCALAPPDATA%\Adobe\Adobe Desktop Common\RemoteComponents\UPI\UnifiedPluginInstallerAgent\UnifiedPluginInstallerAgent.exe"
if defined UPIA_PATH goto upia_found
if exist "%APPDATA%\Adobe\Adobe Desktop Common\RemoteComponents\UPI\UnifiedPluginInstallerAgent\UnifiedPluginInstallerAgent.exe" set "UPIA_PATH=%APPDATA%\Adobe\Adobe Desktop Common\RemoteComponents\UPI\UnifiedPluginInstallerAgent\UnifiedPluginInstallerAgent.exe"

:upia_found
echo [INFO] Removing previous version...
call :remove_required "%APPDATA%\Adobe\CEP\extensions\%EXT_ID%"
if errorlevel 1 exit /b 1
call :remove_optional "%ProgramData%\Adobe\CEP\extensions\%EXT_ID%"
call :remove_optional "%ProgramFiles%\Common Files\Adobe\CEP\extensions\%EXT_ID%"
call :remove_optional "%ProgramFiles(x86)%\Common Files\Adobe\CEP\extensions\%EXT_ID%"
call :try_upia_remove

set "EXT_DIR=%APPDATA%\Adobe\CEP\extensions\%EXT_ID%"
echo [INFO] Installing to: %EXT_DIR%
if exist "%EXT_DIR%" goto ext_dir_ready
mkdir "%EXT_DIR%"
if exist "%EXT_DIR%" goto ext_dir_ready
echo [ERROR] Failed to create install folder:
echo [ERROR]   %EXT_DIR%
pause
exit /b 1

:ext_dir_ready
set "ZXP_SRC=%ZXP_FILE%"
set "ZXP_DST=%EXT_DIR%"
powershell -NoProfile -Command "$src = $env:ZXP_SRC; $dst = $env:ZXP_DST; Expand-Archive -LiteralPath $src -DestinationPath $dst -Force"
if %ERRORLEVEL% equ 0 goto extracted_ok
echo [ERROR] Extraction failed.
echo [ERROR] Source: %ZXP_FILE%
echo [ERROR] Destination: %EXT_DIR%
pause
exit /b 1

:extracted_ok
echo [INFO] Extraction complete.
echo [INFO] Enabling CEP debug mode for self-signed extensions...
call :enable_debug 10
call :enable_debug 11
call :enable_debug 12
call :enable_debug 13
call :enable_debug 14
call :enable_debug 15

echo.
echo ============================================
echo  Installation complete!
echo.
echo  Restart Illustrator and open:
echo  Window - Extensions - egui_expressive Export
echo ============================================
pause
exit /b 0

:remove_required
set "TARGET=%~1"
if not exist "%TARGET%" goto :eof
echo [INFO]   Found old per-user install at: %TARGET%
rd /s /q "%TARGET%"
if not exist "%TARGET%" goto remove_required_deleted
echo [ERROR]   Failed to delete old per-user install:
echo [ERROR]   %TARGET%
echo [ERROR]   Close Illustrator and try again.
pause
exit /b 1

:remove_required_deleted
echo [INFO]   Deleted.
goto :eof

:remove_optional
set "TARGET=%~1"
if not exist "%TARGET%" goto :eof
echo [INFO]   Found old system install at: %TARGET%
rd /s /q "%TARGET%"
if not exist "%TARGET%" goto remove_optional_deleted
echo [WARN]   Could not delete old system install:
echo [WARN]   %TARGET%
echo [WARN]   Continuing with per-user install.
goto :eof

:remove_optional_deleted
echo [INFO]   Deleted.
goto :eof

:try_upia_remove
if not defined UPIA_PATH goto :eof
echo [INFO] Unregistering from Adobe extension database...
"%UPIA_PATH%" /remove "%EXT_ID%"
if %ERRORLEVEL% equ 0 (
    echo [INFO] Unregister done.
) else (
    echo [WARN] UPIA unregister failed.
)
goto :eof

:enable_debug
reg add "HKCU\SOFTWARE\Adobe\CSXS.%~1" /v PlayerDebugMode /t REG_SZ /d 1 /f
if %ERRORLEVEL% equ 0 (
    echo [INFO]   CSXS.%~1 debug mode enabled.
) else (
    echo [WARN]   CSXS.%~1 registry write failed.
)
goto :eof
