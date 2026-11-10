#Requires -Version 7

# Builds MinimalSceneTest and runs it on the primary display, in the foreground.
# Other arguments go to the test, like --no-activate.

Set-StrictMode -Version Latest
$ErrorActionPreference = 'Stop'

& (Join-Path $PSScriptRoot '..\Build\Build.ps1') --release MinimalSceneTest
if ($LASTEXITCODE -ne 0) {
    exit $LASTEXITCODE
}
$exe = Join-Path $PSScriptRoot '..\..\target\Release\MinimalSceneTest.exe'
# In this terminal, so that what the test writes, like why it couldn't start, shows here.
$test = Start-Process -FilePath $exe -ArgumentList (@('--display', 'Primary') + $args) -NoNewWindow -PassThru -Wait
exit $test.ExitCode
