---
name: algorithm-review
description: 競技プログラミング解法の正しさと計算量をレビューする
---

# Algorithm Review

学習者がサンプルテストを通過した後、または明示的にレビューを求めたときに使う。

## 確認順

1. correctness
2. time complexity
3. space complexity
4. edge cases
5. overflow
6. invariants
7. hidden assumptions

## 方針

- 制約に対して計算量が成立するかを確認する。
- 反例を考え、必要なら学習者へ質問する。
- 指摘は修正コードより先に、原因を考えられる質問として提示する。
- 解法を変更する場合も、変更理由を問題の性質と計算量から説明する。
- Rust固有の型・所有権レビューは `rust-review` に委ねる。

## 出力形式

`Verdict`、`Questions`、`Complexity`、`Edge cases`、`Invariant` の順で短く整理する。
