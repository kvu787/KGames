#Requires -Version 7

# Builds UiShowcase and runs it on the primary display, in the foreground.
# Other arguments go to the showcase, like --no-activate.

Set-StrictMode -Version Latest
$ErrorActionPreference = 'Stop'

& (Join-Path $PSScriptRoot '..\Build\Build.ps1') --release UiShowcase
if ($LASTEXITCODE -ne 0) {
    exit $LASTEXITCODE
}
$exe = Join-Path $PSScriptRoot '..\..\target\Release\UiShowcase.exe'
# In this terminal, so that what the showcase writes, like why it couldn't start, shows here.
$showcase = Start-Process -FilePath $exe -ArgumentList (@('--display', 'Primary') + $args) -NoNewWindow -PassThru -Wait
exit $showcase.ExitCode
