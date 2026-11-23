#Requires -Version 7

# Builds PixelFontTester and runs it on the primary display, in the foreground.
# Other arguments go to the tester, like --size 3.

Set-StrictMode -Version Latest
$ErrorActionPreference = 'Stop'

cargo run --release --manifest-path (Join-Path $PSScriptRoot 'Cargo.toml') -- --display Primary @args
exit $LASTEXITCODE
