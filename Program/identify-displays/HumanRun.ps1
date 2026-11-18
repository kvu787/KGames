#Requires -Version 7

# Builds IdentifyDisplays and runs it, which prints the displays, then waits for
# Enter, so that the console window stays open when HumanRun.cmd is
# double-clicked in File Explorer. It waits after a failed build too, so that
# the error stays on screen.

Set-StrictMode -Version Latest
$ErrorActionPreference = 'Stop'

cargo run --quiet --release --manifest-path (Join-Path $PSScriptRoot 'Cargo.toml')
$exitCode = $LASTEXITCODE
Write-Host ''
Read-Host 'Press Enter to close' | Out-Null
exit $exitCode
