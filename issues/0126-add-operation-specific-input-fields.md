# operation 固有の入力項目を aws-sdk-rust 互換にする

- Priority: High
- Created: 2026-07-31
- Completed: {YYYY-MM-DD}
- Branch: feature/add-operation-specific-input-fields
- Polished: {YYYY-MM-DD}
- Model: GPT-5

## 目的

既存 operation の builder に不足している operation 固有の入力項目を追加し、aws-sdk-rust からの移行時に主要な request builder がそのまま対応できるようにする。

## 現状

expected_bucket_owner と request_payer は既存の共通対応 issue、checksum 個別項目は別の checksum issue、HeadObject の response_* は専用 issue で扱う。それ以外にも以下の入力項目が不足している。

- PutObject: bucket_key_enabled、grant_*、object lock 3 項目、sse_customer_key_md5、ssekms_encryption_context、website_redirect_location、write_offset_bytes、content_md5
- CopyObject: annotation_directive、bucket_key_enabled、copy_source_sse_customer_key_md5、grant_*、if_match、if_none_match、object lock 3 項目、sse_customer_key_md5、ssekms_encryption_context、website_redirect_location
- CreateBucket: bucket_namespace、grant_*、object_lock_enabled_for_bucket、object_ownership
- CreateMultipartUpload: bucket_key_enabled、checksum_type、grant_*、object lock 3 項目、sse_customer_key_md5、ssekms_encryption_context、website_redirect_location
- DeleteObject: if_match、mfa、bypass_governance_retention、if_match_last_modified_time、if_match_size
- DeleteObjects: mfa、bypass_governance_retention
- UploadPart: content_md5、sse_customer_key_md5
- UploadPartCopy: copy_source_sse_customer_key_md5、sse_customer_key_md5
- CompleteMultipartUpload: checksum_type、mpu_object_size、sse_customer_key_md5、top-level checksum 以外の不足項目
- AbortMultipartUpload: if_match_initiated_time
- ListParts: sse_customer_key_md5
- ListObjectVersions: optional_object_attributes
- ListObjectsV2: fetch_owner、optional_object_attributes
- PutBucketOwnershipControls: checksum_algorithm、content_md5
- PutBucketWebsite: checksum_algorithm、content_md5
- PutObjectLegalHold: checksum_algorithm、content_md5
- PutObjectLockConfiguration: content_md5
- PutObjectRetention: checksum_algorithm、content_md5
- PutObjectTagging: content_md5
- PutPublicAccessBlock: content_md5

## 設計方針

- aws-sdk-rust の operation input と一項目ずつ照合し、名前・型・setter・HTTP への反映を一致させる。
- 既存の convenience method は維持し、Option を直接受ける set_* 追加は別 issue の方針に従う。
- Content-MD5 を自動計算する既存仕様は維持しつつ、明示指定が必要な API では setter と header 反映を追加する。
- object lock、grant、conditional header の優先順位を AWS API Reference と SDK 実装で検証する。

## 完了条件

- 上記の入力項目が対応する builder に追加される。
- build_request と presigned の両方で必要な header、query、XML に反映される。
- 実際の S3 互換サーバーを使う request 検証と統合テストが通る。

## AWS S3 API Reference

- PutObject: https://docs.aws.amazon.com/AmazonS3/latest/API/API_PutObject.html

> You can use the PutObject action to add an object to a bucket.

- CopyObject: https://docs.aws.amazon.com/AmazonS3/latest/API/API_CopyObject.html

> Creates a copy of an object that is already stored in Amazon S3.

- DeleteObject: https://docs.aws.amazon.com/AmazonS3/latest/API/API_DeleteObject.html

> Removes the null version (if there is one) of an object and inserts a delete marker.

- CompleteMultipartUpload: https://docs.aws.amazon.com/AmazonS3/latest/API/API_CompleteMultipartUpload.html

> Completes a multipart upload by assembling previously uploaded parts.

- ListObjectsV2: https://docs.aws.amazon.com/AmazonS3/latest/API/API_ListObjectsV2.html

> Returns some or all (up to 1,000) of the objects in a bucket.
