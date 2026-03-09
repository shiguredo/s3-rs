# s3cli にサーバサイド暗号化オプションを追加する

## 概要

cp / mv / sync サブコマンドにサーバサイド暗号化 (SSE) オプションを追加する。

## オプション

### SSE-S3 / SSE-KMS

- `--sse` — 暗号化方式を指定する (`AES256` または `aws:kms`)
- `--sse-kms-key-id` — KMS キー ID を指定する (`--sse aws:kms` と併用する)

### SSE-C (カスタマー提供キー)

- `--sse-c` — カスタマー提供キーによる暗号化方式を指定する (`AES256`)
- `--sse-c-key` — Base64 エンコードされた暗号化キーを指定する

### SSE-C コピー元 (S3→S3 コピー時)

- `--sse-c-copy-source` — コピー元の暗号化方式を指定する (`AES256`)
- `--sse-c-copy-source-key` — コピー元の Base64 エンコードされた暗号化キーを指定する

## 対応する S3 API ヘッダー

| オプション | ヘッダー |
|-----------|---------|
| `--sse AES256` | `x-amz-server-side-encryption: AES256` |
| `--sse aws:kms` | `x-amz-server-side-encryption: aws:kms` |
| `--sse-kms-key-id` | `x-amz-server-side-encryption-aws-kms-key-id` |
| `--sse-c AES256` | `x-amz-server-side-encryption-customer-algorithm: AES256` |
| `--sse-c-key` | `x-amz-server-side-encryption-customer-key` + `x-amz-server-side-encryption-customer-key-MD5` |
| `--sse-c-copy-source` | `x-amz-copy-source-server-side-encryption-customer-algorithm` |
| `--sse-c-copy-source-key` | `x-amz-copy-source-server-side-encryption-customer-key` + `...key-MD5` |

## 備考

- ライブラリ側の `PutObject` / `CopyObject` / `CreateMultipartUpload` / `UploadPart` / `GetObject` / `HeadObject` に SSE 関連ヘッダーの追加が必要
- SSE-C の場合、`GetObject` / `HeadObject` でも暗号化キーの指定が必要
- SSE-C キーの MD5 は s3cli 側で自動計算する (ライブラリは Sans I/O なので依存を増やさない)

## 解決方法

ライブラリ側:
- `PutObject`, `GetObject`, `HeadObject`, `CopyObject`, `CreateMultipartUpload`, `UploadPart` に SSE 関連フィールド、セッター、ヘッダー生成を追加
- `CopyObject` にはコピー元 SSE-C ヘッダー (`x-amz-copy-source-server-side-encryption-customer-*`) も追加

s3cli 側:
- `cp` コマンドに `--sse`, `--sse-kms-key-id`, `--sse-c`, `--sse-c-key`, `--sse-c-copy-source`, `--sse-c-copy-source-key` オプションを追加
- `SseParams` 構造体と各ビルダーへの適用メソッド (`apply_to_put`, `apply_to_get`, `apply_to_copy` 等) を実装
- SSE-C キーの MD5 を自前の MD5 実装 + Base64 で自動計算
