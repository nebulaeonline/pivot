@echo off
rem Run inside a Windows guest. The script asks before installing SSH and requesting UAC.
powershell.exe -NoProfile -ExecutionPolicy Bypass -File "%~dp0setup-windows.ps1"
exit /b %errorlevel%
