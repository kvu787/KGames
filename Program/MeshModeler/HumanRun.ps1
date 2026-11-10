#Requires -Version 7

# Builds MeshModeler and runs it on the primary display, in the foreground.
# Other arguments go to the modeler, like --no-activate.

Set-StrictMode -Version Latest
$ErrorActionPreference = 'Stop'

& (Join-Path $PSScriptRoot '..\Build\Build.ps1') --release MeshModeler
if ($LASTEXITCODE -ne 0) {
    exit $LASTEXITCODE
}
$exe = Join-Path $PSScriptRoot '..\..\target\Release\MeshModeler.exe'
# In this terminal, so that what the modeler writes, like why it couldn't start, shows here.
$modeler = Start-Process -FilePath $exe -ArgumentList (@('--display', 'Primary') + $args) -NoNewWindow -PassThru -Wait
exit $modeler.ExitCode
