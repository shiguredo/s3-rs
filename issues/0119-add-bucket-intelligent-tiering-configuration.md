# Bucket Intelligent-Tiering Configuration API を追加する

- Priority: Low
- Created: 2026-07-31
- Completed: {YYYY-MM-DD}
- Branch: feature/add-bucket-intelligent-tiering-configuration
- Polished: {YYYY-MM-DD}
- Model: GPT-5

## 目的

Intelligent-Tiering Configuration の取得、設定、削除、一覧取得を追加し、aws-sdk-rust の S3 API surface を補完する。

## 現状

GetBucketIntelligentTieringConfiguration、PutBucketIntelligentTieringConfiguration、DeleteBucketIntelligentTieringConfiguration、ListBucketIntelligentTieringConfigurations が未実装である。

## 設計方針

- IntelligentTieringConfiguration、Tiering、Filter などを aws-sdk-rust の型に合わせる。
- configuration id、status、tiering、filter、continuation token を仕様どおりに扱う。
- XML の未知要素と複数 tiering を既存のパース方針で処理する。

## 完了条件

- 4 operation の builder、XML、output が追加される。
- 一覧取得の pagination token を扱える。
- 実際の S3 互換サーバー、または実レスポンスを使う Sans I/O テストが通る。

## AWS S3 API Reference

- GetBucketIntelligentTieringConfiguration: https://docs.aws.amazon.com/AmazonS3/latest/API/API_GetBucketIntelligentTieringConfiguration.html

> Returns the configuration for an Amazon S3 bucket's intelligent-tiering configuration.

- PutBucketIntelligentTieringConfiguration: https://docs.aws.amazon.com/AmazonS3/latest/API/API_PutBucketIntelligentTieringConfiguration.html

> Puts a new configuration in place or updates an existing configuration for Amazon S3 Intelligent-Tiering.

- DeleteBucketIntelligentTieringConfiguration: https://docs.aws.amazon.com/AmazonS3/latest/API/API_DeleteBucketIntelligentTieringConfiguration.html

> Deletes the specified configuration from the bucket.

- ListBucketIntelligentTieringConfigurations: https://docs.aws.amazon.com/AmazonS3/latest/API/API_ListBucketIntelligentTieringConfigurations.html

> Lists the S3 Intelligent-Tiering configurations for a bucket.
