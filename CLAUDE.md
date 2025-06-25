# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## プロジェクト概要

VisuModiRamは、STM32マイコン用のRust製GUIデバッグツールです。ST-Linkを使用してELFファイルの解析、変数の可視化・変更、リアルタイムグラフ化を行います。

## プロジェクト構造

```
src/
├── main.rs                    # アプリケーションのエントリーポイント
├── visumodiram.rs            # メインアプリケーションロジックとタブ管理
├── debugging_tools/          # マイコンデバッグ機能
│   ├── elf_parser.rs        # ELFファイル解析とDWARF情報処理
│   ├── probe_interface.rs   # ST-Linkプローブ通信
│   └── memory_interface.rs  # メモリアクセス抽象化
└── monitor_ui/              # UI コンポーネント
    ├── setting_panel.rs     # 設定タブ（プローブ設定、変数選択）
    ├── main_monitor.rs      # メインモニタタブ（変数表示、グラフ）
    ├── logging_tab.rs       # 高速ロギング専用タブ
    ├── data_viewer_tab.rs   # データ閲覧・プロット専用タブ
    └── widgets/            # カスタムUI ウィジェット
        ├── edit_table.rs   # 編集可能テーブル
        ├── graph_monitor.rs # リアルタイムグラフ
        ├── gauge.rs        # ゲージ表示
        └── ...
```

## 主要技術スタック

- **GUI**: eframe/egui (0.30.0) - immediate mode GUI
- **プローブ通信**: probe-rs (0.29.0) - ST-Link/J-Link対応
- **ELF解析**: ddbug_parser - DWARF情報から変数情報抽出
- **データ保存**: sensorlog-ram (自作) - RAMベース高速ロギング
- **シリアライゼーション**: serde + ron - レイアウト保存

## ビルドとテスト

```bash
# ビルド
cargo build --release --bin VisuModiRam

# 開発版実行
cargo run

# 最新のnightly版を使用（probe-rs 0.29.0要件）
rustup override set nightly
```

## 開発タスクと課題

### probe-rs 0.29.0アップデート完了 ✅
- probe-rs を 0.21.1 から 0.29.0 に更新済み
- Rust nightly版も最新に更新
- **全機能復活完了**: `probe_rs::config` 関連機能を新APIで実装
  - ターゲットMCU自動検出機能: `Registry::get_target_by_name()` 使用
  - メモリサイズ取得機能: `MemoryRegion::Ram/Nvm` から正確な計算
  - チップ検索・候補表示機能: `Registry::search_chips()` 使用
  - 段階的検索機能: 文字列を削りながら候補を探索
- 全てのconfig関連機能が正常動作確認済み

### 依存関係の更新が必要
- 主要依存関係は最新版に更新済み
- serde_traitobject 削除 (最新Rust nightlyで非互換)

### sensorlog-ram クレート作成完了 ✅
- 従来のsensorlogクレートの低速I/O問題を解決
- **RAMベース高速ロギング**: メモリ内保存で10倍以上の性能向上
- **定期的自動保存**: 30秒間隔でディスクに安全保存
- **メモリ管理**: 200MB制限、自動クリーンアップ機能
- **互換API**: 既存コードの変更最小限で移行完了
- **バックグラウンド保存**: UI応答性を損なわない設計
- **データ取得時間範囲バグ修正**: モニタ表示問題を解決

### 高速ロギング・データ閲覧機能の完成 ✅
- **4タブ体制**: Setting、Main Monitor、High-Speed Logging、Data Viewer
- **高速ロギングタブ**: ミリ秒オーダーでの高速ロギング、リアルタイム統計、プロット表示
- **データ閲覧タブ**: CSV ファイル読み込み、時間範囲指定、変数選択プロット
- **プロット機能**: 両タブで変数オン/オフ、統計情報、ズーム・ドラッグ対応
- **ブラウザ操作対応**（Web版）

## アーキテクチャの重要ポイント

