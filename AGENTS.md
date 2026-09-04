# Project instructions

## 開発入口

- ツールと開発タスクはプロジェクト直下の `mise.toml` で管理する。
- Rust、Cargo、rustfmtの実行と検証は `mise run` 経由で行う。
- 問題環境は `mise run init <problem-id>` で作成し、サンプルは `mise run sample <problem-id>` で実行する。
- Harness本体の変更後は `mise run verify` を実行する。これはmise定義とHarness本体のformat、unit test、buildだけを検証し、`problems/<id>` のsolutionは検証しない。
- local solutionの確認は `mise run sample <problem-id>` で行う。これはサンプル検証であり、隠しテストまで保証しない。`verify`単独ではsolutionを検証しない。

## 学習方針

- 目的は解答の代行ではなく、学習者が次回自力で解ける状態を作ること。
- `tutor` → `hint` → `rust-tutor` → `algorithm-review` / `rust-review` → `reflection` の順で使う。
- 学習者が考える前に完成解答やアルゴリズム名を提示しない。
- ヒントはLevel 0から1段階ずつ進め、未解決問題でLevel 6へ自動遷移しない。
- Rustのcompiler errorは、型・所有権・借用の状態を学習者が説明してから最小限のヒントを出す。
- idiomaticという理由だけでコードを全面的に書き換えない。

## Public / local boundary

- `problems/`、`learning/`、`generated/` はlocal-onlyである。
- AtCoder問題文、学習者の解答、振り返り、生成問題をcommitしない。
- 秘密情報、認証情報、個人情報をリポジトリへ追加しない。
