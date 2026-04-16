@echo off
echo Building egui_expressive plugin installer...

REM Try common NSIS locations
set NSIS_PATH=
if exist "C:\Program Files (x86)\NSIS\makensis.exe" set NSIS_PATH=C:\Program Files (x86)\NSIS\makensis.exe
if exist "C:\Program Files\NSIS\makensis.exe" set NSIS_PATH=C:\Program Files\NSIS\makensis.exe

if "%NSIS_PATH%"=="" (
    echo NSIS not found. Please install NSIS from https://nsis.sourceforge.io/
    echo Then run: makensis egui_expressive_plugin.nsi
    pause
    exit /b 1
)

"%NSIS_PATH%" egui_expressive_plugin.nsi
echo.
echo Installer built: egui_expressive_plugin_installer.exe
pause
