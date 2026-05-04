# docs/AWS_SDK_RUST.md の対応方針を再分類し主要 Output に取得系フィールドを追加する

Created: 2026-05-04
Model: Opus 4.7

## 根拠

- `docs/AWS_SDK_RUST.md` には現状「対応予定無し」として 19 種のパラメータが列挙されている (例: `expected_bucket_owner`, `request_payer`, `grant_*`, `object_lock_*`, `bucket_key_enabled`, `ssekms_encryption_context`, `website_redirect_location`, `mfa`, `bypass_governance_retention` 等)。
- 一方、`AGENTS.md` および `CLAUDE.md` には「Amazon S3 API の仕様と aws-sdk-rust との互換性を最優先にする」と明記されており、`docs/AWS_SDK_RUST.md` の「対応予定無し」表記と矛盾している。
- 「対応予定無し」のうち、互換性のためにヘッダー渡しのみで実装が軽いもの (`expected_bucket_owner`, `request_payer`, `grant_*`, `website_redirect_location` 等) と、機能本格対応が必要なもの (`object_lock_*`, `mfa`, `bypass_governance_retention` 等) を **再分類** する必要がある。
- 既に issue 0057 (`pending/0057-feature-expected-bucket-owner-request-payer.md`) で `expected_bucket_owner` / `request_payer` の取り扱いが pending になっており、本 issue で方針整理した上で `pending/` から `issues/` に戻す。
- 出力 (`*Output`) のうち、SSE/checksum/storage_class 等の取得系フィールドは S3 互換ストレージでも返却されるため、利用者が結果を確認できるよう追加する必要がある。

## 変更内容

### 1. `docs/AWS_SDK_RUST.md` の方針再分類

「対応予定無し」を以下に再分類する。

#### 「未対応 (互換性のため対応予定)」に格上げ

ヘッダー渡しまたは値の追加のみで実装が軽量なもの:

| パラメータ | 該当 API | 理由 |
|---|---|---|
| `expected_bucket_owner` | 全 API | issue 0057 で対応 |
| `request_payer` | 全 API | issue 0057 で対応、enum 化は issue 0059 後続 |
| `website_redirect_location` | PutObject, CopyObject, CreateMultipartUpload | `x-amz-website-redirect-location` ヘッダー追加のみ |
| `grant_full_control` / `grant_read` / `grant_read_acp` / `grant_write` / `grant_write_acp` | PutObject, CopyObject, CreateBucket, CreateMultipartUpload | レガシー ACL だが API は単純 (文字列ヘッダー) |

#### 「対応予定無し」維持

機能の本格対応とセットになるため保留:

| パラメータ | 理由 |
|---|---|
| `object_lock_legal_hold_status`, `object_lock_mode`, `object_lock_retain_until_date` | Object Lock 機能本格対応とセット (別 issue) |
| `object_lock_enabled_for_bucket` | 同上 |
| `mfa`, `bypass_governance_retention` | Object Lock / Versioning MFA 機能とセット |
| `if_match_initiated_time`, `if_match_last_modified_time`, `if_match_size` | S3 固有の条件付き機能、AWS SDK でも限定 |
| `mpu_object_size`, `write_offset_bytes` | S3 Express One Zone 専用機能 |
| `bucket_key_enabled`, `ssekms_encryption_context` | KMS 詳細機能、AWS 環境前提 |
| `optional_object_attributes`, `fetch_owner` | AWS IAM ベース、S3 互換ストレージで意味が薄い |
| `confirm_remove_self_bucket_access` | AWS 固有の安全装置 |
| `object_ownership` | バケットオーナーシップコントロール (別 issue) |
| `checksum_type` | issue 0059 後続で型化検討 |

### 2. 出力フィールド補完

以下を該当 `*Output` に追加する。XML/レスポンスヘッダーのパース処理も併せて実装する。

#### `GetObjectOutput` / `HeadObjectOutput` (`src/types.rs:144-189`)

