# competitive-programming-harness

Codexを利用した競技プログラミング学習ハーネス。AtCoderの問題を題材に、Rustの実装力とアルゴリズムの発見力を学ぶためのローカル環境を提供します。

## Quick start

miseをインストールした後、プロジェクトルートで実行します。

```sh
mise install
mise run init abc001_c
```

`problems/abc001_c/problem.md` に問題文をローカル保存し、`problems/abc001_c/src/main.rs` と `problems/abc001_c/samples/*.in`、`problems/abc001_c/samples/*.out` を編集します。

```sh
mise run sample abc001_c
mise run verify
```

`mise run sample <problem-id>` は、ローカルsolutionをコンパイル・実行し、サンプル出力と比較するためのコマンドです。サンプル検証だけを行うため、隠しテストまで正しさを保証するものではありません。

`mise run verify` は、mise定義とHarness本体のformat、unit test、buildだけを検証します。`verify`単独では `problems/<id>` のsolutionを検証しません。

利用可能なタスクは `mise tasks` で確認できます。

## Local-only data

`problems/`、`learning/`、`generated/` はローカル専用です。AtCoder問題文、学習者の解答、振り返り、生成問題をPublic repositoryへcommitしないでください。

## M1 learning flow

1. `tutor` で問題理解、制約、愚直解を整理する。
2. 必要な場合だけ `hint` を一段階ずつ使う。
3. Rustの型・所有権・コンパイラエラーは `rust-tutor` で考える。
4. サンプル通過後に `algorithm-review` と `rust-review` を行う。
5. AC後に `reflection` で学習内容を記録する。

Codexは学習者が考える前に完成解答を提示せず、Rustコードを全面的に書き換えません。

## Scope

M1では、ローカル問題ワークスペース、Rustサンプル実行、学習支援Skillを提供します。Weakness自動推定、学習履歴の構造化、問題生成、Validator、Web UI、データベースはM2以降です。
