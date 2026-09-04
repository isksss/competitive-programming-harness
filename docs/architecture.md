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

`cph test` は各 `samples/*.in` を対応する `.out` と比較する。比較は行ごとのトークン列で行い、各行内の先頭・末尾・連続する ASCII space/tab、LF と CRLF の違い、最終改行1個の有無を無視する。一方、トークンの行所属・順序・空行位置は一致必須である。これは AtCoder の判定を完全に再現するものではなく、ローカル学習用に出力の行構造を維持して確認する仕様である。

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
mise run sample <id>  sample execution
mise run verify       format, unit test, build, task validation
```