追加:
- `content_encoding: Option<String>`
- `content_disposition: Option<String>`
- `content_language: Option<String>`
- `cache_control: Option<String>`
- `expires: Option<String>` (出力日時は文字列のまま、issue 0060 方針)
- `storage_class: Option<StorageClass>` (issue 0059 で型化)
- `parts_count: Option<i32>`
- `accept_ranges: Option<String>`
- `delete_marker: Option<bool>` (`GetObjectOutput` のみ)
- `replication_status: Option<String>` (S3 互換ストレージで意味が薄い場合 None になるが、aws-sdk-rust 互換のため定義)
- `restore: Option<String>`, `expiration: Option<String>`
- `server_side_encryption: Option<ServerSideEncryption>`
- `sse_customer_algorithm: Option<String>`
- `sse_customer_key_md5: Option<String>`
- `ssekms_key_id: Option<String>`
- `bucket_key_enabled: Option<bool>`
- `request_charged: Option<String>` (issue 0059 後続で `RequestCharged` 型化)
- `tag_count: Option<i32>` (`GetObjectOutput` のみ)

#### `PutObjectOutput` (`src/types.rs:191-197`)

追加:
- `expiration: Option<String>`
- `server_side_encryption: Option<ServerSideEncryption>`
- `sse_customer_algorithm: Option<String>`
- `sse_customer_key_md5: Option<String>`
- `ssekms_key_id: Option<String>`
- `bucket_key_enabled: Option<bool>`
- `request_charged: Option<String>`
- `checksum_crc32: Option<String>`
- `checksum_crc32_c: Option<String>`
- `checksum_crc64_nvme: Option<String>`
- `checksum_sha1: Option<String>`
- `checksum_sha256: Option<String>`
- `checksum_type: Option<String>`

#### `CompleteMultipartUploadOutput` (`src/types.rs:229-238`)

追加:
- `expiration: Option<String>`
- `server_side_encryption: Option<ServerSideEncryption>`
- `ssekms_key_id: Option<String>`
- `bucket_key_enabled: Option<bool>`
- `request_charged: Option<String>`
- `checksum_crc32: Option<String>`
- `checksum_crc32_c: Option<String>`
- `checksum_crc64_nvme: Option<String>`
- `checksum_sha1: Option<String>`
- `checksum_sha256: Option<String>`
- `checksum_type: Option<String>`

#### `UploadPartOutput` (`src/types.rs:214-218`)

追加:
- `server_side_encryption: Option<ServerSideEncryption>`
- `sse_customer_algorithm: Option<String>`
- `sse_customer_key_md5: Option<String>`
- `ssekms_key_id: Option<String>`
- `bucket_key_enabled: Option<bool>`
- `request_charged: Option<String>`
- `checksum_crc32: Option<String>`
- `checksum_crc32_c: Option<String>`
- `checksum_crc64_nvme: Option<String>`
- `checksum_sha1: Option<String>`
- `checksum_sha256: Option<String>`

#### `CopyObjectOutput` (issue 0063 で新設したもの)

トップレベルに追加:
- `expiration: Option<String>`
- `server_side_encryption: Option<ServerSideEncryption>`
- `sse_customer_algorithm: Option<String>`
- `sse_customer_key_md5: Option<String>`
- `ssekms_key_id: Option<String>`
- `ssekms_encryption_context: Option<String>`
- `bucket_key_enabled: Option<bool>`
- `request_charged: Option<String>`

#### `Object` / `ObjectVersion` (`src/types.rs:354-362, 333-343`)

追加:
- `owner: Option<Owner>` (`Owner { display_name, id }` 型を新設)
- `restore_status: Option<RestoreStatus>` (`RestoreStatus { is_restore_in_progress: bool, restore_expiry_date: Option<String> }` 型を新設)
- `checksum_algorithm: Option<Vec<ChecksumAlgorithm>>` (issue 0059)
- `checksum_type: Option<String>`

#### `ObjectIdentifier` (`src/types.rs:280-284`)

追加 (S3 Conditional Delete 対応):
- `e_tag: Option<String>`
- `last_modified_time: Option<String>` (出力日時は文字列のまま)
- `size: Option<i64>`

#### `CompletedPart` (`src/types.rs:286-291`)

