# PowerShell run script for VisuModiRam

Write-Host "Running VisuModiRam..." -ForegroundColor Green

# CARGO_TARGET_DIRを一時的に無効化
$originalTargetDir = $env:CARGO_TARGET_DIR
$env:CARGO_TARGET_DIR = $null

try {
    # Rust nightlyを使用
    Write-Host "Setting Rust nightly toolchain..." -ForegroundColor Yellow
    rustup override set nightly
    if ($LASTEXITCODE -ne 0) {
        throw "Failed to set nightly toolchain"
    }

    # デバッグ版を実行
    Write-Host "Starting application..." -ForegroundColor Yellow
    RUST_LOG=debug cargo run --bin VisuModiRam
    if ($LASTEXITCODE -ne 0) {
        throw "Failed to run application"
    }
}
catch {
    Write-Host "✗ Error: $_" -ForegroundColor Red
    Write-Host "Press any key to continue..." -ForegroundColor Gray
    $null = $Host.UI.RawUI.ReadKey("NoEcho,IncludeKeyDown")
    exit 1
}
finally {
    # CARGO_TARGET_DIRを元に戻す
    $env:CARGO_TARGET_DIR = $originalTargetDir
} 