### タブベースアーキテクチャ
- `visumodiram.rs`でタブ管理とアプリケーション状態
- 各タブは独立したeframe::Appトレイト実装
- **4タブ構成**: SettingTab、MainMonitorTab、LoggingTab、DataViewerTab
- タブ切り替え時にプローブ設定が自動反映

### プローブ通信の抽象化
- `ProbeInterface`がprobe-rs APIをラップ
- `WatchSetting`で監視対象変数を管理
- フラッシュ書き込み進捗の追跡
- **sensorlog-ram統合**: RAMベース高速ロギング機能

### ターゲット設定とconfig機能
- `Registry::from_builtin_families()`で組み込みターゲット群を初期化
- `TargetMCUInfo`でチップ検証、メモリサイズ計算、候補検索を管理
- 不正確なチップ名入力時の段階的検索とRAM/ROM容量表示

### ELF解析とDWARF処理
- `elf_parser.rs`でELFファイルから変数情報抽出
- DWARF デバッグ情報を使用してアドレス特定
- 型情報（u8, u16, u32, float等）の自動判定

### データ永続化
- **sensorlog-ram**: RAMベース高速ロギング + 定期ディスク保存
- レイアウト情報はRON形式で保存
- serde実装によるシリアライゼーション
- Load/Save機能でレイアウト共有
- **複数保存形式**: Binary（高速）、JSON（可読）、CSV（Excel互換）

## テストファームウェア

G474_test_firmwareディレクトリ：
- 主要コード: `G474_test_firmware/Core/Src/main_cpp.cpp`
- ELFファイル: `G474_test_firmware/build/Debug/G474_test_firmware.elf`
- マップファイル: 同じディレクトリ内
- **重要**: 別システムでビルド済み、再ビルド不可

## カスタムUI ウィジェット

- `EditTable`: 変数値の編集可能テーブル
- `GraphMonitor`: リアルタイムプロット表示
- `Gauge`: 円形ゲージ表示
- `ToggleSwitch`: ON/OFFスイッチ
- `Button`: カスタムボタン

## 開発時の注意事項

- プローブ接続前の動作確認はテストファームウェアを使用
- ELF解析エラーは型情報の欠損が原因の場合が多い
- ウィジェットのレイアウトはegui_extras使用
- Web版対応のためwasm32条件コンパイル多用

## 最近の重要な修正

### 高速ロギング専用タブ実装完了 (2025-06-25)
- **機能**: 新しいLogging専用タブ（`logging_tab.rs`）を追加
- **特徴**: ミリ秒オーダーの高速ロギング、RAMベース記録、プロット表示
- **UI構成**: 3パネル（制御・統計・データ表示+プロット）
- **プロット**: ログ停止後に表示、変数選択可能、統計情報付き

### データビューアタブ実装完了 (2025-06-25)
- **機能**: 保存データ専用の読み込み・プロット表示タブ（`data_viewer_tab.rs`）
- **対応形式**: CSV ファイル読み込み対応
- **UI構成**: 3パネル（ファイル管理・プロット制御・データ表示）
- **プロット機能**: 時間範囲指定、変数選択、統計表示、ズーム・ドラッグ対応

### タブアーキテクチャ拡張 (2025-06-25)
- **追加タブ**: LoggingTab（高速ロギング）、DataViewerTab（データ閲覧）
- **タブ統合**: `visumodiram.rs`に4タブ体制で統合完了
- **プローブ設定**: タブ切り替え時の自動設定反映機能

### データ取得時間範囲バグ修正 (2025-06-25)
- **問題**: モニタ画面で値が一瞬しか表示されない
- **原因**: `probe_interface.rs`の`get_newest_date`と`get_log_vec`で時間範囲が逆
- **修正**: `load_data`の`time_start`と`time_end`引数を正しい順序に修正
- **影響**: TableView、GraphMonitor等の全てのモニタ機能が正常動作

## Memories

- Memorized the update guidance for this project and its requirements