# Architecture

## Boundary

このリポジトリは、再利用可能なRust CLI、mise設定、Codex Skill、テンプレート、ドキュメントをPublicとして管理する。

以下はlocal-onlyである。

```text
problems/   AtCoder問題文、サンプル、学習者のRust解答
learning/   振り返り
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
problems/<id>/
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

`cph test` は各 `samples/*.in` を対応する `.out` と比較する。比較は空白区切りトークン単位で行い、末尾空白と改行形式の差を無視する。

`mise run sample <id>` は `problems/<id>` のlocal solutionをコンパイル・実行し、サンプルを検証する。サンプル検証だけを行うため、隠しテストまで正しさを保証するものではない。`mise run verify` はこのsolutionを検証しない。

## Learning layer

Codex Skillは学習の進め方を担当し、Rust CLIはローカルワークスペースの作成とサンプル実行だけを担当する。

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
