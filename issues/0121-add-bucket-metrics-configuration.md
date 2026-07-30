# Bucket Metrics Configuration API を追加する

- Priority: Low
- Created: 2026-07-31
- Completed: {YYYY-MM-DD}
- Branch: feature/add-bucket-metrics-configuration
- Polished: {YYYY-MM-DD}
- Model: GPT-5

## 目的

Bucket Metrics Configuration の取得、設定、削除、一覧取得を追加し、aws-sdk-rust の S3 API surface を補完する。

## 現状

GetBucketMetricsConfiguration、PutBucketMetricsConfiguration、DeleteBucketMetricsConfiguration、ListBucketMetricsConfigurations が未実装である。

## 設計方針

- MetricsConfiguration、Filter、TagFilter、And などを aws-sdk-rust の型に合わせる。
- id、filter、prefix、tag、continuation token、truncated を仕様どおりに扱う。
- XML の nested filter を既存の XML パース方針で処理する。

## 完了条件

- 4 operation の builder、XML、output が追加される。
- 一覧取得の pagination と各 filter 形式を検証できる。
- 実際の S3 互換サーバー、または実レスポンスを使う Sans I/O テストが通る。

## AWS S3 API Reference

- GetBucketMetricsConfiguration: https://docs.aws.amazon.com/AmazonS3/latest/API/API_GetBucketMetricsConfiguration.html

> Returns a metrics configuration for the specified bucket.

- PutBucketMetricsConfiguration: https://docs.aws.amazon.com/AmazonS3/latest/API/API_PutBucketMetricsConfiguration.html

> Sets a metrics configuration for the CloudWatch request metrics from an Amazon S3 bucket.

- DeleteBucketMetricsConfiguration: https://docs.aws.amazon.com/AmazonS3/latest/API/API_DeleteBucketMetricsConfiguration.html

> Deletes a metrics configuration from the CloudWatch request metrics for a bucket.

- ListBucketMetricsConfigurations: https://docs.aws.amazon.com/AmazonS3/latest/API/API_ListBucketMetricsConfigurations.html

> Lists the metrics configurations for the bucket.
