@echo off
rem Runs HumanRun.ps1, which builds IdentifyDisplays, runs it, and waits for Enter.
pwsh -NoProfile -ExecutionPolicy Bypass -File "%~dp0HumanRun.ps1"
exit /b %ERRORLEVEL%
