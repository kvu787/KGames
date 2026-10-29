Set-StrictMode -Version Latest
$ErrorActionPreference = 'Stop'

# Publishes DisplayIdentification.exe to the Output folder. The exe needs the .NET 10 Desktop Runtime.
$projectPath = Join-Path $PSScriptRoot 'Source\DisplayIdentification.csproj'
$outputPath = Join-Path $PSScriptRoot 'Output'

dotnet publish $projectPath --configuration Release --runtime win-x64 --self-contained false --output $outputPath -p:PublishSingleFile=true
if ($LASTEXITCODE -ne 0) {
    throw "dotnet publish failed with exit code $LASTEXITCODE."
}

Write-Host "Published $(Join-Path $outputPath 'DisplayIdentification.exe')"
