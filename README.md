# kandume

ターミナル上で動く、[Ratatui](https://github.com/ratatui-org/ratatui) 製の多重セッション用 CLI。**プロジェクト → セッション → ペイン**の階層と、[tmux](https://github.com/tmux/tmux) 風の **Ctrl-b プレフィックス**で操作します。各ペインは実際のシェル（PTY）がぶら下がります。

## 動作環境

- Rust ツールチェーン（2024 edition でビルド）
- Unix 系で PTY が使えること（macOS / Linux を想定）
- `$SHELL` が既定のログインシェルとして参照されます（未設定時は `/bin/zsh`）

## インストール

### Homebrew（macOS / Linuxbrew）

このリポジトリを [Homebrew tap](https://docs.brew.sh/Taps) として登録してからインストールします。

```sh
brew tap tarabakz25/kandume https://github.com/tarabakz25/kandume.git
brew install kandume
```

開発ブランチの最新を入れる場合は次です。

```sh
brew install kandume --HEAD
```

`--HEAD` は tap 先の `develop` をビルドします。

リリース後に Formula の tarball を新しいタグへ更新するときは、次で `sha256` を取得して `Formula/kandume.rb` の `url` / `sha256` を合わせてください。

```sh
curl -sL "https://github.com/tarabakz25/kandume/archive/refs/tags/v0.1.2.tar.gz" | shasum -a 256
```

（`v0.1.2` は実際のタグ名に読み替え。）

### ソースから

リポジトリをクローンしてリリースビルドします。

```sh
git clone https://github.com/tarabakz25/kandume.git
cd kandume
cargo build --release
```

バイナリは `target/release/kandume` に出力されます。

## 使い方

```sh
./target/release/kandume
```

起動すると左にプロジェクト一覧、右にアクティブプロジェクトのセッション／ペインが表示されます。キーボードはアクティブなペインのシェルへ、そのほかは UI とプレフィックスが処理します。

### Ctrl-b プレフィックス

一度 **Ctrl-b** を押すとプレフィックス状態になり、その後に別キーを押してコマンドを実行します（ステータス行にヒントが出ます）。

| キー | 動作 |
|------|------|
| **Ctrl-b** を続けて **Ctrl-b** | シェルへリテラル `^B`（0x02）を送信 |
| **t** | 新規プロジェクト（カレントは **ホームディレクトリ**） |
| **c** | 現在のプロジェクトにセッションを追加 |
| **%** | 現在のセッションを縦分割 |
| **"** | 現在のセッションを横分割 |
| **n** / **p** | 次／前のプロジェクト |
| **1–9** | プロジェクトを番号で選択 |
| **]** / **[** | 次／前のセッション |
| **o** / **;** | 次／前のペイン |
| **x** | アクティブなペインを閉じる（セッションにペインが 1 枚だけならセッションごと閉じて **ひとつ前のセッション** に移る。プロジェクトにセッションが残らなくなったら、その枠を **ホームディレクトリの新規プロジェクト**（セッション 1・ペイン 1）に置き換える） |
| **,** / **.** / **r** | プロジェクト／セッション／ペインの名前変更 |
| **d** | セッションを保存して終了 |
| **q** | 保存せず終了 |
| **?** | ヘルプの表示／非表示 |

リネーム中は **Enter** で確定、**Esc** でキャンセルです。

### マウス

マウスが使える端末では、左のプロジェクト行クリックで切り替え、ペイン上ではクリック・ドラッグ・ホイールを XTerm SGR 形式で PTY に転送します（例: Vim は `:set mouse=a`）。ホバーだけの動きは PTY に送らず、シェルにゴミ文字が出ないようにしてあります。

### セッションの保存

**Ctrl-b d** で終了したとき、レイアウト・名前・作業ディレクトリなどが TOML で保存されます。

- **パス**: プラットフォームのユーザー設定ディレクトリ直下の `kandume/session.toml`（例: Linux でよくあるのは `~/.config/kandume/session.toml`。macOS では `~/Library/Application Support/kandume/session.toml` になります）
- **Ctrl-b q** では保存されません

## 開発

```sh
cargo build
cargo test
cargo run
```
