@echo off
setlocal enabledelayedexpansion

:: Change to the project root directory
cd %~dp0..

echo ################################################
echo # DuinoDCX Rust - Windows Management Service    #
echo ################################################
echo.

:: Check for Rust installation
where cargo >nul 2>nul
if %errorlevel% neq 0 (
    echo [ERROR] Rust ^(cargo^) is not installed or not in PATH.
    echo Please install Rust from https://rustup.rs/
    pause
    exit /b 1
)

cd duinodcx-rs

echo [INFO] Configuration:
echo   - Port: Defined in src/main.rs (default COM1)
echo   - UI: http://127.0.0.1:3000
echo.
echo [INFO] Press any key to start the service...

:: Run the application
set RUST_LOG=info
cargo run --release

if %errorlevel% neq 0 (
    echo.
    echo [ERROR] Application crashed or failed to start.
    echo Check if the COM port is correct and not in use by another program.
)

echo.
echo [INFO] Service stopped. Press any key to exit.
pause >nul
endlocal
