# s3cli に --acl オプションを追加する

## 概要

cp / mv / sync サブコマンドに `--acl` オプションを追加する。

## 対応する ACL

- `private`
- `public-read`
- `public-read-write`
- `authenticated-read`
- `aws-exec-read`
- `bucket-owner-read`
- `bucket-owner-full-control`

## 備考

- ライブラリ側の `PutObject` / `CopyObject` / `CreateMultipartUpload` に `x-amz-acl` ヘッダーの追加が必要

## 解決方法

ライブラリ側の `PutObject`, `CopyObject`, `CreateMultipartUpload` に `acl` フィールド・セッター・`x-amz-acl` ヘッダー生成を追加。s3cli の `cp` コマンドに `--acl` オプションを追加し、`UploadParams` 経由で各ビルダーに適用。
