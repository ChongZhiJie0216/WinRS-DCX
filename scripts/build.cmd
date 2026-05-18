@echo off
echo Cleaning previous builds...
cd duinodcx-rs
cargo clean
cd ..

echo Building Dioxus Web UI...
cd dcx-ui
dx build --release
cd ..

echo Refreshing Windows Icon Cache...
ie4uinit.exe -show

echo Building Rust Executable...
cd duinodcx-rs
cargo build --release
cd ..

echo Moving Executable...
move /y duinodcx-rs\target\release\WinRS_DCX_Management.exe WinRS-DCX-Management.exe

echo Build Complete!
