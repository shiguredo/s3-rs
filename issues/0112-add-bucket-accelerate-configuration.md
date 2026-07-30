# Bucket Accelerate Configuration API を追加する

- Priority: Medium
- Created: 2026-07-31
- Completed: {YYYY-MM-DD}
- Branch: feature/add-bucket-accelerate-configuration
- Polished: {YYYY-MM-DD}
- Model: GPT-5

## 目的

GetBucketAccelerateConfiguration と PutBucketAccelerateConfiguration を追加し、Transfer Acceleration の設定を aws-sdk-rust 互換に扱えるようにする。

## 現状

src/api/ に accelerate サブリソース用の operation と AccelerateConfiguration の XML 処理が存在しない。

## 設計方針

- Status の Enabled / Suspended を aws-sdk-rust の型・フィールド名に合わせる。
- GET のレスポンス XML と PUT の XML body を実装する。
- PUT の checksum ヘッダーと expected_bucket_owner は既存の共通方針に従う。
- directory bucket では利用できない仕様を API ドキュメントとテストに反映する。

## 完了条件

- 2 operation の URI、XML、レスポンス処理が実装される。
- Enabled、Suspended、未指定の各状態を検証するテストが通る。
- 実際の S3 互換サーバーで利用可能な範囲の統合テストが通る。

## AWS S3 API Reference

- GetBucketAccelerateConfiguration: https://docs.aws.amazon.com/AmazonS3/latest/API/API_GetBucketAccelerateConfiguration.html

> Returns the Transfer Acceleration state of a bucket.

- PutBucketAccelerateConfiguration: https://docs.aws.amazon.com/AmazonS3/latest/API/API_PutBucketAccelerateConfiguration.html

> Sets the accelerate configuration of an existing bucket.
