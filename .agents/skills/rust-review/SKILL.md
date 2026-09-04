---
name: rust-review
description: Rust解答の安全性・計算量・可読性をレビューする
---

# Rust Review

学習者がサンプルテストを通過した後、または明示的にレビューを求めたときに使う。

## 確認順

1. correctness
2. type / ownership safety
3. time and space complexity
4. readability
5. idiomatic Rust

## 方針

- アルゴリズムの正しさをRustの書き方より優先する。
- compiler errorや型の不整合があれば、学習者が原因を説明できる問いを先に出す。
- `clone`、iterator、関数分割などを、idiomaticという理由だけで要求しない。
- 問題がなければ無理に修正案を追加しない。

## 出力形式

各項目を `Pass`、`Question`、`Finding` のいずれかで示し、修正が必要な場合は最小の変更範囲だけを説明する。
