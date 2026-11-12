@echo off
rem Runs HumanRun.ps1, which builds IdentifyDisplays and runs it.
pwsh -NoProfile -ExecutionPolicy Bypass -File "%~dp0HumanRun.ps1" %*
exit /b %ERRORLEVEL%
