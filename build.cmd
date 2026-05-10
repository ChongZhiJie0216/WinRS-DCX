@echo off
setlocal enabledelayedexpansion

echo ################################################
echo # DuinoDCX Rust - Build Script                 #
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
if exist duinodcx-rs.exe (
    echo [INFO] Removing old duinodcx-rs.exe...
    del /f /q duinodcx-rs.exe
    if %errorlevel% neq 0 (
        echo [WARN] Could not remove duinodcx-rs.exe. It might be running.
        echo Please close the application and try again.
        pause
        exit /b 1
    )
)

cd duinodcx-rs

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
copy /y target\release\duinodcx-rs.exe ..\duinodcx-rs.exe >nul

if %errorlevel% neq 0 (
    echo [ERROR] Failed to copy the executable.
    cd ..
    pause
    exit /b 1
)

cd ..

echo.
echo ################################################
echo # SUCCESS: duinodcx-rs.exe is ready!           #
echo ################################################
echo.
pause
endlocal
