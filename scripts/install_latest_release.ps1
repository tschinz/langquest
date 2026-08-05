#requires -Version 5.1
$ErrorActionPreference = "Stop"

$Repo = if ($env:REPO) { $env:REPO } else { "tschinz/langquest" }
$App = if ($env:APP) { $env:APP } else { "lq" }
$InstallDir = if ($env:INSTALL_DIR) { $env:INSTALL_DIR } else { Join-Path $env:LOCALAPPDATA "Programs\lq\bin" }

$arch = $env:PROCESSOR_ARCHITECTURE
if ($arch -ne "AMD64") {
  throw "Unsupported Windows architecture: $arch. Current release assets include x86_64-pc-windows-msvc."
}

$target = "x86_64-pc-windows-msvc"

$apiUrl = "https://api.github.com/repos/$Repo/releases/latest"
$release = Invoke-RestMethod -Uri $apiUrl -Headers @{ "User-Agent" = "lq-installer" }
$tag = $release.tag_name
if (-not $tag) { throw "Could not determine latest release tag." }

$asset = "$App-$tag-$target.zip"
$url = "https://github.com/$Repo/releases/download/$tag/$asset"

$tmp = Join-Path $env:TEMP ("lq-install-" + [guid]::NewGuid().ToString("N"))
New-Item -ItemType Directory -Path $tmp | Out-Null

$zipPath = Join-Path $tmp $asset
Write-Host "Installing $App $tag for $target"
Write-Host "Downloading: $url"
Invoke-WebRequest -Uri $url -OutFile $zipPath

Expand-Archive -Path $zipPath -DestinationPath $tmp -Force

$exe = Join-Path $tmp "$App.exe"
if (-not (Test-Path $exe)) {
  throw "Archive did not contain expected binary: $App.exe"
}

New-Item -ItemType Directory -Path $InstallDir -Force | Out-Null
Copy-Item -Path $exe -Destination (Join-Path $InstallDir "$App.exe") -Force

Write-Host "Installed to: $(Join-Path $InstallDir "$App.exe")"

$userPath = [Environment]::GetEnvironmentVariable("Path", "User")
if (($userPath -split ";") -notcontains $InstallDir) {
  $newPath = if ([string]::IsNullOrWhiteSpace($userPath)) { $InstallDir } else { "$userPath;$InstallDir" }
  [Environment]::SetEnvironmentVariable("Path", $newPath, "User")
  Write-Host "Added $InstallDir to user PATH. Open a new terminal to use $App."
}

& (Join-Path $InstallDir "$App.exe") --version