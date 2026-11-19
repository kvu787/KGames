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
# The command gets its arguments as it would if PowerShell ran it, but for a
# --: PowerShell takes one that isn't quoted for itself, before this script
# sees it, so quote one that's for the command:
#
#     Program\RunLightly\RunLightly.ps1 cargo test -p present-timing-tests '--' --ignored
#
# Run it from PowerShell, or from another shell with pwsh -Command: pwsh's
# -File reads an argument like -name:value as two, -name and value, before
# this script sees it.
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

# pwsh's command line would read arguments like -name:value and -- as its
# own, so the command and its arguments, as PowerShell gave them to this
# script, with the marks it puts on the ones written as parameters, go to
# RunLightlyInJob.ps1 as one argument: serialized, compressed, in Base64.
$serialized = [System.Text.Encoding]::UTF8.GetBytes([System.Management.Automation.PSSerializer]::Serialize($args, 4))
$compressed = [System.IO.MemoryStream]::new()
$gzip = [System.IO.Compression.GZipStream]::new($compressed, [System.IO.Compression.CompressionLevel]::Optimal)
$gzip.Write($serialized, 0, $serialized.Length)
$gzip.Dispose()
& pwsh -NoProfile -File (Join-Path $PSScriptRoot 'RunLightlyInJob.ps1') ([Convert]::ToBase64String($compressed.ToArray()))
exit $LASTEXITCODE
