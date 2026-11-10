#Requires -Version 7

# Builds the Platformer and runs it on the primary display, in the
# foreground. Other arguments go to the game, like --no-activate.

Set-StrictMode -Version Latest
$ErrorActionPreference = 'Stop'

& (Join-Path $PSScriptRoot '..\Build\Build.ps1') --release Platformer
if ($LASTEXITCODE -ne 0) {
    exit $LASTEXITCODE
}
$exe = Join-Path $PSScriptRoot '..\..\target\Release\Platformer.exe'
# In this terminal, so that what the game writes, like why it couldn't start, shows here.
$game = Start-Process -FilePath $exe -ArgumentList (@('--display', 'Primary') + $args) -NoNewWindow -PassThru -Wait
exit $game.ExitCode
