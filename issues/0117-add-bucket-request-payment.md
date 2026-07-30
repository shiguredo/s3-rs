# Bucket Request Payment API を追加する

- Priority: Medium
- Created: 2026-07-31
- Completed: {YYYY-MM-DD}
- Branch: feature/add-bucket-request-payment
- Polished: {YYYY-MM-DD}
- Model: GPT-5

## 目的

GetBucketRequestPayment と PutBucketRequestPayment を追加し、Requester Pays bucket の設定を aws-sdk-rust 互換に扱えるようにする。

## 現状

src/api/ に requestPayment サブリソース用の operation、Payer 型、XML 入出力処理が存在しない。

## 設計方針

- aws-sdk-rust の Payer と output 型に合わせる。
- BucketPayer の Requester / BucketOwner を XML で扱う。
- 共通の request_payer header 対応とは分け、bucket の RequestPaymentConfiguration を実装する。

## 完了条件

- GET /?requestPayment と PUT /?requestPayment を構築できる。
- Requester と BucketOwner を相互にパース・シリアライズできる。
- 実際の S3 互換サーバーで利用可能な範囲の統合テストが通る。

## AWS S3 API Reference

- GetBucketRequestPayment: https://docs.aws.amazon.com/AmazonS3/latest/API/API_GetBucketRequestPayment.html

> Returns the request payment configuration of a bucket.

- PutBucketRequestPayment: https://docs.aws.amazon.com/AmazonS3/latest/API/API_PutBucketRequestPayment.html

> Sets the request payment configuration of a bucket.