追加:
- `checksum_crc32: Option<String>`
- `checksum_crc32_c: Option<String>`
- `checksum_crc64_nvme: Option<String>`
- `checksum_sha1: Option<String>`
- `checksum_sha256: Option<String>`

#### `HeadBucketOutput` (`src/types.rs:374-377`)

追加:
- `bucket_arn: Option<String>`
- `bucket_location_type: Option<String>` (issue 0059 後続で型化検討)
- `bucket_location_name: Option<String>`
- `access_point_alias: Option<bool>`

#### `ListBucketsOutput` (`src/types.rs:399-406`)

追加:
- `owner: Option<Owner>`

### 3. 関連型の新設

```rust
#[derive(Debug, Clone)]
pub struct Owner {
    pub display_name: Option<String>,
    pub id: Option<String>,
}

#[derive(Debug, Clone)]
pub struct RestoreStatus {
    pub is_restore_in_progress: Option<bool>,
    pub restore_expiry_date: Option<String>,
}
```

### 4. パース処理の追加

各 `parse_response` 内で XML 要素 (`<Owner>`, `<RestoreStatus>`, `<ChecksumCRC32>` 等) およびレスポンスヘッダー (`x-amz-expiration`, `x-amz-server-side-encryption`, 各 `x-amz-checksum-*`, `x-amz-storage-class` 等) からフィールドを抽出する処理を追加する。

### 5. issue 0057 の pending 解除

`issues/pending/0057-feature-expected-bucket-owner-request-payer.md` を `issues/` に `git mv` で戻す。本 issue 完了後の別作業として処理する。

### 6. `lib.rs` の `pub use` 更新

```rust
pub use types::{Owner, RestoreStatus};
```

## AWS S3 API Reference

複数 API のレスポンスフィールドを補完するため、代表的な箇所を引用する。

