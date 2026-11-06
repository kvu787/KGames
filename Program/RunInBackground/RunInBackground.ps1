#Requires -Version 7

# Runs RunInBackground.cs, which builds one of the GUI programs in Program,
# runs it on one display without taking focus, saves a screenshot of its
# window, and closes it. Pass --help for its options. It runs lightly, with
# RunLightly, so that it, the build, and the program are light on someone
# who's using the computer, like playing a game.

Set-StrictMode -Version Latest
$ErrorActionPreference = 'Stop'

# The -- is quoted, as PowerShell would take a bare one as the end of the
# script's own parameters, and not pass it on to dotnet.
& (Join-Path $PSScriptRoot '..\RunLightly\RunLightly.ps1') dotnet run --file (Join-Path $PSScriptRoot 'RunInBackground.cs') '--' @args
exit $LASTEXITCODE
