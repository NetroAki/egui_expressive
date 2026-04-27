@echo off
REM install.bat - Per-user CEP extension installer (no admin required)
REM Extracts .zxp to %%APPDATA%%\Adobe\CEP\extensions\<bundleId>

setlocal EnableExtensions DisableDelayedExpansion

set "EXT_ID=com.egui-expressive.illustrator-exporter"
set "EXT_NAME=egui_expressive Exporter"
set "SCRIPT_DIR=%~dp0"
set "ZXP_FILE="

echo ============================================
echo  %EXT_NAME% Installer
echo ============================================
echo.

if exist "%SCRIPT_DIR%egui_expressive_export-1.0.0-win32.zxp" set "ZXP_FILE=%SCRIPT_DIR%egui_expressive_export-1.0.0-win32.zxp"
if defined ZXP_FILE goto zxp_found
if exist "%SCRIPT_DIR%..\dist\egui_expressive_export-1.0.0-win32.zxp" set "ZXP_FILE=%SCRIPT_DIR%..\dist\egui_expressive_export-1.0.0-win32.zxp"
if defined ZXP_FILE goto zxp_found
echo [ERROR] egui_expressive_export-1.0.0-win32.zxp not found.
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
call :remove_optional "%APPDATA%\Adobe\CEPServiceManager4\extensions\%EXT_ID%"
call :remove_optional "%LOCALAPPDATA%\Adobe\CEPServiceManager4\extensions\%EXT_ID%"
call :remove_optional "%ProgramData%\Adobe\CEP\extensions\%EXT_ID%"
call :remove_optional "%ProgramData%\Adobe\CEPServiceManager4\extensions\%EXT_ID%"
call :remove_optional "%ProgramFiles%\Common Files\Adobe\CEP\extensions\%EXT_ID%"
call :remove_optional "%ProgramFiles%\Common Files\Adobe\CEPServiceManager4\extensions\%EXT_ID%"
call :remove_optional "%ProgramFiles(x86)%\Common Files\Adobe\CEP\extensions\%EXT_ID%"
call :remove_optional "%ProgramFiles(x86)%\Common Files\Adobe\CEPServiceManager4\extensions\%EXT_ID%"
if exist "%ProgramFiles%\Adobe" for /d %%D in ("%ProgramFiles%\Adobe\*") do call :remove_optional "%%~fD\CEP\extensions\%EXT_ID%"
if exist "%ProgramFiles(x86)%\Adobe" for /d %%D in ("%ProgramFiles(x86)%\Adobe\*") do call :remove_optional "%%~fD\CEP\extensions\%EXT_ID%"
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
powershell -NoProfile -Command "$src = $env:ZXP_SRC; $dst = $env:ZXP_DST; Add-Type -AssemblyName System.IO.Compression.FileSystem; try { if (Test-Path -LiteralPath $dst) { Get-ChildItem -LiteralPath $dst -Force | Remove-Item -Recurse -Force }; [System.IO.Compression.ZipFile]::ExtractToDirectory($src, $dst); exit 0 } catch { Write-Host ('[POWERSHELL] ' + $_.Exception.Message); exit 1 }"
if %ERRORLEVEL% equ 0 goto extracted_ok
echo [ERROR] Extraction failed while unpacking the .zxp file.
echo [ERROR] See the PowerShell message above for the exact cause.
echo [ERROR] Source: %ZXP_FILE%
echo [ERROR] Destination: %EXT_DIR%
pause
exit /b 1

:extracted_ok
echo [INFO] Extraction complete.
echo [INFO] Enabling CEP debug mode for self-signed extensions...
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
if errorlevel 1 goto upia_remove_failed
echo [INFO] Unregister done.
goto :eof

:upia_remove_failed
echo [WARN] UPIA unregister failed with exit code %ERRORLEVEL%.
echo [WARN] This usually means Adobe has no registered record for this extension,
echo [WARN] or the extension database entry is stale. Continuing because file-based cleanup already ran.
goto :eof

:enable_debug
reg add "HKCU\SOFTWARE\Adobe\CSXS.%~1" /v PlayerDebugMode /t REG_SZ /d 1 /f
if %ERRORLEVEL% equ 0 (
    echo [INFO]   CSXS.%~1 debug mode enabled.
) else (
    echo [WARN]   CSXS.%~1 registry write failed.
)
goto :eof
