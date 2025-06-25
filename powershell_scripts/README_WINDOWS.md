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