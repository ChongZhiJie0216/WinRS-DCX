@echo off
pushd "%~dp0.."

echo [1/3] Checking tools...
where cargo >nul 2>nul
if errorlevel 1 (
    echo [ERROR] cargo not found. Install Rust from https://rustup.rs/
    goto :fail
)
if not exist "node_modules\dcx-ui\dist\index.html" (
    echo [ERROR] UI files missing: node_modules\dcx-ui\dist
    goto :fail
)

echo [2/3] Building Rust executable...
cargo build --release -p duinodcx-management
if errorlevel 1 (
    echo [ERROR] Rust build failed.
    goto :fail
)

echo [3/3] Copying executable...
copy /y "target\release\WinRS-DCX-Management.exe" "WinRS-DCX-Management.exe" >nul
if errorlevel 1 (
    echo [ERROR] Copy failed. Close WinRS-DCX-Management.exe if it is running.
    goto :fail
)
ie4uinit.exe -show >nul 2>nul

echo Build complete: %CD%\WinRS-DCX-Management.exe
popd
if /i not "%~1"=="--no-pause" pause
exit /b 0

:fail
popd
if /i not "%~1"=="--no-pause" pause
exit /b 1
