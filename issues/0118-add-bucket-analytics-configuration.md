# Bucket Analytics Configuration API を追加する

- Priority: Low
- Created: 2026-07-31
- Completed: {YYYY-MM-DD}
- Branch: feature/add-bucket-analytics-configuration
- Polished: {YYYY-MM-DD}
- Model: GPT-5

## 目的

Bucket Analytics Configuration の取得、設定、削除、一覧取得を追加し、aws-sdk-rust の S3 API surface を補完する。

## 現状

GetBucketAnalyticsConfiguration、PutBucketAnalyticsConfiguration、DeleteBucketAnalyticsConfiguration、ListBucketAnalyticsConfigurations が未実装で、AnalyticsConfiguration の XML モデルも存在しない。

## 設計方針

- Id、Filter、StorageClassAnalysis、DataExport、S3BucketDestination などを aws-sdk-rust の型に合わせる。
- Get / Put / Delete / List の query、path、XML、pagination token をそれぞれ実装する。
- unknown XML 要素を既存の non-exhaustive 方針で安全に扱う。

## 完了条件

- 4 operation が client から利用できる。
- 複数設定の一覧と continuation token を扱える。
- 実際の S3 互換サーバー、または実レスポンスを使う Sans I/O テストが通る。

## AWS S3 API Reference

- GetBucketAnalyticsConfiguration: https://docs.aws.amazon.com/AmazonS3/latest/API/API_GetBucketAnalyticsConfiguration.html

> Returns the configuration of the analytics filter, data export, and storage class analysis for the specified bucket.

- PutBucketAnalyticsConfiguration: https://docs.aws.amazon.com/AmazonS3/latest/API/API_PutBucketAnalyticsConfiguration.html

> Sets an analytics configuration for the bucket.

- DeleteBucketAnalyticsConfiguration: https://docs.aws.amazon.com/AmazonS3/latest/API/API_DeleteBucketAnalyticsConfiguration.html

> Deletes an analytics configuration for the bucket.

- ListBucketAnalyticsConfigurations: https://docs.aws.amazon.com/AmazonS3/latest/API/API_ListBucketAnalyticsConfigurations.html

> Lists the analytics configurations for the bucket.
