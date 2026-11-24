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

if (-not (Test-Path $TempPath) -or ((Get-Item $TempPath).Length -lt 10240)) {
    Write-Output "Error: Downloaded setup file is missing or corrupted."
    exit 1
}

Write-Output "🔍 Verifying package integrity..."
$ShaUrl = "$DownloadUrl.sha256"
$ShaFile = Join-Path $env:TEMP "Sysora_${Ver}_setup.exe.sha256"
try {
    Invoke-WebRequest -Uri $ShaUrl -OutFile $ShaFile -ErrorAction Stop -UseBasicParsing
    $ExpectedHash = (Get-Content $ShaFile).Trim().Split()[0].ToUpper()
    $ActualHash = (Get-FileHash -Path $TempPath -Algorithm SHA256).Hash.ToUpper()
    Remove-Item -Path $ShaFile -Force -ErrorAction SilentlyContinue
    if (-not $ExpectedHash -or ($ExpectedHash -ne $ActualHash)) {
        Write-Output "Error: SHA256 Checksum mismatch!"
        Remove-Item -Path $TempPath -Force -ErrorAction SilentlyContinue
        exit 1
    }
    Write-Output "✅ Checksum verified."
} catch {
    if ($env:SYSORA_INSECURE_NO_CHECKSUM -eq "1") {
        Write-Output "Warning: Release checksum not found; proceeding because SYSORA_INSECURE_NO_CHECKSUM=1."
    } else {
        Write-Output "Error: Required release checksum ($ShaUrl) could not be retrieved."
        Write-Output "Failing closed to prevent installation of unverified package."
        Remove-Item -Path $TempPath -Force -ErrorAction SilentlyContinue
        exit 1
    }
}

Write-Output "📦 Running installer..."
try {
    $proc = Start-Process -FilePath $TempPath -ArgumentList "/S" -Verb RunAs -Wait -PassThru
    if ($proc.ExitCode -ne 0) {
        Write-Output "Error: Installer exited with code $($proc.ExitCode)"
        Remove-Item -Path $TempPath -Force -ErrorAction SilentlyContinue
        exit $proc.ExitCode
    }
    Remove-Item -Path $TempPath -Force -ErrorAction SilentlyContinue
    Write-Output "✅ Sysora updated successfully to $Tag!"
} catch {
    Write-Output "Error: UAC prompt was cancelled or elevation failed."
    Remove-Item -Path $TempPath -Force -ErrorAction SilentlyContinue
    exit 1602
}
