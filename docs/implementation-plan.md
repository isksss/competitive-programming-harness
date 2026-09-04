# Implementation roadmap

## M1 — Core Learning Harness

- miseによるRust・Cargo・rustfmtと開発タスクの管理
- `cph init` によるlocal workspace初期化
- `cph test` によるRustサンプル実行と出力比較
- tutor、hint、rust-tutor、algorithm-review、rust-review、reflection Skill
- Public/local境界の文書化

完了条件は、AtCoder ABC C相当の問題について、自力思考からAC後の振り返りまでを完走できること。

## M2 — Learning History

- Concept、Attempt、Observation schema
- reflectionからObservationへの記録
- mistake-analysis Skill

## M3 — Weakness Model

- Weakness、Learning Goal schema
- recognition、understanding、implementation、independenceの評価
- 観測根拠を説明できる弱点導出

## M4 — Generated Drill

- problem-generator Skill
- reference solution、brute-force oracle
- randomized differential testing
- problem-validator Skill

M2以降はM1の利用結果を確認してから実装する。
