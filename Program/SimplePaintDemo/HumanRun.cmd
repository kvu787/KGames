@echo off
rem Runs HumanRun.ps1, which builds SimplePaintDemo and runs it on the primary display.
pwsh -NoProfile -ExecutionPolicy Bypass -File "%~dp0HumanRun.ps1" %*
exit /b %ERRORLEVEL%
