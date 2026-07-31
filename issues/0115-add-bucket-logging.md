# Bucket Logging API を追加する

- Priority: Medium
- Created: 2026-07-31
- Completed: {YYYY-MM-DD}
- Branch: feature/add-bucket-logging
- Polished: {YYYY-MM-DD}
- Model: GPT-5

## 目的

GetBucketLogging と PutBucketLogging を追加し、server access logging の設定を aws-sdk-rust 互換に扱えるようにする。

## 現状

src/api/ に logging サブリソース用の operation と BucketLoggingStatus、LoggingEnabled の XML モデルが存在しない。

## 設計方針

- aws-sdk-rust の BucketLoggingStatus、LoggingEnabled、TargetGrant、Grantee、LoggingPermission に合わせる。
- logging の有効化、無効化、target bucket、target prefix、target grants を XML で扱う。
- Content-MD5、expected_bucket_owner、checksum の扱いを公式 API 仕様と既存 XML API に合わせる。

## 完了条件

- GET /?logging と PUT /?logging を構築できる。
- 空の状態を logging 無効として正しくパースできる。
- 実際の S3 互換サーバーを使う統合テストが通る。

## AWS S3 API Reference

- GetBucketLogging: https://docs.aws.amazon.com/AmazonS3/latest/API/API_GetBucketLogging.html

> Returns the logging status of a bucket and the target bucket and prefix for the logs.

- PutBucketLogging: https://docs.aws.amazon.com/AmazonS3/latest/API/API_PutBucketLogging.html

> Set the logging parameters for a bucket and to specify permissions for who can view and modify the logs.
