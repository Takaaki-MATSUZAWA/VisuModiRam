# PowerShell build script for VisuModiRam (Debug)

Write-Host "Building VisuModiRam for Windows (Debug)..." -ForegroundColor Green

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

    # ビルド実行
    Write-Host "Building project..." -ForegroundColor Yellow
    cargo build --bin VisuModiRam
    if ($LASTEXITCODE -ne 0) {
        throw "Build failed"
    }

    Write-Host "✓ Build completed successfully!" -ForegroundColor Green
    Write-Host "Executable location: target\debug\VisuModiRam.exe" -ForegroundColor Cyan
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