- [GetObject](https://docs.aws.amazon.com/AmazonS3/latest/API/API_GetObject.html)

  > Response Headers: x-amz-server-side-encryption, x-amz-storage-class, x-amz-expiration, x-amz-restore, x-amz-checksum-crc32, x-amz-checksum-crc32c, x-amz-checksum-crc64nvme, x-amz-checksum-sha1, x-amz-checksum-sha256, accept-ranges, x-amz-mp-parts-count, x-amz-tagging-count, x-amz-replication-status, x-amz-delete-marker.

- [PutObject](https://docs.aws.amazon.com/AmazonS3/latest/API/API_PutObject.html)

  > Response Headers: x-amz-expiration, x-amz-server-side-encryption, x-amz-server-side-encryption-aws-kms-key-id, x-amz-server-side-encryption-bucket-key-enabled, x-amz-checksum-crc32, x-amz-checksum-crc32c, x-amz-checksum-crc64nvme, x-amz-checksum-sha1, x-amz-checksum-sha256, x-amz-checksum-type, x-amz-request-charged, x-amz-version-id.

- [HeadObject](https://docs.aws.amazon.com/AmazonS3/latest/API/API_HeadObject.html)

  > Response Headers: Content-Encoding, Content-Language, Content-Disposition, Cache-Control, Expires, x-amz-storage-class, x-amz-server-side-encryption, x-amz-mp-parts-count, x-amz-tagging-count, x-amz-replication-status, x-amz-restore, x-amz-checksum-crc32, x-amz-checksum-crc32c, x-amz-checksum-crc64nvme, x-amz-checksum-sha1, x-amz-checksum-sha256, accept-ranges.

- [ListObjectsV2](https://docs.aws.amazon.com/AmazonS3/latest/API/API_ListObjectsV2.html)

  XML レスポンス内 `<Contents>` 要素:

  > ```xml
  > <Contents>
  >    <Key>string</Key>
  >    <LastModified>timestamp</LastModified>
  >    <ETag>string</ETag>
  >    <ChecksumAlgorithm>string</ChecksumAlgorithm>
  >    <ChecksumType>string</ChecksumType>
  >    <Size>long</Size>
  >    <StorageClass>string</StorageClass>
  >    <Owner>
  >       <DisplayName>string</DisplayName>
  >       <ID>string</ID>
  >    </Owner>
  >    <RestoreStatus>
  >       <IsRestoreInProgress>boolean</IsRestoreInProgress>
  >       <RestoreExpiryDate>timestamp</RestoreExpiryDate>
  >    </RestoreStatus>
  > </Contents>
  > ```

- [DeleteObjects (Conditional)](https://docs.aws.amazon.com/AmazonS3/latest/API/API_ObjectIdentifier.html)

  > ETag - The entity tag of the object. LastModifiedTime - The time at which the object was last modified. Size - The size of the object in bytes.

- [HeadBucket](https://docs.aws.amazon.com/AmazonS3/latest/API/API_HeadBucket.html)

  > Response Headers: x-amz-bucket-region, x-amz-bucket-arn, x-amz-bucket-location-type, x-amz-bucket-location-name, x-amz-access-point-alias.

- [ListBuckets](https://docs.aws.amazon.com/AmazonS3/latest/API/API_ListBuckets.html)

  XML レスポンス:

  > ```xml
  > <ListAllMyBucketsResult>
  >    <Buckets>...</Buckets>
  >    <Owner>
  >       <DisplayName>string</DisplayName>
  >       <ID>string</ID>
  >    </Owner>
  >    <ContinuationToken>string</ContinuationToken>
  >    <Prefix>string</Prefix>
  > </ListAllMyBucketsResult>
  > ```

## 影響範囲

- `docs/AWS_SDK_RUST.md` の対応表全面再分類。
- `src/types.rs` の各 `*Output` 構造体へのフィールド追加 (合計 80 フィールド以上)。
- `src/types.rs` への `Owner` / `RestoreStatus` 型新設。
- `src/api/` 配下の各 `parse_response` 関数のパース処理拡張。
- `src/lib.rs` の `pub use` 更新。
- `examples/s3cli`, `tests/` の既存出力アクセスは影響を受けない (フィールド追加のみのため)。

## 細分化の検討

本 issue は規模が大きいため、実装時には以下のように分割実施を検討する:

- 0064-a: `docs/AWS_SDK_RUST.md` の方針再分類のみ (文書修正)
- 0064-b: `Owner` / `RestoreStatus` 型新設、`Object` / `ObjectVersion` / `ListBucketsOutput` への追加
- 0064-c: `GetObjectOutput` / `HeadObjectOutput` のフィールド補完
- 0064-d: `PutObjectOutput` / `UploadPartOutput` / `CompleteMultipartUploadOutput` / `CopyObjectOutput` の SSE/checksum 補完
- 0064-e: `ObjectIdentifier` / `CompletedPart` / `HeadBucketOutput` の補完

実装着手時に細分化要否を判断する。

## 依存関係

- 本 issue は issue 0058 (リネーム), 0059 (enum 化), 0060 (時刻処理), 0063 (`CopyObjectResult`) の後に実施することで型整合が取りやすい。
- issue 0057 (`expected_bucket_owner` / `request_payer`) の pending 解除は本 issue の方針整理を前提とする。

## 優先度

中

## CHANGES.md への記載

- `[CHANGE] docs/AWS_SDK_RUST.md の対応方針を再分類する`
- `[ADD] Owner / RestoreStatus 型を追加する`
- `[ADD] GetObjectOutput / HeadObjectOutput に取得系レスポンスフィールドを追加する`
- `[ADD] PutObjectOutput / UploadPartOutput / CompleteMultipartUploadOutput / CopyObjectOutput に SSE / checksum / request_charged 等のフィールドを追加する`
- `[ADD] Object / ObjectVersion に owner / restore_status / checksum_algorithm / checksum_type を追加する`
- `[ADD] ObjectIdentifier に e_tag / last_modified_time / size を追加する`
- `[ADD] CompletedPart にアルゴリズム別チェックサムフィールドを追加する`
- `[ADD] HeadBucketOutput に bucket_arn / bucket_location_type / bucket_location_name / access_point_alias を追加する`
- `[ADD] ListBucketsOutput に owner を追加する`
