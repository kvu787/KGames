#Requires -Version 7

# Runs a command lightly, so that it doesn't disturb someone who's using the
# computer, like playing a game. RunLightlyInJob.ps1 says how: in a job
# object, on the efficiency cores, at idle priority, with a share of the CPU,
# at low I/O and memory priority, in efficiency mode, and with its GPU work
# after other processes'. It runs that in a PowerShell process of its own, so
# that the shell this is run from, like someone's terminal, stays as it is.
#
#     Program\RunLightly\RunLightly.ps1 cargo test --workspace
#
# Run from a command that's already in a RunLightly job, which
# KGAMES_LIGHT_JOB names, like RunInBackground run through RunLightly, it just
# runs the command, which the job holds already. It exits with the command's
# exit code.

Set-StrictMode -Version Latest
$ErrorActionPreference = 'Stop'

if ($args.Count -eq 0) {
    Write-Error 'Name a command to run, like: Program\RunLightly\RunLightly.ps1 cargo test --workspace'
}

if ($env:KGAMES_LIGHT_JOB) {
    $command = $args[0]
    $arguments = @($args | Select-Object -Skip 1)
    & $command @arguments
    exit $LASTEXITCODE
}

& pwsh -NoProfile -File (Join-Path $PSScriptRoot 'RunLightlyInJob.ps1') @args
exit $LASTEXITCODE
