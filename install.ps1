# Create envfetch directory in AppData/Roaming
$envfetchPath = "$env:APPDATA\envfetch"
New-Item -ItemType Directory -Force -Path $envfetchPath -ErrorAction SilentlyContinue | Out-Null
Write-Host "Installing envfetch to $envfetchPath"

Push-Location $envfetchPath

try {
    $outFile = "envfetch.exe"
    if ([System.IO.File]::Exists($outFile)) {
        Write-Host "envfetch already exists. Updating..."
    }
    Invoke-WebRequest -Uri "https://github.com/ankddev/envfetch/releases/latest/download/envfetch-windows-amd64.exe" -OutFile $outFile

    # Check integrity
    $expectedChecksum = Invoke-WebRequest -Uri "https://github.com/ankddev/envfetch/releases/latest/download/envfetch-windows-amd64.exe.sha256"
    $expectedChecksum = [System.Text.Encoding]::UTF8.GetString($expectedChecksum.Content).TrimEnd()
    $actualChecksum = (Get-FileHash -Path $outFile -Algorithm SHA256).Hash.ToLower()
    if ($actualChecksum -ne $expectedChecksum) {
        Write-Host "Checksum mismatch!"
        Write-Host "Expected: \"$expectedChecksum\""
        Write-Host "Actual:   \"$actualChecksum\""
        Write-Host "Please download the file manually and verify its integrity."
        exit 1
    }

    # Add to PATH
    $userPath = [Environment]::GetEnvironmentVariable("Path", "User")
    if ($userPath -notlike "*$envfetchPath*") {
        [Environment]::SetEnvironmentVariable("Path", "$userPath;$envfetchPath", "User")
    }
} finally {
    Pop-Location
}

Write-Host "envfetch has been installed and added to your PATH. Please restart your terminal for the changes to take effect."
