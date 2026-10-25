@echo off
rem Read-only inspection. ExecutionPolicy applies only to this process.
powershell.exe -NoProfile -ExecutionPolicy Bypass -File "%~dp0inspect-windows.ps1"
exit /b %errorlevel%
