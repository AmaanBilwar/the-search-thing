$ErrorActionPreference = "Stop"

function Write-Info {
  param([string]$Message)
  Write-Host ("[ffmpeg] " + $Message)
}

if ($env:OS -ne "Windows_NT") {
  Write-Error "This script is Windows-only."
  exit 1
}

$scriptDir = Split-Path -Parent $MyInvocation.MyCommand.Path
$clientRoot = Resolve-Path (Join-Path $scriptDir "..")
$resourcesRoot = Join-Path $clientRoot "resources"
$cacheDir = Join-Path $resourcesRoot "ffmpeg-cache\win-x64"

$ffmpegVersion = if ($env:FFMPEG_VERSION) { $env:FFMPEG_VERSION } else { "7.1" }
$zipName = "ffmpeg-$ffmpegVersion-full_build-shared"
$zipUrl = "https://github.com/GyanD/codexffmpeg/releases/download/$ffmpegVersion/$zipName.zip"
$zipPath = Join-Path $cacheDir "ffmpeg-$ffmpegVersion.zip"

New-Item -ItemType Directory -Force $cacheDir | Out-Null

if (-not (Test-Path $zipPath)) {
  Write-Info "Downloading $zipName..."
  Invoke-WebRequest -Uri $zipUrl -OutFile $zipPath
  Write-Info "Downloaded $zipName"
} else {
  Write-Info "Using cached ffmpeg archive: $zipPath"
}
