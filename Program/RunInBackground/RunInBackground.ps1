#Requires -Version 7

# Runs RunInBackground.cs, which builds one of the GUI programs in Program,
# runs it on one display without taking focus, saves a screenshot of its
# window, and closes it. Pass --help for its options.

Set-StrictMode -Version Latest
$ErrorActionPreference = 'Stop'

dotnet run --file (Join-Path $PSScriptRoot 'RunInBackground.cs') -- @args
exit $LASTEXITCODE
