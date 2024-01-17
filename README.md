# STM32EguiMonitor

Debugging tools for STM32 to visualize and modify variables

## 概要
- [STM32CubeMonitor](https://www.st.com/ja/development-tools/stm32cubemonitor.html)を真似て作ったソフト
- Rust + Eguiで実装されている
- 名前は後で変える予定

## できること
- ELFファイルから変数の一覧の取得と検索
- ST-Linkを通してマイコンで実行中のコードの変数の可視化と変更
- 上記を行うGUIを提供
- それらのレイアウトを自由に変更できる

## 環境構築
- [Arm GNU Toolchain](https://developer.arm.com/downloads/-/gnu-rm)のインストール
    - 上記のリンクから環境に合わせてインストール
        - windowsで[Chocolatey](https://community.chocolatey.org/)を導入済みの場合は`choco install gcc-arm-embedded`でインストール
    - "arm-none-eabi-gdb.exe"が使えるように環境変数などでパスを通しておく
- Rustの環境構築
    - [rustup](https://www.rust-lang.org/tools/install)から"DOWNLOAD RUSTUP-INIT.EXE(64-BIT)"をダウンロード＆インストール
        - `cargo -V`などでコマンドが通ることを確認 
- probe-rsのインストール
    - `cargo install probe-rs --features cli`
    - ターゲットマイコンの型番確認のために使う
        - インストールしなくても問題ない
- git
    - 入ってるっしょ

## インストール
```bash
git clone https://github.com/Takaaki-MATSUZAWA/STM32EguiMonitor.git
cd STM32EguiMonitor

# nightylyチャネルへ切り替え
rustup override set nightly
# ビルド
cargo buid --release
# target/release/のどっかにSTM32EguiMonitor.exeが出来上がる
```

## 操作方法
あとで書く