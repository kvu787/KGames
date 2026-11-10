#Requires -Version 7

# Builds Veehiicuul and runs it on the primary display, in the foreground.
# Other arguments go to the game, like --no-activate.

Set-StrictMode -Version Latest
$ErrorActionPreference = 'Stop'

$repository = (Resolve-Path (Join-Path $PSScriptRoot '..\..')).Path
& (Join-Path $repository 'Program\Build\Build.ps1') --release Veehiicuul
if ($LASTEXITCODE -ne 0) {
    exit $LASTEXITCODE
}
# Through the pipeline, so that this waits for the game, which has no console
# of its own, and shows what it writes.
& (Join-Path $repository 'target\Release\Veehiicuul.exe') --display Primary @args | Out-Host
exit $LASTEXITCODE
