# aws-sdk-rust 互換 Output 型フィールドの追加

- Priority: High
- Created: 2026-05-25
- Model: Composer 2.5
- Polished: 2026-05-31

## 目的

`*Output` 型と `parse_response` が aws-sdk-rust と比較して欠落フィールドを持つ。レスポンス header / XML から取得可能なフィールドを追加し、移行利用者が結果を失わないようにする。

## 優先度根拠

CHANGES.md ## develop に `[ADD]` 記載の API が実装されていても、Output 型が aws-sdk-rust と一致しないと移行時にフィールドアクセスがコンパイルエラーまたは silent None になる。

## 現状

欠落が確認された Output（parse 側も未実装のもの含む）:

| Output | 欠落フィールド |
|--------|----------------|
| `DeleteObjectOutput` | `request_charged` |
| `CreateMultipartUploadOutput` | SSE 系, `request_charged`, `abort_date`, `abort_rule_id`, `checksum_algorithm` 等 |
| `ListObjectsV2Output` | `encoding_type` |
| `ListPartsOutput` | `abort_date`, `abort_rule_id`, `checksum_algorithm`, `request_charged` |
| `ListMultipartUploadsOutput` | `encoding_type`, `request_charged` |
| `GetBucketLifecycleConfigurationOutput` | `transition_default_minimum_object_size` |
| `GetObjectOutput` / `HeadObjectOutput` | `website_redirect_location` |

**注**: `ListBucketsOutput.owner` / `Bucket.bucket_region` / `ObjectIdentifier.e_tag` 等は issue 0065 / 0068 で意図的に追加済み。**削除しない**。

## 設計方針

- aws-sdk-rust の `aws_sdk_s3::operation::*::Output` を一次参照
- header 由来は `parse_response` で `get_header` 追加
- XML 由来は `extract_element` またはスコープ付きパース
- `docs/AWS_SDK_RUST.md` の「出力のみ対応」方針と整合

## AWS S3 API Reference

- DeleteObject: <https://docs.aws.amazon.com/AmazonS3/latest/API/API_DeleteObject.html>
- CreateMultipartUpload: <https://docs.aws.amazon.com/AmazonS3/latest/API/API_CreateMultipartUpload.html>
- ListObjectsV2: <https://docs.aws.amazon.com/AmazonS3/latest/API/API_ListObjectsV2.html>

> x-amz-request-charged: If present, indicates that the requester was successfully charged for the request.

## 完了条件

- 上記 Output 型にフィールドが追加される
- `parse_response` が実レスポンスから値を設定する
- CHANGES.md ## develop に `[ADD]` エントリを追記
- 統合テストで主要フィールドを検証

## 解決方法

1. `types.rs` にフィールド追加
2. 各 `parse_response` 拡張
3. MinIO / RustFS 統合テスト追加
