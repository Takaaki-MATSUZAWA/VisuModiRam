# PowerShell setup script for VisuModiRam

Write-Host "Setting up VisuModiRam development environment..." -ForegroundColor Green

# CARGO_TARGET_DIRを一時的に無効化
$originalTargetDir = $env:CARGO_TARGET_DIR
$env:CARGO_TARGET_DIR = $null

try {
    # Rust nightlyのインストール確認
    Write-Host "Checking Rust nightly installation..." -ForegroundColor Yellow
    rustup override set nightly
    if ($LASTEXITCODE -ne 0) {
        throw "Failed to set Rust nightly. Please install rustup first."
    }
    Write-Host "✓ Rust nightly set successfully" -ForegroundColor Green

    # 依存関係の更新
    Write-Host "Updating dependencies..." -ForegroundColor Yellow
    cargo update
    if ($LASTEXITCODE -ne 0) {
        throw "Failed to update dependencies"
    }
    Write-Host "✓ Dependencies updated successfully" -ForegroundColor Green

    # ビルドテスト
    Write-Host "Testing build..." -ForegroundColor Yellow
    cargo check
    if ($LASTEXITCODE -ne 0) {
        throw "Build check failed. Please check the error messages above"
    }
    Write-Host "✓ Build check passed" -ForegroundColor Green

    Write-Host "Setup completed successfully!" -ForegroundColor Green
    Write-Host ""
    Write-Host "Available PowerShell scripts:" -ForegroundColor Cyan
    Write-Host "  .\build.ps1         - Debug build" -ForegroundColor Cyan
    Write-Host "  .\build_release.ps1 - Release build" -ForegroundColor Cyan
    Write-Host "  .\run.ps1           - Run debug version" -ForegroundColor Cyan
    Write-Host "  .\clean.ps1         - Clean build artifacts" -ForegroundColor Cyan
    Write-Host ""
    Write-Host "Note: CARGO_TARGET_DIR is temporarily disabled for this project" -ForegroundColor Yellow
}
catch {
    Write-Host "✗ Error: $_" -ForegroundColor Red
    exit 1
}
finally {
    # CARGO_TARGET_DIRを元に戻す
    $env:CARGO_TARGET_DIR = $originalTargetDir
}

Write-Host "Press any key to continue..." -ForegroundColor Gray
$null = $Host.UI.RawUI.ReadKey("NoEcho,IncludeKeyDown") 