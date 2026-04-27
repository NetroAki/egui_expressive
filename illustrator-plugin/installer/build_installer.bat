@echo off
echo Building egui_expressive plugin installer...

pushd "%~dp0..\.."
cargo build --release --bin ai-parser
if errorlevel 1 (
    popd
    echo Failed to build bundled ai-parser.
    pause
    exit /b 1
)
popd

if not exist "%~dp0..\bin\win32" mkdir "%~dp0..\bin\win32"
copy "%~dp0..\..\target\release\ai-parser.exe" "%~dp0..\bin\win32\ai-parser.exe" >nul
if errorlevel 1 (
    echo Failed to stage bundled ai-parser.exe.
    pause
    exit /b 1
)

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
