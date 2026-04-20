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

set "EXTENSION_ID=com.egui-expressive.illustrator-exporter"
set "VERSION=1.0.0"
set "ZXP_NAME=egui_expressive_export-%VERSION%.zxp"

REM Certificate defaults
set "CERT_COUNTRY=US"
set "CERT_STATE=NA"
set "CERT_ORG=egui_expressive"
set "CERT_NAME=egui_expressive Exporter"
if defined ZXP_SIGN_PASSWORD (
    set "CERT_PASSWORD=%ZXP_SIGN_PASSWORD%"
) else (
    set "CERT_PASSWORD=selfsign_temp"
    echo [WARN] No ZXP_SIGN_PASSWORD env var set - using ephemeral password.
)
set "CERT_FILE=%OUTPUT_DIR%\cert.p12"

set "TSA_URL=http://timestamp.digicert.com"

echo ============================================================
echo   egui_expressive Exporter - .zxp Package Builder
echo ============================================================
echo.

REM Create output directory
if not exist "%OUTPUT_DIR%" mkdir "%OUTPUT_DIR%"

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
copy "%PLUGIN_DIR%\manifest.json" "%STAGE%\manifest.json" >nul

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
    echo [ERROR] Failed to sign package
    goto :error
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

REM Cleanup
if exist "%STAGE%" rmdir /s /q "%STAGE%"

echo.
echo [INFO] Done! Package: %OUTPUT_DIR%\%ZXP_NAME%
echo.
echo Install with:
echo   UPIA:  "%%ProgramFiles%%\Common Files\Adobe\Adobe Desktop Common\RemoteComponents\UPI\UnifiedPluginInstallerAgent\UnifiedPluginInstallerAgent.exe" /install "%OUTPUT_DIR%\%ZXP_NAME%"
echo   Or:    Anastasiy's Extension Manager (https://install.anastasiy.com)
echo.
goto :done

:error
if exist "%STAGE%" rmdir /s /q "%STAGE%"
exit /b 1

:done
endlocal
