# Create envfetch directory in AppData/Roaming
$envfetchPath = "$env:APPDATA\envfetch"
New-Item -ItemType Directory -Force -Path $envfetchPath -ErrorAction SilentlyContinue | Out-Null
Write-Host "Installing envfetch to $envfetchPath"

Push-Location $envfetchPath

try {
    $outFile = "envfetch.exe"
    Invoke-WebRequest -Uri "https://github.com/ankddev/envfetch/releases/latest/download/envfetch-windows-amd64.exe" -OutFile $outFile

    # Add to PATH
    $userPath = [Environment]::GetEnvironmentVariable("Path", "User")
    if ($userPath -notlike "*$envfetchPath*") {
        [Environment]::SetEnvironmentVariable("Path", "$userPath;$envfetchPath", "User")
    }
} finally {
    Pop-Location
}

Write-Host "envfetch has been installed and added to your PATH. Please restart your terminal for the changes to take effect."
