#Requires -Version 7

# Builds the tests in Test and runs them: every project there, or the ones
# named. Each is a console program that runs its tests one after another
# (see Library\Testing\Testing.h).
#
#     Program\RunTests\RunTests.ps1                        Every test, in Debug
#     Program\RunTests\RunTests.ps1 MathTests JsonTests    Two projects' tests
#     Program\RunTests\RunTests.ps1 JsonTests --filter writes
#     Program\RunTests\RunTests.ps1 --release
#     Program\RunTests\RunTests.ps1 --no-build             Runs what's built already
#
# Tests build in Debug by default, with the C runtime's and the standard
# library's checks on, like the bounds of every index. --filter runs only the
# tests with the text in their names.
#
# Coding agents run it through RunLightly, as AGENTS.md says:
#
#     Program\RunLightly\RunLightly.ps1 Program\RunTests\RunTests.ps1
#
# It exits with 1 if a build or a test failed.

Set-StrictMode -Version Latest
$ErrorActionPreference = 'Stop'

$repository = (Resolve-Path (Join-Path $PSScriptRoot '..\..')).Path
$configuration = 'Debug'
$build = $true
$filters = @()
$names = @()
for ($index = 0; $index -lt $args.Count; $index++) {
    $argument = $args[$index]
    switch ($argument) {
        '--release' { $configuration = 'Release' }
        '--debug' { $configuration = 'Debug' }
        '--no-build' { $build = $false }
        '--filter' {
            $index++
            if ($index -ge $args.Count) {
                Write-Error '--filter needs text after it.'
            }
            $filters += $args[$index]
        }
        default {
            if ($argument.StartsWith('-')) {
                Write-Error "Unknown argument $argument. RunTests.ps1 takes --release, --debug, --no-build, --filter <text>, and the names of test projects."
            }
            $names += $argument
        }
    }
}

$all = @(Get-ChildItem -Path (Join-Path $repository 'Test') -Recurse -Filter '*Tests.vcxproj' | ForEach-Object { $_.BaseName } | Sort-Object)
foreach ($name in $names) {
    if ($all -notcontains $name) {
        Write-Error "Test has no project named $name.vcxproj. It has: $($all -join ', ')"
    }
}
# An array, even of one: an if gives one name as a string.
$selected = @(if ($names.Count -gt 0) { $names } else { $all })

if ($build) {
    $buildArguments = @("--$($configuration.ToLowerInvariant())")
    # Everything at once, through the solution, when it's every test.
    if ($names.Count -gt 0) {
        $buildArguments += $names
    }
    & (Join-Path $PSScriptRoot '..\Build\Build.ps1') @buildArguments
    if ($LASTEXITCODE -ne 0) {
        Write-Host "The build failed, so no tests ran."
        exit 1
    }
}

$failed = @()
foreach ($name in $selected) {
    $exe = Join-Path $repository "target\$configuration\$name.exe"
    if (-not (Test-Path $exe)) {
        Write-Host "`n$name isn't built: $exe is missing."
        $failed += $name
        continue
    }
    Write-Host "`n     Running $name"
    & $exe @filters
    if ($LASTEXITCODE -ne 0) {
        $failed += "$name (exit code $LASTEXITCODE)"
    }
}

Write-Host ''
if ($failed.Count -gt 0) {
    Write-Host "FAILED: $($failed -join ', ')"
    exit 1
}
Write-Host "Every test passed, in $($selected.Count) projects."
exit 0
