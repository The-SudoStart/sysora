param(
    [string]$Version = "latest"
)

[Console]::OutputEncoding = [System.Text.Encoding]::UTF8
$ProgressPreference = 'SilentlyContinue'
[Net.ServicePointManager]::SecurityProtocol = [Net.SecurityProtocolType]::Tls12

$Repo = "The-SudoStart/sysora"
Write-Output "📥 Checking for latest Sysora release for Windows..."

if ($Version -eq "latest") {
    $Response = Invoke-WebRequest -Uri "https://github.com/$Repo/releases/latest" -MaximumRedirection 0 -ErrorAction Ignore -UseBasicParsing
    if ($Response.StatusCode -in 301, 302) {
        $Tag = ($Response.Headers.Location -split '/')[-1]
    } else {
        $ApiUrl = "https://api.github.com/repos/$Repo/releases/latest"
        $Tag = (Invoke-RestMethod -Uri $ApiUrl -UseBasicParsing).tag_name
    }
} else {
    $Tag = $Version
}

if (-not $Tag) {
    Write-Output "Error: Could not determine version tag."
    exit 1
}

$Ver = $Tag.TrimStart('v')
$Tag = "v$Ver"

$DownloadUrl = "https://github.com/$Repo/releases/download/$Tag/Sysora_${Ver}_x64-setup.exe"
$TempPath = Join-Path $env:TEMP "Sysora_${Ver}_setup.exe"

Write-Output "🔍 Checking release asset: $DownloadUrl"
try {
    Invoke-WebRequest -Uri $DownloadUrl -Method Head -ErrorAction Stop -UseBasicParsing > $null
} catch {
    Write-Output "Error: Could not find Windows setup exe ($DownloadUrl) in release"
    exit 1
}

Write-Output "⬇️ Downloading Sysora from: $DownloadUrl"
Invoke-WebRequest -Uri $DownloadUrl -OutFile $TempPath -UseBasicParsing

Write-Output "📦 Running installer..."
try {
    Start-Process -FilePath $TempPath -ArgumentList "/S" -Verb RunAs -Wait
    Write-Output "✅ Sysora updated successfully to $Tag!"
} catch {
    Write-Output "Error: UAC prompt was cancelled or elevation failed."
    exit 1602
}
