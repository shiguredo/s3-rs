# Bucket Inventory Configuration API を追加する

- Priority: Low
- Created: 2026-07-31
- Completed: {YYYY-MM-DD}
- Branch: feature/add-bucket-inventory-configuration
- Polished: {YYYY-MM-DD}
- Model: GPT-5

## 目的

Bucket Inventory Configuration の取得、設定、削除、一覧取得を追加し、aws-sdk-rust の S3 API surface を補完する。

## 現状

GetBucketInventoryConfiguration、PutBucketInventoryConfiguration、DeleteBucketInventoryConfiguration、ListBucketInventoryConfigurations が未実装である。

## 設計方針

- InventoryConfiguration、Destination、InventoryS3BucketDestination、Schedule などを aws-sdk-rust の型に合わせる。
- id、filter、included object versions、optional fields、schedule、destination を XML で扱う。
- 一覧取得の continuation token と truncated を正しく処理する。

## 完了条件

- 4 operation の builder、XML、output が追加される。
- 複数設定と pagination を検証できる。
- 実際の S3 互換サーバー、または実レスポンスを使う Sans I/O テストが通る。

## AWS S3 API Reference

- GetBucketInventoryConfiguration: https://docs.aws.amazon.com/AmazonS3/latest/API/API_GetBucketInventoryConfiguration.html

> Returns an inventory configuration for the specified bucket.

- PutBucketInventoryConfiguration: https://docs.aws.amazon.com/AmazonS3/latest/API/API_PutBucketInventoryConfiguration.html

> This implementation of the PUT action adds an inventory configuration to the specified bucket.

- DeleteBucketInventoryConfiguration: https://docs.aws.amazon.com/AmazonS3/latest/API/API_DeleteBucketInventoryConfiguration.html

> Deletes an inventory configuration from the bucket.

- ListBucketInventoryConfigurations: https://docs.aws.amazon.com/AmazonS3/latest/API/API_ListBucketInventoryConfigurations.html

> Returns a list of inventory configurations for the specified bucket.
