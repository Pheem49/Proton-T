Write-Host "Installing Proton-T for Windows..." -ForegroundColor Cyan

$REPO_URL = "https://github.com/Pheem49/Proton-T.git"
$INSTALL_DIR = Join-Path $HOME ".proton-t"

# 1. Bootstrap: If not in the project folder, clone it to ~/.proton-t
if (-not (Test-Path "Cargo.toml")) {
    if (-not (Get-Command git -ErrorAction SilentlyContinue)) {
        Write-Host "Error: git is not installed." -ForegroundColor Red
        return
    }
    Write-Host "Downloading Proton-T..."
    if (Test-Path $INSTALL_DIR) {
        Set-Location $INSTALL_DIR
        git pull
    } else {
        git clone $REPO_URL $INSTALL_DIR
        Set-Location $INSTALL_DIR
    }
}

$PROJECT_DIR = $PWD.Path
$INIT_SCRIPT = Join-Path $PROJECT_DIR "init.ps1"

# 2. Install as Rust Binary
Write-Host "Installing Rust binary..."
if (-not (Get-Command cargo -ErrorAction SilentlyContinue)) {
    Write-Host "Rust is not installed. Please install Rust from https://rustup.rs/ before running this script." -ForegroundColor Red
    return
}
cargo install --path .

# 3. Integration in PowerShell Profile (Avoid duplicates)
if (-not $PROFILE) {
    Write-Host "Error: No PowerShell profile found." -ForegroundColor Red
    return
}

if (-not (Test-Path $PROFILE)) {
    $dir = Split-Path $PROFILE
    if (-not (Test-Path $dir)) { New-Item -ItemType Directory -Path $dir -Force }
    New-Item -ItemType File -Path $PROFILE -Force
}

$line = "proton-t init powershell | Out-String | Invoke-Expression"
$content = Get-Content $PROFILE -ErrorAction SilentlyContinue
$found = $false
if ($content) {
    foreach ($c in $content) {
        if ($c -like "*proton-t init powershell*") {
            $found = $true
            break
        }
    }
}

if (-not $found) {
    Add-Content -Path $PROFILE -Value "`n# Proton-T Integration`n$line"
    Write-Host "Updated PowerShell profile: $PROFILE" -ForegroundColor Green
}

Write-Host "Done! Please restart PowerShell or run: . `$PROFILE" -ForegroundColor Green
Write-Host "Note: You might need to run 'Set-ExecutionPolicy RemoteSigned' if scripts are blocked." -ForegroundColor Yellow
