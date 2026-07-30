# GetBucketLocation を追加する

- Priority: Medium
- Created: 2026-07-31
- Completed: {YYYY-MM-DD}
- Branch: feature/add-bucket-location
- Polished: {YYYY-MM-DD}
- Model: GPT-5

## 目的

GetBucketLocation を追加し、bucket の region / location constraint を aws-sdk-rust 互換に取得できるようにする。

## 現状

src/api/ に GetBucketLocation がなく、LocationConstraint のレスポンスを表す型も存在しない。

## 設計方針

- aws-sdk-rust の GetBucketLocationOutput と BucketLocationConstraint に合わせる。
- 空の XML、us-east-1、リージョン名、古い EU 値を仕様どおりに扱う。
- expected_bucket_owner は共通対応 issue の実装を利用する。

## 完了条件

- GET /?location のリクエストを構築できる。
- XML の LocationConstraint を正しい型に変換できる。
- 実際の S3 互換サーバーでレスポンスを検証する統合テストが通る。

## AWS S3 API Reference

- GetBucketLocation: https://docs.aws.amazon.com/AmazonS3/latest/API/API_GetBucketLocation.html

> Returns the Region the bucket resides in.
