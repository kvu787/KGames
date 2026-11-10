#Requires -Version 7

# Builds the C++ projects with Visual Studio's MSBuild, which isn't on the
# PATH outside a developer prompt, so this finds it. Without a project, it
# builds every project in KGames.slnx.
#
#     Program\Build\Build.ps1                          Everything, in Debug
#     Program\Build\Build.ps1 --release                Everything, in Release
#     Program\Build\Build.ps1 --release Platformer     One project, and what it uses
#     Program\Build\Build.ps1 MathTests JsonTests
#
# A project is named as its .vcxproj is, like Platformer, PlatformerLibrary,
# MathTests or Json. Programs and libraries go to target\Debug or
# target\Release, at the root of the repository.
#
# Coding agents run it through RunLightly, as AGENTS.md says:
#
#     Program\RunLightly\RunLightly.ps1 Program\Build\Build.ps1 --release
#
# It exits with MSBuild's exit code.

Set-StrictMode -Version Latest
$ErrorActionPreference = 'Stop'

$repository = (Resolve-Path (Join-Path $PSScriptRoot '..\..')).Path
$configuration = 'Debug'
$names = @()
foreach ($argument in $args) {
    switch ($argument) {
        '--release' { $configuration = 'Release' }
        '--debug' { $configuration = 'Debug' }
        default {
            if ($argument.StartsWith('-')) {
                Write-Error "Unknown argument $argument. Build.ps1 takes --release, --debug, and the names of projects."
            }
            $names += $argument
        }
    }
}

# Visual Studio's own MSBuild, which has the C++ targets. dotnet's doesn't.
$vswhere = Join-Path ${env:ProgramFiles(x86)} 'Microsoft Visual Studio\Installer\vswhere.exe'
if (-not (Test-Path $vswhere)) {
    Write-Error 'Building KGames needs Visual Studio, with its Desktop development with C++ workload. vswhere.exe, which its installer puts in Program Files (x86)\Microsoft Visual Studio\Installer, is missing.'
}
$msbuild = & $vswhere -latest -prerelease -products * -requires Microsoft.VisualStudio.Component.VC.Tools.x86.x64 -find 'MSBuild\**\Bin\amd64\MSBuild.exe' | Select-Object -First 1
if (-not $msbuild) {
    Write-Error 'Building KGames needs Visual Studio with its Desktop development with C++ workload, which has MSBuild and the C++ compiler. vswhere found none.'
}

$projects = @()
if ($names.Count -eq 0) {
    $projects += Join-Path $repository 'KGames.slnx'
}
foreach ($name in $names) {
    $found = @(Get-ChildItem -Path (Join-Path $repository 'Library'), (Join-Path $repository 'Program'), (Join-Path $repository 'Test') -Recurse -Filter "$name.vcxproj")
    if ($found.Count -ne 1) {
        Write-Error "Found $($found.Count) projects named $name.vcxproj in Library, Program and Test."
    }
    $projects += $found[0].FullName
}

foreach ($project in $projects) {
    # Without node reuse, MSBuild's worker processes end with the build, so
    # none outlives a RunLightly job, and none from another build, outside
    # the job, does this one's work.
    & $msbuild $project -nologo -maxCpuCount -nodeReuse:false -verbosity:minimal "-property:Configuration=$configuration" -property:Platform=x64
    if ($LASTEXITCODE -ne 0) {
        exit $LASTEXITCODE
    }
}
exit 0
