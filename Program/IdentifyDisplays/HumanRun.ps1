#Requires -Version 7

# Builds IdentifyDisplays and runs it, in the foreground: a box on each display
# with its identifier. Other arguments go to it, like --list.

Set-StrictMode -Version Latest
$ErrorActionPreference = 'Stop'

cargo run --release --manifest-path (Join-Path $PSScriptRoot 'Cargo.toml') -- @args
exit $LASTEXITCODE
