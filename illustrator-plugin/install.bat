@echo off
setlocal enabledelayedexpansion

echo ============================================
echo  egui_expressive Illustrator Plugin Installer
echo ============================================
echo.

set UPIA="C:\Program Files\Common Files\Adobe\Adobe Desktop Common\RemoteComponents\UPI\UnifiedPluginInstallerAgent\UnifiedPluginInstallerAgent.exe"

for %%f in ("%~dp0..\dist\*.zxp") do set ZXP_FILE=%%f
if not defined ZXP_FILE (
  echo ERROR: No .zxp found in dist\. Run installer\build_zxp.bat first.
  exit /b 1
)

if exist %UPIA% (
    echo Found UPIA. Installing ZXP...
    %UPIA% /install "%ZXP_FILE%"
) else (
    echo ERROR: UPIA not found.
    echo Please use the installer\install_zxp.bat script instead.
)

echo Enabling CEP debug mode for self-signed extensions...
reg add "HKCU\Software\Adobe\CSXS.10" /v PlayerDebugMode /t REG_SZ /d 1 /f >nul 2>&1
reg add "HKCU\Software\Adobe\CSXS.11" /v PlayerDebugMode /t REG_SZ /d 1 /f >nul 2>&1
reg add "HKCU\Software\Adobe\CSXS.12" /v PlayerDebugMode /t REG_SZ /d 1 /f >nul 2>&1
reg add "HKCU\Software\Adobe\CSXS.13" /v PlayerDebugMode /t REG_SZ /d 1 /f >nul 2>&1
reg add "HKCU\Software\Adobe\CSXS.14" /v PlayerDebugMode /t REG_SZ /d 1 /f >nul 2>&1
reg add "HKCU\Software\Adobe\CSXS.15" /v PlayerDebugMode /t REG_SZ /d 1 /f >nul 2>&1
echo CEP debug mode enabled (CSXS.10 through CSXS.15).

echo.
pause
