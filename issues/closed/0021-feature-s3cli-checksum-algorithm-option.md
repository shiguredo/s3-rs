# s3cli に --checksum-algorithm オプションを追加する

## 概要

cp / mv / sync サブコマンドに `--checksum-algorithm` オプションを追加する。

## 対応するアルゴリズム

- `CRC32`
- `CRC32C`
- `CRC64NVME`
- `SHA1`
- `SHA256`

## 対応する S3 API ヘッダー

- `x-amz-checksum-algorithm` — リクエスト時にアルゴリズムを指定する
- `x-amz-checksum-crc32` / `x-amz-checksum-sha256` 等 — 計算済みチェックサム値を送信する

## 備考

- ライブラリ側の `PutObject` / `UploadPart` / `CompleteMultipartUpload` にチェックサム関連ヘッダーの追加が必要
- クライアント側でチェックサムを計算して送信する
- MinIO 等の S3 互換サービスでも対応している

## 解決方法

- ライブラリ側の `PutObject` と `UploadPart` に `checksum_algorithm` / `checksum_value` フィールドを追加した
- アルゴリズムに応じた `x-amz-checksum-*` ヘッダーを自動的に設定する
