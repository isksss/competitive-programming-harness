# Architecture

## Boundary

このリポジトリは、再利用可能なRust CLI、mise設定、Codex Skill、テンプレート、ドキュメントをPublicとして管理する。

以下はlocal-onlyである。

```text
problems/   AtCoder問題文、サンプル、学習者のRust解答
learning/   振り返り、Hintのセッション状態
generated/  将来の生成問題
```

## Runtime flow

```text
mise run init <id>
        |
        v
   cph init
        |
        v
problems/<id>/ と learning/<id>/
        |
        v
mise run sample <id>
        |
        v
   cph test
        |
        v
sample output comparison
```

`cph init <id>` は問題workspaceと学習workspaceを作成し、`learning/<id>/session.yaml` を `hint_level: 0`、`solution_revealed: false` で初期化する。既存workspaceの再初期化では既存ファイルを上書きしない。

`cph test` は各 `samples/*.in` を対応する `.out` と比較する。比較は空白区切りトークン単位で行い、末尾空白と改行形式の差を無視する。

## Learning layer

Codex Skillは学習の進め方と `learning/<id>/session.yaml` のHint進行状態の読み書きを担当し、Rust CLIはローカルワークスペースの作成とサンプル実行だけを担当する。

Hintのセッション状態は固定2項目（`hint_level: 0..=6`、`solution_revealed: true|false`）だけを持つ。不正状態はfail-closedで扱い、履歴、問題文、解答本文は保存しない。

```text
tutor
  -> hint
  -> rust-tutor
  -> algorithm-review / rust-review
  -> reflection
```

M1では学習データの構造化分析、問題選択、問題生成、Web UI、データベースを扱わない。

## Tool and task management

プロジェクトのツールとタスクは `mise.toml` を唯一の入口とする。

```text
mise run init <id>    local workspace creation
mise run sample <id>  sample execution
mise run verify       format, unit test, build, task validation
```
