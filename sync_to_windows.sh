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
cp -r sensorlog-ram "$WINDOWS_DIR/"

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

# Windows用のPowerShellスクリプトをコピー
echo "🖥️  Copying Windows PowerShell build scripts..."

# PowerShellスクリプトディレクトリからファイルをコピー
if [ -d "powershell_scripts" ]; then
    cp powershell_scripts/*.ps1 "$WINDOWS_DIR/"
    echo "✓ PowerShell scripts copied successfully"
else
    echo "⚠️  powershell_scripts directory not found, skipping PowerShell scripts"
fi

# Windows環境向けREADMEをコピー
if [ -f "powershell_scripts/README_WINDOWS.md" ]; then
    cp powershell_scripts/README_WINDOWS.md "$WINDOWS_DIR/"
    echo "✓ README_WINDOWS.md copied successfully"
else
    echo "⚠️  README_WINDOWS.md not found in powershell_scripts directory"
fi

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