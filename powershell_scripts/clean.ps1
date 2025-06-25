# PowerShell clean script for VisuModiRam

Write-Host "Cleaning VisuModiRam build artifacts..." -ForegroundColor Green

# CARGO_TARGET_DIRを一時的に無効化
$originalTargetDir = $env:CARGO_TARGET_DIR
$env:CARGO_TARGET_DIR = $null

try {
    Write-Host "Running cargo clean..." -ForegroundColor Yellow
    cargo clean
    if ($LASTEXITCODE -ne 0) {
        throw "Clean failed"
    }

    Write-Host "✓ Clean completed successfully!" -ForegroundColor Green
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