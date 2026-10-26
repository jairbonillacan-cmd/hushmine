@echo off
cd /d "%~dp0"
if exist "HushmineLauncher.exe" (
    start "" "%~dp0HushmineLauncher.exe"
    exit /b 0
)
if exist "target\release\hush-launcher.exe" (
    copy /Y "target\release\hush-launcher.exe" "HushmineLauncher.exe" >nul
    start "" "%~dp0HushmineLauncher.exe"
    exit /b 0
)
echo Compilando HUSHMINE Launcher...
cargo build --release -p hush-launcher -p hush-client
copy /Y "target\release\hush-launcher.exe" "HushmineLauncher.exe" >nul
start "" "%~dp0HushmineLauncher.exe"
