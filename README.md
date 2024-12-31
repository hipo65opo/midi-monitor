# MIDI Monitor

Windows用のMIDIモニターアプリケーションです。MIDIデバイスからのメッセージをリアルタイムで表示します。

## 機能

- MIDIデバイスの検出と選択
- リアルタイムでのMIDIメッセージ表示
- メッセージの種類別表示（Note On/Off、Control Change、Program Change等）
- メッセージ履歴の管理

## 必要要件

- Windows 10以降
- [Rust](https://www.rust-lang.org/tools/install)
- Visual Studio Build Tools（C++デスクトップ開発ワークロード）

## インストール

```bash
# リポジトリのクローン
git clone https://github.com/yourusername/midi-monitor.git
cd midi-monitor

# ビルドと実行
cargo run
```

## 使用方法

1. アプリケーションを起動
2. "Refresh Devices"ボタンをクリックしてMIDIデバイスを検出
3. 表示されたデバイス一覧から使用したいデバイスを選択
4. MIDIメッセージがリアルタイムで表示されます

## ライセンス

MIT License 