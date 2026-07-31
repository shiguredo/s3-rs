# Bucket ABAC API を追加する

- Priority: Medium
- Created: 2026-07-31
- Completed: {YYYY-MM-DD}
- Branch: feature/add-bucket-abac
- Polished: {YYYY-MM-DD}
- Model: GPT-5

## 目的

aws-sdk-rust に存在する GetBucketAbac と PutBucketAbac を追加し、バケットの属性ベースアクセス制御を S3 API 互換に扱えるようにする。

## 現状

src/api/ に ABAC 用の operation、AbacStatus のモデル、XML リクエスト・レスポンス処理が存在しない。

## 設計方針

- GetBucketAbac と PutBucketAbac の builder を追加する。
- AbacStatus の XML 構造と aws-sdk-rust の型を反映する。
- PutBucketAbac の Content-MD5 と x-amz-sdk-checksum-algorithm を既存の XML API と同じ方針で処理する。
- expected_bucket_owner は既存の共通対応 issue と重複させず、その共通実装を利用する。

## 完了条件

- 2 operation のリクエストを正しい URI、ヘッダー、XML で構築できる。
- Enabled と Disabled のレスポンスを正しくパースできる。
- 実際の S3 互換サーバー、または実レスポンスを用いた Sans I/O テストで検証できる。

## AWS S3 API Reference

- GetBucketAbac: https://docs.aws.amazon.com/AmazonS3/latest/API/API_GetBucketAbac.html

> Returns the attribute-based access control (ABAC) property of the general purpose bucket.

- PutBucketAbac: https://docs.aws.amazon.com/AmazonS3/latest/API/API_PutBucketAbac.html

> Sets the attribute-based access control (ABAC) property of the general purpose bucket.
