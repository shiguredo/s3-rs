# s3cli ls でバケット一覧表示に対応する

## 概要

`s3cli ls` を引数なしで実行した場合にバケット一覧を表示する。

## 動作

```bash
# バケット一覧
s3cli ls

# 特定バケット内のオブジェクト一覧（既存機能）
s3cli ls s3://bucket/prefix
```

## 出力形式

aws s3 ls と同様:

```
2024-01-15 09:30:00 my-bucket
2024-02-20 14:15:30 another-bucket
```

## 備考

- ライブラリ側に `ListBuckets` API は既に存在する
- ls の引数を必須からオプショナルに変更する必要がある

## 解決方法

s3cli の `cmd_ls` で `<S3URI>` 引数をオプショナルに変更し、引数なしの場合は `ListBuckets` API を呼んでバケット一覧を表示するようにした。ページネーション (continuation_token) にも対応。
