# PowerShell build script for VisuModiRam (Release)

Write-Host "Building VisuModiRam for Windows (Release)..." -ForegroundColor Green

# CARGO_TARGET_DIRを一時的に無効化
$originalTargetDir = $env:CARGO_TARGET_DIR
$env:CARGO_TARGET_DIR = $null

try {
    # Rust nightlyを使用してビルド
    Write-Host "Setting Rust nightly toolchain..." -ForegroundColor Yellow
    rustup override set nightly
    if ($LASTEXITCODE -ne 0) {
        throw "Failed to set nightly toolchain"
    }

    # 依存関係をアップデート
    Write-Host "Updating dependencies..." -ForegroundColor Yellow
    cargo update
    if ($LASTEXITCODE -ne 0) {
        throw "Failed to update dependencies"
    }

    # リリースビルド実行
    Write-Host "Building project (Release)..." -ForegroundColor Yellow
    cargo build --release
    if ($LASTEXITCODE -ne 0) {
        throw "Release build failed"
    }

    Write-Host "✓ Release build completed successfully!" -ForegroundColor Green
    Write-Host "Executable location: target\release\VisuModiRam.exe" -ForegroundColor Cyan
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