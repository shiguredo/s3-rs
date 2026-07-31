# GetBucketPolicyStatus を追加する

- Priority: Medium
- Created: 2026-07-31
- Completed: {YYYY-MM-DD}
- Branch: feature/add-bucket-policy-status
- Polished: {YYYY-MM-DD}
- Model: GPT-5

## 目的

GetBucketPolicyStatus を追加し、bucket policy が public かどうかを aws-sdk-rust 互換に取得できるようにする。

## 現状

src/api/ に policy status 用の operation と PolicyStatus output が存在しない。

## 設計方針

- PolicyStatus の IsPublic boolean を aws-sdk-rust と同じフィールド名で公開する。
- GET /?policyStatus と expected_bucket_owner を扱う。
- true、false、要素欠落時のレスポンス処理を仕様に合わせる。

## 完了条件

- builder と output 型が追加される。
- IsPublic を XML から正しくパースできる。
- 実際の S3 互換サーバー、または実レスポンスを使う Sans I/O テストが通る。

## AWS S3 API Reference

- GetBucketPolicyStatus: https://docs.aws.amazon.com/AmazonS3/latest/API/API_GetBucketPolicyStatus.html

> Retrieves the policy status for an Amazon S3 bucket, indicating whether the bucket is public.
