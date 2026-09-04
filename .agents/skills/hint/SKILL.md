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

## セッション状態

問題ごとのHint進行状況は `learning/<problem-id>/session.yaml` に保存する。`cph init <problem-id>` が次の初期ファイルを生成する。

```yaml
hint_level: 0
solution_revealed: false
```

このファイルは次の固定2項目だけで構成する。

- `hint_level`: 開示済みの最高Level。10進整数 `0`〜`6`。
- `solution_revealed`: Level 6のRust実装を開示済みなら `true`、それ以外は `false`。

キーはそれぞれ1回だけ現れなければならない。必須項目の欠損、重複、未知の項目、整数・真偽値以外の型、範囲外のLevel、`hint_level: 6`と`solution_revealed: false`の組み合わせは不正状態として扱う。`solution_revealed: true`もLevel 6の開示時だけ許可し、`hint_level: 6`と組み合わせる。不正状態ではHintを提示せず、既存ファイルを変更しない。

セッションファイル全体が存在しない場合だけ、`hint_level: 0`かつ `solution_revealed: false` の新規セッションとして扱う。ファイルが存在するのに読み込めない場合はI/Oエラーとして停止する。履歴、日時、問題文、解答本文などは保存しない。

Hintを提示・更新する順序は次のとおりとする。

1. `session.yaml` を読み込み、存在しなければ初期状態を使い、存在すれば固定形式を検証する。
2. 既存のHint Ladderルールに従い、明示的な要求、前提確認、最大1段階の進行を判断する。Level 6へ自動的に進めず、Level 6の要求時はLevel 5までの理解を確認する。
3. Level 1〜5のHintを実際に開示できた場合だけ、そのLevelへ更新する。Level 6の解答を開示できた場合だけ `hint_level: 6` と `solution_revealed: true` に更新する。質問や前提確認だけでは更新しない。

状態の更新時は、新しい2行を同じディレクトリの一時ファイルへ最後まで書き込んでから `session.yaml` と置き換える。書き込みまたは置き換えに失敗した場合は一時ファイルだけを削除し、既存の `session.yaml` を上書きしない。`solution_revealed: true` の状態では追加の段階的Hintを提示せず、状態も変更しない。

## 受入確認

- 初期化: `cph init <id>` 後に `learning/<id>/session.yaml` が上記の初期値で存在する。
- 更新・再読込: Level 1、Level 2の開示後にそれぞれ `hint_level: 1`、`hint_level: 2` となり、次の応答で読み直しても値が維持される。Level 6の解答開示後は `6/true` となる。
- 不正状態: 欠損、重複、未知項目、型不正、範囲外、`6/false`、`true`とLevel 6の不一致を検出したら、Hintを出さずファイルを変更しない。
- 既存workspace保護: 既存の `learning/<id>/session.yaml` を含むworkspaceで `cph init <id>` を再実行しても、エラーになり、既存ファイルの内容が変わらない。
