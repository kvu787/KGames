#Requires -Version 7

# Builds GameInputDemo and runs it on the primary display, in the foreground.
# Other arguments go to the demo, like --no-activate.

Set-StrictMode -Version Latest
$ErrorActionPreference = 'Stop'

& (Join-Path $PSScriptRoot '..\Build\Build.ps1') --release GameInputDemo
if ($LASTEXITCODE -ne 0) {
    exit $LASTEXITCODE
}
$exe = Join-Path $PSScriptRoot '..\..\target\Release\GameInputDemo.exe'
# In this terminal, so that what the demo writes, like why it couldn't start, shows here.
$demo = Start-Process -FilePath $exe -ArgumentList (@('--display', 'Primary') + $args) -NoNewWindow -PassThru -Wait
exit $demo.ExitCode
