@echo off
setlocal enabledelayedexpansion

echo ========================================================
echo Historical COM Ports on this Computer
echo ========================================================
echo.

:: We use PowerShell to query the Windows Registry for all devices
:: that have a "FriendlyName" containing "(COM"
powershell -NoProfile -Command "$ErrorActionPreference = 'SilentlyContinue'; Get-ChildItem -Path HKLM:\SYSTEM\CurrentControlSet\Enum -Recurse | Get-ItemProperty -Name FriendlyName | Where-Object { $_.FriendlyName -match '\(COM\d+\)' } | Select-Object -Unique FriendlyName | ForEach-Object { $_.FriendlyName }"

echo.
pause
