# aws-sdk-rust 互換 Output の残りのフィールドを追加する

- Priority: Medium
- Created: 2026-07-31
- Completed: {YYYY-MM-DD}
- Branch: feature/add-remaining-output-fields
- Polished: {YYYY-MM-DD}
- Model: GPT-5

## 目的

既存の Output 型と parse_response に残っている非 checksum の不足フィールドを追加し、レスポンス情報を aws-sdk-rust と同じ粒度で利用できるようにする。

## 現状

過去の output 拡張 issue で主要フィールドは追加されたが、現在も以下が不足している。

- GetObjectOutput: content_range、expires_string、missing_meta、object lock 3 項目
- HeadObjectOutput: archive_status、content_range、delete_marker、expires_string、missing_meta、object lock 3 項目、tag_count
- PutObjectOutput: size、ssekms_encryption_context
- CreateBucketOutput: bucket_arn
- CreateMultipartUploadOutput: abort_date、abort_rule_id、SSE / KMS 関連、bucket_key_enabled
- UploadPartCopyOutput: CopyPartResult 以外の SSE / KMS 関連、bucket_key_enabled、request_charged
- DeleteObjectsOutput、ListObjectsV2Output、ListObjectVersionsOutput: request_charged
- ListPartsOutput: abort_date、abort_rule_id、initiator、owner、request_charged
- GetBucketLifecycleConfigurationOutput、PutBucketLifecycleConfigurationOutput: transition_default_minimum_object_size
- PutObjectLegalHoldOutput、PutObjectLockConfigurationOutput、PutObjectRetentionOutput: request_charged

checksum fields と CopyPartResult の checksum は Flexible Checksum issue で扱う。

## 設計方針

- aws-sdk-rust の operation output と同じフィールド名・型・optional semantics を採用する。
- header 由来、XML 由来、HTTP status 由来を明確に分けて parse_response に実装する。
- 欠落 header / XML 要素は None とし、無効値は既存の InvalidResponse 方針で扱う。
- 既存利用者が使うフィールドは壊さず、必要な nested model は SDK の構造に合わせる。

## 完了条件

- 上記の output fields が型定義と parse_response の両方に追加される。
- header、XML、空レスポンスを含む実レスポンスのテストが通る。
- docs/AWS_SDK_RUST.md の対応表が現状と一致する。

## AWS S3 API Reference

- GetObject: https://docs.aws.amazon.com/AmazonS3/latest/API/API_GetObject.html

> If the object is stored in Amazon S3, the response includes the object's metadata.

- HeadObject: https://docs.aws.amazon.com/AmazonS3/latest/API/API_HeadObject.html

> The HEAD operation retrieves metadata from an object without returning the object itself.

- CreateMultipartUpload: https://docs.aws.amazon.com/AmazonS3/latest/API/API_CreateMultipartUpload.html

> This action initiates a multipart upload and returns an upload ID.

- UploadPartCopy: https://docs.aws.amazon.com/AmazonS3/latest/API/API_UploadPartCopy.html

> Uploads a part by copying data from an existing object as the data source.

- ListParts: https://docs.aws.amazon.com/AmazonS3/latest/API/API_ListParts.html

> Lists the parts that have been uploaded for a specific multipart upload.
