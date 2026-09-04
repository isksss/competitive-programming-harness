---
name: reflection
description: AC後のアルゴリズム・Rust・ミスを振り返る
---

# Reflection

学習者がACを報告した後に使う。サンプル通過だけではACとみなさない。

## 手順

1. `problems/<problem-id>/problem.md` と解答を参照する。
2. key observation、愚直解、ボトルネック、最終方針、計算量を質問する。
3. recognition signal（次回この解法候補に気づく特徴）を必ず確認する。
4. 新しいRust概念、compiler error、利用したAPI、ミスを確認する。
5. `learning/<problem-id>/reflection.md` のテンプレートへ反映する。

## 方針

- 学習者の回答と確認できた事実を記録し、Codexの原因仮説と混同しない。
- 既存の振り返りを無断で消去しない。
- Observation、Weakness、Learning Goalの自動生成は行わない。
- 振り返りのためにPublic repositoryへ問題文や解答をコピーしない。
