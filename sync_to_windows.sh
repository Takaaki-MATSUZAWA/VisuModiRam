#!/bin/bash

# Windows環境への同期スクリプト
# Usage: ./sync_to_windows.sh

# 同期先ディレクトリ
WINDOWS_DIR="/mnt/c/Users/takaa/rust_workspace/VisuModiRam_CC_test"

echo "🔄 Syncing VisuModiRam to Windows environment..."

# ディレクトリが存在しない場合は作成
mkdir -p "$WINDOWS_DIR"

# 必要なファイルをコピー
echo "📁 Copying source files..."

# ルートファイル
cp Cargo.toml "$WINDOWS_DIR/"
cp README.md "$WINDOWS_DIR/" 2>/dev/null || echo "⚠️  README.md not found, skipping"

# CLAUDE.mdをコピー（プロジェクト情報として重要）
cp CLAUDE.md "$WINDOWS_DIR/"

# srcディレクトリ全体をコピー
echo "📂 Copying src directory..."
cp -r src "$WINDOWS_DIR/"
cp -r assets "$WINDOWS_DIR/"

# テストファームウェアは除外（サイズが大きいため）
echo "⚠️  Skipping G474_test_firmware (excluded from sync)"

# Cargo.lockがあればコピー
if [ -f "Cargo.lock" ]; then
    cp Cargo.lock "$WINDOWS_DIR/"
fi

# .gitignoreをコピー
if [ -f ".gitignore" ]; then
    cp .gitignore "$WINDOWS_DIR/"
fi

# Windows用のPowerShellスクリプトを作成
echo "🖥️  Creating Windows PowerShell build scripts..."

cat > "$WINDOWS_DIR/build.ps1" << 'EOF'
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
EOF

cat > "$WINDOWS_DIR/build_release.ps1" << 'EOF'
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
EOF

cat > "$WINDOWS_DIR/run.ps1" << 'EOF'
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
    cargo run
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
EOF

cat > "$WINDOWS_DIR/clean.ps1" << 'EOF'
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
EOF

cat > "$WINDOWS_DIR/setup.ps1" << 'EOF'
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
EOF

# Windows環境での注意事項を含むREADMEを作成
cat > "$WINDOWS_DIR/README_WINDOWS.md" << 'EOF'
# VisuModiRam - Windows Build Instructions

このディレクトリはLinux環境からWindows環境にコピーされたVisuModiRamプロジェクトです。

## 前提条件

1. **Rust Toolchain**: 最新のRust nightlyが必要
   ```cmd
   rustup install nightly
   rustup default nightly
   ```

2. **Visual Studio Build Tools**: Windowsでのネイティブ依存関係のビルドに必要
   - Visual Studio 2019/2022のC++ビルドツール
   - または Visual Studio Community with C++ development tools

## ビルド手順

### 方法1: PowerShellスクリプトを使用（推奨）
```powershell
# 初回セットアップ
.\setup.ps1

# デバッグビルド
.\build.ps1

# リリースビルド
.\build_release.ps1

# 実行
.\run.ps1

# クリーンビルド
.\clean.ps1
```

### 方法2: 手動ビルド（CARGO_TARGET_DIR対応）
```powershell
# CARGO_TARGET_DIRを一時的に無効化
$originalTargetDir = $env:CARGO_TARGET_DIR
$env:CARGO_TARGET_DIR = $null

rustup override set nightly
cargo update
cargo build
cargo run

# 元に戻す
$env:CARGO_TARGET_DIR = $originalTargetDir
```

## 注意事項

1. **probe-rs 0.29.0**: 最新バージョンを使用しています
2. **Nightly Rust**: trait upcasting coercionが必要なため
3. **ST-Link**: WindowsでST-Linkドライバーが正しくインストールされている必要があります
4. **テストファームウェア**: 必要に応じて別途ELFファイルを用意してテスト
5. **CARGO_TARGET_DIR**: 環境変数の設定に関係なく、このプロジェクトは./targetディレクトリにビルドされます

## トラブルシューティング

### ビルドエラーが発生した場合
1. Rust nightlyが最新であることを確認
2. `cargo clean` でクリーンビルド
3. Visual Studio Build Toolsがインストールされていることを確認

### プローブが認識されない場合
1. ST-Linkドライバーが正しくインストールされているか確認
2. デバイスマネージャーでプローブが認識されているか確認
3. 管理者権限でアプリケーションを実行してみる

## ファイル構造

```
VisuModiRam_CC_test/
├── Cargo.toml              # プロジェクト設定
├── CLAUDE.md               # プロジェクト情報
├── src/                    # ソースコード
# ├── G474_test_firmware/     # テスト用ファームウェア（同期対象外）
├── build.ps1               # デバッグビルド用（PowerShell）
├── build_release.ps1       # リリースビルド用（PowerShell）
├── run.ps1                 # 実行用（PowerShell）
├── clean.ps1               # クリーンビルド用（PowerShell）
├── setup.ps1               # PowerShell初回セットアップ
└── README_WINDOWS.md       # このファイル
```

## 開発時の注意

- 変更をLinux環境に反映する場合は、手動で同期する必要があります
- Windows固有の問題が発生した場合は、CLAUDE.mdを参照してください
EOF

# 実行権限を付与
chmod +x "$WINDOWS_DIR"/*.bat 2>/dev/null || true

echo "✅ Sync completed successfully!"
echo ""
echo "📁 Files copied to: $WINDOWS_DIR"
echo ""
echo "🖥️  Windows environment setup:"
echo "   1. Open Command Prompt or PowerShell as Administrator"
echo "   2. Navigate to: $WINDOWS_DIR"
echo "   3. Run setup.ps1 (PowerShell)"
echo ""
echo "📋 Available PowerShell scripts:"
echo "   - setup.ps1         : Initial setup and dependency check"
echo "   - build.ps1         : Debug build (CARGO_TARGET_DIR disabled)"
echo "   - build_release.ps1 : Release build (CARGO_TARGET_DIR disabled)"
echo "   - run.ps1           : Run debug version (CARGO_TARGET_DIR disabled)"
echo "   - clean.ps1         : Clean build artifacts (CARGO_TARGET_DIR disabled)"
echo ""
echo "📖 See README_WINDOWS.md for detailed instructions"