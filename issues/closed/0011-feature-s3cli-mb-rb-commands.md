# s3cli に mb / rb サブコマンドを追加する

## 概要

バケットの作成 (`mb`) と削除 (`rb`) サブコマンドを s3cli に追加する。

## mb (make bucket)

```
s3cli mb s3://<BucketName>
```

- `CreateBucket` API を使用する

## rb (remove bucket)

```
s3cli rb s3://<BucketName> [--force]
```

- `DeleteBucket` API を使用する
- `--force` — バケット内の全オブジェクトを削除してからバケットを削除する（内部的に `rm --recursive` 相当を実行する）

## 備考

- ライブラリ側に `CreateBucket` / `DeleteBucket` API は既に存在する
- 実装は単純

## 解決方法

- `cmd_mb` 関数を追加し、`CreateBucket` API を呼び出す
- `cmd_rb` 関数を追加し、`DeleteBucket` API を呼び出す
- `--force` オプション付きで `rb` を実行すると、`delete_recursive` で全オブジェクトを削除してからバケットを削除する
- サブコマンドディスパッチに `mb` / `rb` を追加した
