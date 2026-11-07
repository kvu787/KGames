#Requires -Version 7

# Builds WinUi3Sample and runs it on the primary display, in the foreground.
# Other arguments go to the sample, like --theme Dark.

Set-StrictMode -Version Latest
$ErrorActionPreference = 'Stop'

dotnet build (Join-Path $PSScriptRoot 'WinUi3Sample.csproj') --configuration Release --nologo --verbosity quiet
if ($LASTEXITCODE -ne 0) {
    exit $LASTEXITCODE
}
$exe = Join-Path $PSScriptRoot '..\..\target\DotNet\bin\WinUi3Sample\release_win-x64\WinUi3Sample.exe'
$sample = Start-Process -FilePath $exe -ArgumentList (@('--display', 'Primary') + $args) -PassThru -Wait
exit $sample.ExitCode
