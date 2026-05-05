# Contributing to kandume

## 前提環境

- Rust stable（edition 2024 が動く最新安定版を推奨）
- Unix 系 OS（macOS / Linux）—— PTY が必要なため Windows は非対応

## 開発フロー

```sh
git clone https://github.com/tarabakz25/kandume.git
cd kandume
cargo build       # 動作確認
cargo test        # テスト実行
cargo clippy      # lint
cargo fmt         # フォーマット
```

## ブランチ戦略

| ブランチ | 用途 |
|----------|------|
| `main`   | リリース済み安定版。直プッシュ禁止。 |
| `develop`| 開発統合ブランチ。Homebrew `--HEAD` はここを参照。 |
| `feat/*` | 機能追加 |
| `fix/*`  | バグ修正 |
| `chore/*`| ビルド・CI・ドキュメント等の雑務 |

作業は `develop` からブランチを切り、PR で `develop` へマージします。`develop → main` は release タグを打つときにまとめてマージします。

## コミットメッセージ

[Conventional Commits](https://www.conventionalcommits.org/) に従います。

```
<type>(<scope>): <summary>
```

- **type**: `feat` / `fix` / `refactor` / `test` / `docs` / `chore`
- **scope**: 任意（例: `input`, `layout`, `session`, `ui`）
- **summary**: 英語または日本語、命令形で

例:
```
feat(input): add Ctrl-b z to zoom active pane
fix(session): migrate v1 schema when project list is empty
```

## プルリクエスト

1. `develop` の最新を取り込んでからブランチを作る
2. CI（`fmt --check` / `clippy` / `test`）が全 pass であること
3. `CHANGELOG.md` の `## Unreleased` セクションに変更内容を追記する
4. PR の説明欄に「何を・なぜ変えたか」を書く
5. PTY の振る舞いを変えた場合は、手動動作確認の結果も記載する

## テスト方針

- ロジックテストは `src/app.rs` の `#[cfg(test)]` に書く
- PTY・TUI の統合テストはセットアップが難しいため現時点では対象外
- `cargo test` が通ることを必須条件とする

## コーディング規約

- クレート内公開は `pub(crate)`、外部公開が必要なものだけ `pub`
- `cargo clippy` の警告はゼロを保つ
- `cargo fmt` のデフォルト設定に従う（`rustfmt.toml` なし）
- パニックを起こす可能性のあるパスは `install_panic_hook()` が機能するよう維持する

## セッション schema 変更時の注意

`src/session.rs` には 3 世代のスキーマが `#[serde(untagged)]` で共存しています。
スキーマを変更する場合は既存の全バリアントが引き続き正しくデシリアライズできることを確認し、必要なら新バリアントを追加してください。デシリアライズ順の変更は避けてください。

## リリース手順（メンテナ向け）

1. `CHANGELOG.md` の `## Unreleased` をバージョン番号と日付に変える
2. `Cargo.toml` の `version` を更新
3. `cargo build --release` でエラーがないことを確認
4. `develop → main` へ PR をマージ
5. `main` 上で `git tag v<version>` を打ち push
6. GitHub Release を作成してタグに紐付ける
7. Formula の tarball URL と sha256 を更新する PR を出す
