@echo off
setlocal enabledelayedexpansion

echo ============================================
echo  egui_expressive Illustrator Plugin Installer
echo ============================================
echo.

REM Set plugin name
set PLUGIN_NAME=egui_expressive_export

REM Find AppData path
set APPDATA_PATH=%APPDATA%

REM Try multiple Illustrator versions (28=2024, 27=2023, 26=2022)
set INSTALLED=0
for %%V in (28 27 26 25) do (
    set PLUGIN_DIR=%APPDATA_PATH%\Adobe\UXP\PluginsStorage\ILST\%%V\develop\%PLUGIN_NAME%
    if exist "%APPDATA_PATH%\Adobe\UXP\PluginsStorage\ILST\%%V" (
        echo Found Illustrator version %%V
        mkdir "!PLUGIN_DIR!" 2>nul
        copy /Y "manifest.json" "!PLUGIN_DIR!\manifest.json" >nul
        copy /Y "plugin.js" "!PLUGIN_DIR!\plugin.js" >nul
        copy /Y "index.html" "!PLUGIN_DIR!\index.html" >nul
        echo Plugin installed to: !PLUGIN_DIR!
        set INSTALLED=1
    )
)

if !INSTALLED!==0 (
    echo.
    echo WARNING: Could not find Illustrator UXP plugin directory.
    echo.
    echo Please manually copy these files to your Illustrator plugins folder:
    echo   manifest.json
    echo   plugin.js
    echo   index.html
    echo.
    echo Typical location:
    echo   %%APPDATA%%\Adobe\UXP\PluginsStorage\ILST\28\develop\egui_expressive_export\
    echo.
    echo After copying, restart Illustrator and enable the plugin in:
    echo   Plugins ^> Plugin Manager
)

echo.
echo After installation:
echo   1. Restart Adobe Illustrator
echo   2. Go to Plugins ^> Plugin Manager
echo   3. Enable "egui_expressive Export"
echo   4. Open the plugin from Plugins ^> egui_expressive Export
echo.
pause
