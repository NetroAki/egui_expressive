@echo off
REM ────────────────────────────────────────────────────────────────────────────
REM build_zxp.bat — Package egui_expressive Exporter as a signed .zxp extension
REM
REM Prerequisites:
REM   - ZXPSignCmd.exe (download from https://github.com/Adobe-CEP/CEP-Resources)
REM     Place in this directory, ..\tools\, or set ZXPSIGNCMD env var.
REM
REM Usage:
REM   build_zxp.bat                    Build with self-signed cert
REM   build_zxp.bat C:\path\cert.p12   Build with existing cert
REM ────────────────────────────────────────────────────────────────────────────
setlocal enabledelayedexpansion

set "SCRIPT_DIR=%~dp0"
set "PLUGIN_DIR=%SCRIPT_DIR%.."
set "PROJECT_ROOT=%PLUGIN_DIR%\.."
set "OUTPUT_DIR=%PROJECT_ROOT%\dist"
set "RELEASE_DIR=%OUTPUT_DIR%\release"

set "EXTENSION_ID=com.egui-expressive.illustrator-exporter"
set "VERSION=1.0.0"
set "PLATFORM=win32"
set "ZXP_NAME=egui_expressive_export-%VERSION%-%PLATFORM%.zxp"

REM Certificate defaults
set "CERT_COUNTRY=US"
set "CERT_STATE=NA"
set "CERT_ORG=egui_expressive"
set "CERT_NAME=egui_expressive Exporter"
if defined ZXP_SIGN_PASSWORD (
    set "CERT_PASSWORD=%ZXP_SIGN_PASSWORD%"
) else (
    set "CERT_PASSWORD=selfsign_%RANDOM%%RANDOM%"
    echo [WARN] No ZXP_SIGN_PASSWORD env var set - using ephemeral password.
)
set "CERT_FILE=%OUTPUT_DIR%\cert.p12"

set "TSA_URL=https://timestamp.digicert.com"
REM Note: If your signer fails with HTTPS, you can try HTTP:
REM set "TSA_URL=http://timestamp.digicert.com"

echo ============================================================
echo   egui_expressive Exporter - .zxp Package Builder
echo ============================================================
echo.

REM Create output directory
if not exist "%OUTPUT_DIR%" mkdir "%OUTPUT_DIR%"
if not exist "%RELEASE_DIR%" mkdir "%RELEASE_DIR%"
del /f /q "%OUTPUT_DIR%\egui_expressive_export-%VERSION%.zxp" 2>nul
del /f /q "%OUTPUT_DIR%\egui_expressive_export-%VERSION%.zip" 2>nul
del /f /q "%OUTPUT_DIR%\egui_expressive_export-%VERSION%-installer.zip" 2>nul
del /f /q "%OUTPUT_DIR%\test_fresh.zxp" 2>nul
del /f /q "%RELEASE_DIR%\egui_expressive_export-%VERSION%.zxp" 2>nul
del /f /q "%RELEASE_DIR%\egui_expressive_export-%VERSION%.zip" 2>nul
del /f /q "%RELEASE_DIR%\egui_expressive_export-%VERSION%-installer.zip" 2>nul
del /f /q "%RELEASE_DIR%\%ZXP_NAME%" 2>nul
del /f /q "%RELEASE_DIR%\egui_expressive_export-%VERSION%-*.zxp" 2>nul
del /f /q "%RELEASE_DIR%\egui_expressive_export-%VERSION%-*-installer.zip" 2>nul

REM ─── Find ZXPSignCmd ─────────────────────────────────────────────────────
set "SIGNER="

if defined ZXPSIGNCMD (
    if exist "%ZXPSIGNCMD%" (
        set "SIGNER=%ZXPSIGNCMD%"
    )
)

if not defined SIGNER (
    where ZXPSignCmd >nul 2>&1
    if !errorlevel! equ 0 (
        set "SIGNER=ZXPSignCmd"
    )
)

if not defined SIGNER (
    if exist "%SCRIPT_DIR%ZXPSignCmd.exe" (
        set "SIGNER=%SCRIPT_DIR%ZXPSignCmd.exe"
    )
)

if not defined SIGNER (
    if exist "%PROJECT_ROOT%\tools\ZXPSignCmd.exe" (
        set "SIGNER=%PROJECT_ROOT%\tools\ZXPSignCmd.exe"
    )
)

if not defined SIGNER (
    echo [ERROR] ZXPSignCmd not found.
    echo.
    echo Download from: https://github.com/Adobe-CEP/CEP-Resources/tree/master/ZXPSignCMD
    echo Place ZXPSignCmd.exe in:
    echo   - %SCRIPT_DIR%
    echo   - %PROJECT_ROOT%\tools\
    echo   - Or set ZXPSIGNCMD environment variable
    echo.
    echo Or install npm wrapper: npm install -g zxp-sign-cmd
    goto :error
)

echo [INFO] Using signer: %SIGNER%

REM ─── Prepare staging directory ───────────────────────────────────────────
set "STAGE=%OUTPUT_DIR%\staging"
if exist "%STAGE%" rmdir /s /q "%STAGE%"
mkdir "%STAGE%\CSXS"

echo [INFO] Staging extension files...

copy "%PLUGIN_DIR%\CSXS\manifest.xml" "%STAGE%\CSXS\manifest.xml" >nul
copy "%PLUGIN_DIR%\index.html" "%STAGE%\index.html" >nul
copy "%PLUGIN_DIR%\plugin.js" "%STAGE%\plugin.js" >nul
if exist "%PLUGIN_DIR%\host.jsx" copy "%PLUGIN_DIR%\host.jsx" "%STAGE%\host.jsx" >nul

echo [INFO] Building bundled ai-parser for win32...
pushd "%PROJECT_ROOT%"
cargo build --release --bin ai-parser
if errorlevel 1 (
    popd
    echo [ERROR] Failed to build ai-parser
    goto :error
)
popd
if not exist "%PROJECT_ROOT%\target\release\ai-parser.exe" (
    echo [ERROR] Built ai-parser binary not found: %PROJECT_ROOT%\target\release\ai-parser.exe
    goto :error
)
mkdir "%STAGE%\bin" 2>nul
mkdir "%STAGE%\bin\win32" 2>nul
copy "%PROJECT_ROOT%\target\release\ai-parser.exe" "%STAGE%\bin\win32\ai-parser.exe" >nul
if errorlevel 1 (
    echo [ERROR] Failed to stage ai-parser.exe
    goto :error
)

REM NOTE: .debug file excluded from production builds.
REM For development, create manually with appropriate Port.

REM ─── Certificate ─────────────────────────────────────────────────────────
set "CERT_PATH=%CERT_FILE%"
set "USER_PROVIDED_CERT=0"
if not "%~1"=="" (
    set "CERT_PATH=%~1"
    set "USER_PROVIDED_CERT=1"
)

if "%USER_PROVIDED_CERT%"=="1" (
    REM User provided explicit cert path — use as-is
    if not exist "%CERT_PATH%" (
        echo [ERROR] Specified certificate not found: %CERT_PATH%
        goto :error
    )
    echo [INFO] Using provided certificate: %CERT_PATH%
) else (
    REM Default self-signed cert — regenerate to avoid password mismatch
    if exist "%CERT_PATH%" (
        echo [INFO] Removing old self-signed cert to regenerate with current password...
        del /f /q "%CERT_PATH%"
    )
    echo [INFO] Generating self-signed certificate...
    "%SIGNER%" -selfSignedCert %CERT_COUNTRY% %CERT_STATE% %CERT_ORG% "%CERT_NAME%" "%CERT_PASSWORD%" "%CERT_PATH%"
    if errorlevel 1 (
        echo [ERROR] Failed to generate certificate
        goto :error
    )
    echo [INFO] Certificate created: %CERT_PATH%
)

REM ─── Sign and package ────────────────────────────────────────────────────
echo [INFO] Signing and packaging...
echo   Input:  %STAGE%
echo   Output: %OUTPUT_DIR%\%ZXP_NAME%

"%SIGNER%" -sign "%STAGE%" "%OUTPUT_DIR%\%ZXP_NAME%" "%CERT_PATH%" "%CERT_PASSWORD%" -tsa %TSA_URL%
if errorlevel 1 (
    echo [WARN] TSA unavailable, signing without timestamp
    "%SIGNER%" -sign "%STAGE%" "%OUTPUT_DIR%\%ZXP_NAME%" "%CERT_PATH%" "%CERT_PASSWORD%"
    if errorlevel 1 (
        echo [ERROR] Failed to sign package
        goto :error
    )
)

REM Verify
echo [INFO] Verifying package...
"%SIGNER%" -verify "%OUTPUT_DIR%\%ZXP_NAME%" 2>nul
if errorlevel 1 (
    echo [WARN] Signature verification failed - package may not install via Creative Cloud
    echo [WARN] Common with self-signed certificates. Manual install will still work.
) else (
    echo [INFO] Signature verified successfully
)

copy "%OUTPUT_DIR%\%ZXP_NAME%" "%RELEASE_DIR%\%ZXP_NAME%" >nul
if errorlevel 1 (
    echo [ERROR] Failed to sync release artifact
    goto :error
)
set "INSTALLER_ZXP_NAME=egui_expressive_export-%VERSION%.zxp"
copy "%OUTPUT_DIR%\%ZXP_NAME%" "%OUTPUT_DIR%\%INSTALLER_ZXP_NAME%" >nul
if errorlevel 1 (
    echo [ERROR] Failed to create Windows installer ZXP alias
    goto :error
)
copy "%OUTPUT_DIR%\%ZXP_NAME%" "%RELEASE_DIR%\%INSTALLER_ZXP_NAME%" >nul
if errorlevel 1 (
    echo [ERROR] Failed to sync Windows installer ZXP alias
    goto :error
)
del /f /q "%RELEASE_DIR%\install.sh" 2>nul
del /f /q "%RELEASE_DIR%\install.bat" 2>nul
del /f /q "%RELEASE_DIR%\install_zxp.bat" 2>nul
copy "%PLUGIN_DIR%\install.bat" "%RELEASE_DIR%\install.bat" >nul
if errorlevel 1 (
    echo [ERROR] Failed to sync Windows install helper
    goto :error
)
echo [INFO] Release artifact synced: %RELEASE_DIR%\%ZXP_NAME%

(
  echo # egui_expressive Illustrator Exporter - %PLATFORM% package
  echo.
  echo This bundle contains the platform-specific ZXP:
  echo.
  echo - %INSTALLER_ZXP_NAME%
  echo.
  echo Install only on a matching %PLATFORM% host. Do not install this ZXP on another platform because the bundled ai-parser binary is platform-specific.
  echo.
  echo ## Install
  echo.
  echo Run install.bat on Windows. The helper expects %INSTALLER_ZXP_NAME% next to the script.
) > "%RELEASE_DIR%\README.md"

set "INSTALLER_ZIP=%OUTPUT_DIR%\egui_expressive_export-%VERSION%-%PLATFORM%-installer.zip"
del /f /q "%INSTALLER_ZIP%" 2>nul
del /f /q "%RELEASE_DIR%\egui_expressive_export-%VERSION%-%PLATFORM%-installer.zip" 2>nul
powershell -NoProfile -Command "$files = @('%OUTPUT_DIR%\%INSTALLER_ZXP_NAME%', '%RELEASE_DIR%\README.md', '%PLUGIN_DIR%\install.bat'); Compress-Archive -LiteralPath $files -DestinationPath '%INSTALLER_ZIP%' -Force"
if errorlevel 1 (
    echo [ERROR] Failed to create installer bundle
    goto :error
)
copy "%INSTALLER_ZIP%" "%RELEASE_DIR%\egui_expressive_export-%VERSION%-%PLATFORM%-installer.zip" >nul
if errorlevel 1 (
    echo [ERROR] Failed to sync installer bundle
    goto :error
)
echo [INFO] Installer bundle synced: %RELEASE_DIR%\egui_expressive_export-%VERSION%-%PLATFORM%-installer.zip

REM Cleanup
if exist "%STAGE%" rmdir /s /q "%STAGE%"

echo.
echo [INFO] Done! Built platform-specific package for %PLATFORM%: %OUTPUT_DIR%\%ZXP_NAME%
echo.
set "UPIA_PATH="
if exist "%ProgramFiles%\Common Files\Adobe\Adobe Desktop Common\RemoteComponents\UPI\UnifiedPluginInstallerAgent\UnifiedPluginInstallerAgent.exe" (
    set "UPIA_PATH=%ProgramFiles%\Common Files\Adobe\Adobe Desktop Common\RemoteComponents\UPI\UnifiedPluginInstallerAgent\UnifiedPluginInstallerAgent.exe"
) else if exist "%ProgramFiles(x86)%\Common Files\Adobe\Adobe Desktop Common\RemoteComponents\UPI\UnifiedPluginInstallerAgent\UnifiedPluginInstallerAgent.exe" (
    set "UPIA_PATH=%ProgramFiles(x86)%\Common Files\Adobe\Adobe Desktop Common\RemoteComponents\UPI\UnifiedPluginInstallerAgent\UnifiedPluginInstallerAgent.exe"
) else if exist "%LOCALAPPDATA%\Adobe\Adobe Desktop Common\RemoteComponents\UPI\UnifiedPluginInstallerAgent\UnifiedPluginInstallerAgent.exe" (
    set "UPIA_PATH=%LOCALAPPDATA%\Adobe\Adobe Desktop Common\RemoteComponents\UPI\UnifiedPluginInstallerAgent\UnifiedPluginInstallerAgent.exe"
)

if defined UPIA_PATH (
    echo Install with:
    echo   UPIA:  "!UPIA_PATH!" /install "%OUTPUT_DIR%\%ZXP_NAME%"
) else (
    echo Install manually:
    echo   1. Extract .zxp ^(it's a ZIP^) to:
    echo      %%APPDATA%%\Adobe\CEP\extensions\com.egui-expressive.illustrator-exporter\
    echo   2. Restart Illustrator
)
echo.
echo Or use Anastasiy's Extension Manager ^(https://install.anastasiy.com^)
echo.
goto :done

:error
if exist "%STAGE%" rmdir /s /q "%STAGE%"
exit /b 1

:done
endlocal
