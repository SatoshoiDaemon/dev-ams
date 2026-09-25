param(
    [string]$Target = "",
    [switch]$SkipBuild
)

$ErrorActionPreference = "Stop"
$repository = Split-Path -Parent $PSScriptRoot
$artifactName = if ($Target) { "ams-$Target" } else { "ams-windows-x86_64" }
$destination = [System.IO.Path]::GetFullPath((Join-Path $repository "dist/$artifactName"))
$distributionRoot = [System.IO.Path]::GetFullPath((Join-Path $repository "dist"))
if (-not $destination.StartsWith($distributionRoot + [System.IO.Path]::DirectorySeparatorChar)) {
    throw "Unsafe package destination: $destination"
}

if (-not $SkipBuild) {
    $arguments = @("build", "--release", "--locked")
    if ($Target) { $arguments += @("--target", $Target) }
    & cargo @arguments
    if ($LASTEXITCODE -ne 0) { throw "cargo build failed" }
}

if (Test-Path -LiteralPath $destination) {
    Remove-Item -LiteralPath $destination -Recurse -Force
}
New-Item -ItemType Directory -Path $destination | Out-Null
foreach ($directory in @("saves", "mods", "logs")) {
    New-Item -ItemType Directory -Path (Join-Path $destination $directory) | Out-Null
}
Copy-Item -LiteralPath (Join-Path $repository "config.toml") -Destination $destination
Copy-Item -LiteralPath (Join-Path $repository "LICENSE") -Destination $destination
Copy-Item -LiteralPath (Join-Path $repository "data") -Destination $destination -Recurse
Copy-Item -LiteralPath (Join-Path $repository "gamemodes") -Destination $destination -Recurse
$binaryRoot = if ($Target) { Join-Path $repository "target/$Target/release" } else { Join-Path $repository "target/release" }
$binary = Join-Path $binaryRoot "ams.exe"
if (-not (Test-Path -LiteralPath $binary)) { $binary = Join-Path $binaryRoot "ams" }
Copy-Item -LiteralPath $binary -Destination $destination
Write-Output $destination
