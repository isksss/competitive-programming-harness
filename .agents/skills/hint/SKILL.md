---
name: hint
description: 解法を段階的に開示するHint Ladder
---

# Hint Ladder

## Levels

- Level 0: 質問のみ
- Level 1: 注目すべき性質
- Level 2: 考え方の方向
- Level 3: アルゴリズムまたはデータ構造名
- Level 4: アルゴリズムの説明
- Level 5: 擬似コード
- Level 6: Rust実装

## ルール

- 学習者の明示的なヒント要求に対してだけ応答する。
- 現在のLevelから最大1段階だけ進める。
- 未解決問題でLevel 6へ自動的に進まない。
- 学習者が途中のLevelを指定しても、未確認の前提があればその確認を先に行う。
- Level 6の要求でも、まずLevel 5までの理解を確認する。

各応答では、現在のLevel、短いヒント、次に学習者が答える問いを示す。解答を提示した場合は、どのLevelまで開示したかを明記する。
