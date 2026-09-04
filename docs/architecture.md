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

`cph test` は各 `samples/*.in` を対応する `.out` と比較する。比較は行ごとのトークン列で行い、各行内の先頭・末尾・連続する ASCII space/tab、LF と CRLF の違い、最終改行1個の有無を無視する。一方、トークンの行所属・順序・空行位置は一致必須である。これは AtCoder の判定を完全に再現するものではなく、ローカル学習用に出力の行構造を維持して確認する仕様である。

`mise run sample <id>` は `problems/<id>` のlocal solutionをコンパイル・実行し、サンプルを検証する。サンプル検証だけを行うため、隠しテストまで正しさを保証するものではない。`mise run verify` はこのsolutionを検証しない。

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
mise run sample <id>  local solution sample execution and comparison
mise run verify       task validation and Harness format, unit test, build only
```

`mise run verify` はmise定義とHarness本体のformat、unit test、buildだけを対象とし、`problems/<id>` のsolutionは検証しない。local solutionはproblem IDを指定した `mise run sample <problem-id>` でサンプル検証する。
