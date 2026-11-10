#Requires -Version 7

# Builds the C++ projects with Visual Studio's MSBuild, which isn't on the
# PATH outside a developer prompt, so this finds it. Without a project, it
# builds every project in KGames.slnx.
#
#     Program\Build\Build.ps1                          Everything, in Debug
#     Program\Build\Build.ps1 --release                Everything, in Release
#     Program\Build\Build.ps1 --release Platformer     One project, and what it uses
#     Program\Build\Build.ps1 MathTests JsonTests
#     Program\Build\Build.ps1 --release --hardened Benchmarks
#
# --hardened builds beside the usual build, into target\ReleaseHardened or
# target\DebugHardened, with the standard library checking every index and
# stopping the program at a wrong one, as KGames.props says.
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
$hardened = $false
$names = @()
foreach ($argument in $args) {
    switch ($argument) {
        '--release' { $configuration = 'Release' }
        '--debug' { $configuration = 'Debug' }
        '--hardened' { $hardened = $true }
        default {
            if ($argument.StartsWith('-')) {
                Write-Error "Unknown argument $argument. Build.ps1 takes --release, --debug, --hardened, and the names of projects."
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

# Restores the NuGet packages that Library\GameInput\NuGetPackages.proj
# declares, with dotnet, as packages.lock.json pins them: Microsoft.GameInput,
# whose GameInput.h the C++ includes and whose GameInput.lib it links (see
# Library\GameInput\GameInput.props). Only when what declares them is newer
# than the last restore, so that a build that has them doesn't start dotnet.
$gameInput = Join-Path $repository 'Library\GameInput'
$restored = Join-Path $repository 'target\DotNet\obj\NuGetPackages\NuGetPackages.proj.nuget.g.props'
$declared = @(
    (Join-Path $gameInput 'NuGetPackages.proj'),
    (Join-Path $gameInput 'packages.lock.json'),
    (Join-Path $gameInput 'Directory.Build.props'),
    (Join-Path $repository 'NuGet.config'),
    (Join-Path $repository 'global.json')
)
$newest = ($declared | ForEach-Object { (Get-Item $_).LastWriteTimeUtc } | Measure-Object -Maximum).Maximum
if (-not (Test-Path $restored) -or (Get-Item $restored).LastWriteTimeUtc -lt $newest) {
    if (-not (Get-Command dotnet -ErrorAction SilentlyContinue)) {
        Write-Error 'Building KGames needs the .NET SDK, 10 or later, whose dotnet restores the Microsoft.GameInput NuGet package that Library/GameInput/NuGetPackages.proj declares. Visual Studio installs it with its .NET desktop development workload, or get it from https://dotnet.microsoft.com/download'
    }
    # A build that restores in passing doesn't send dotnet's usage data.
    $env:DOTNET_CLI_TELEMETRY_OPTOUT = '1'
    $env:DOTNET_NOLOGO = '1'
    # Locked mode fails, instead of changing packages.lock.json, when the
    # project asks for other packages than the lock file pins, and a package
    # that it downloads must have the hash that the lock file gives. Without
    # build servers, nothing that it starts outlives it.
    & dotnet restore (Join-Path $gameInput 'NuGetPackages.proj') --locked-mode --disable-build-servers --nologo
    if ($LASTEXITCODE -ne 0) {
        Write-Error "dotnet restore of Library\GameInput\NuGetPackages.proj failed with exit code $LASTEXITCODE."
    }
    # A restore that changed nothing leaves the file as old as it was.
    (Get-Item $restored).LastWriteTimeUtc = [DateTime]::UtcNow
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
    & $msbuild $project -nologo -maxCpuCount -nodeReuse:false -verbosity:minimal "-property:Configuration=$configuration" -property:Platform=x64 "-property:KGamesHardened=$($hardened.ToString().ToLowerInvariant())"
    if ($LASTEXITCODE -ne 0) {
        exit $LASTEXITCODE
    }
}
exit 0
