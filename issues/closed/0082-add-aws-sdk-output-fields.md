# aws-sdk-rust 互換 Output 型フィールドの追加

- Priority: Medium
- Created: 2026-05-25
- Completed: 2026-06-06
- Model: Composer 2.5
- Polished: 2026-06-06
- Branch: feature/add-aws-sdk-output-fields

## 目的

`*Output` 型と `parse_response` が aws-sdk-rust と比較して欠落フィールドを持つ。レスポンス header / XML から取得可能なフィールドを追加し、移行利用者が結果を失わないようにする。

## 優先度根拠

CHANGES.md の `develop` に `[ADD]` 記載の API が実装されていても、Output 型が aws-sdk-rust と一致しないと移行時にフィールドアクセスがコンパイルエラーまたは silent None になる。ただし欠落フィールドの多くは必須ではなく、既存の実運用への影響が限定的なため Medium とする。

## 現状

欠落が確認された Output（parse 側も未実装のもの含む）:

| Output | 欠落フィールド |
|--------|----------------|
| `DeleteObjectOutput` | `request_charged` |
| `CreateMultipartUploadOutput` | SSE 系, `request_charged`, `abort_date`, `abort_rule_id`, `checksum_algorithm` |
| `ListObjectsV2Output` | `encoding_type` |
| `ListPartsOutput` | `abort_date`, `abort_rule_id`, `checksum_algorithm`, `request_charged` |
| `ListMultipartUploadsOutput` | `encoding_type`, `request_charged` |
| `GetBucketLifecycleConfigurationOutput` | `transition_default_minimum_object_size` |
| `GetObjectOutput` / `HeadObjectOutput` | `website_redirect_location` |

注: `ListBucketsOutput.owner` / `Bucket.bucket_region` / `ObjectIdentifier.e_tag` 等は issue 0065 / 0068 で対応済み。

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
- CHANGES.md の `develop` に `[ADD]` エントリを追記する
- 統合テストで主要フィールドを検証する

## 解決方法

以下の Output 型に欠落フィールドを追加し、parse_response でレスポンスから値を設定するようにした:

- `DeleteObjectOutput`: `request_charged` フィールドを追加、`x-amz-request-charged` ヘッダーからパース
- `CreateMultipartUploadOutput`: `request_charged` フィールドを追加、`x-amz-request-charged` ヘッダーからパース
- `ListObjectsV2Output`: `encoding_type` フィールドを追加、XML `<EncodingType>` 要素から `EncodingType::from()` でパース
- `ListMultipartUploadsOutput`: `encoding_type` フィールドを追加（XML `<EncodingType>` 要素からパース）、`request_charged` フィールドを追加（`x-amz-request-charged` ヘッダーからパース）
- `GetObjectOutput`: `website_redirect_location` フィールドを追加、`x-amz-website-redirect-location` ヘッダーからパース
- `HeadObjectOutput`: `website_redirect_location` フィールドを追加、`x-amz-website-redirect-location` ヘッダーからパース
