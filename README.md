# VisuModiRam

Debugging tools for STM32 to visualize or modify variables on RAM

![monitor_capture](https://github.com/user-attachments/assets/81b2db67-42f3-442e-9ec4-2ec249023daa)

## 概要
- [STM32CubeMonitor](https://www.st.com/ja/development-tools/stm32cubemonitor.html)を真似て作ったソフト
- Rust + Eguiで実装されている

## できること
- ELFファイルから変数の一覧の取得と検索
- ST-Linkを通してマイコンで実行中のコードの変数の可視化と変更
- 上記を行うGUIを提供
- それらのレイアウトを自由に変更できる

## 環境構築
- Rustの環境構築
    - [rustup](https://www.rust-lang.org/tools/install)から"DOWNLOAD RUSTUP-INIT.EXE(64-BIT)"をダウンロード＆インストール
        - `cargo -V`などでコマンドが通ることを確認 
- probe-rsのインストール
    - `cargo install probe-rs --features cli`
    - ターゲットマイコンの型番確認のために使う
        - インストールしなくても問題ない

## インストール
```bash
git clone https://github.com/Takaaki-MATSUZAWA/VisuModiRam.git
cd VisuModiRam

# nightylyチャネルへ切り替え
rustup override set nightly-2024-01-01
# ビルド
cargo build --release
# target/release/のどっかにVisuModiRam.exeが出来上がる
```

## 操作方法

https://github.com/user-attachments/assets/d9b431ca-3f7e-4cb2-b0ac-9d9201009f6b
