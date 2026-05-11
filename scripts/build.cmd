@echo off
setlocal enabledelayedexpansion

:: Change to the project root directory
cd %~dp0..

echo ################################################
echo # DuinoDCX Rust - Management Build Script      #
echo ################################################
echo.

:: Check for Rust installation
where cargo >nul 2>nul
if %errorlevel% neq 0 (
    echo [ERROR] Rust cargo is not installed or not in PATH.
    echo Please install Rust from https://rustup.rs/
    pause
    exit /b 1
)

:: Try to remove the old executable in root to avoid confusion
if exist WinRS-DCX-Management.exe (
    echo [INFO] Removing old WinRS-DCX-Management.exe...
    del /f /q WinRS-DCX-Management.exe
    if %errorlevel% neq 0 (
        echo [WARN] Could not remove WinRS-DCX-Management.exe. It might be running.
        echo Please close the application and try again.
        pause
        exit /b 1
    )
)

:: Try to remove the WebView2 cache to force a clean state
if exist WinRS-DCX-Management.exe.WebView2 (
    echo [INFO] Clearing WebView2 cache...
    rmdir /s /q WinRS-DCX-Management.exe.WebView2
)

cd duinodcx-rs

echo [INFO] Cleaning project to force rebuild...
cargo clean

echo [INFO] Starting Rust release build...
cargo build --release

if %errorlevel% neq 0 (
    echo.
    echo [ERROR] Rust build failed.
    cd ..
    pause
    exit /b 1
)

echo.
echo [INFO] Copying executable to root...
copy /y target\release\WinRS-DCX-Management.exe ..\WinRS-DCX-Management.exe >nul

:: Copy WebView2Loader.dll if it's found in the target directory (sometimes cargo puts it there)
if exist target\release\WebView2Loader.dll (
    echo [INFO] Copying WebView2Loader.dll to root...
    copy /y target\release\WebView2Loader.dll ..\WebView2Loader.dll >nul
)

if %errorlevel% neq 0 (
    echo [ERROR] Failed to copy the executable.
    cd ..
    pause
    exit /b 1
)

cd ..

:: Refresh Windows icon cache so the new icon shows up immediately
echo [INFO] Refreshing Windows icon cache...
ie4uinit.exe -show

echo.
echo ################################################
echo # SUCCESS: WinRS-DCX-Management.exe is ready!   #
echo ################################################
echo.
pause
endlocal
