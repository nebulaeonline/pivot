@echo off
python "%~dp0attend_validation.py" %*
exit /b %errorlevel%
