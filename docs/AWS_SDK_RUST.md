# aws-sdk-rust スタイル対応一覧

aws-sdk-s3 と shiguredo_s3 の API パラメータ対応をまとめる。

## 凡例

- 対応済み: shiguredo_s3 で利用可能
- **未対応**: shiguredo_s3 で未実装
- 対応予定無し: S3 固有機能 / S3 互換ストレージで意味が薄いため対応しない
- (*): shiguredo_s3 独自パラメータ (aws-sdk-rust に存在しない)

## 対応方針

`AGENTS.md` / `CLAUDE.md` の「Amazon S3 API の仕様と aws-sdk-rust との互換性を最優先」方針に従い、
パラメータを以下の 3 カテゴリに再分類する。

### 「未対応 (互換性のため対応予定)」

ヘッダー渡しまたは値の追加のみで実装が軽量で、aws-sdk-rust 互換のため対応する。

| パラメータ | 該当 API | 理由 |
|---|---|---|
| `expected_bucket_owner` | 全 API | issue 0057 で対応 |
| `request_payer` | 全 API | issue 0057 で対応、enum 化は issue 0059 後続 |
| `website_redirect_location` | PutObject, CopyObject, CreateMultipartUpload | `x-amz-website-redirect-location` ヘッダー追加のみ |
| `grant_full_control` / `grant_read` / `grant_read_acp` / `grant_write` / `grant_write_acp` | PutObject, CopyObject, CreateBucket, CreateMultipartUpload | レガシー ACL だが API は単純 (文字列ヘッダー) |

### 「入力は対応予定無し、出力は対応」(出力のみ対応)

入力側は機能本格対応とセットになるため保留するが、**出力側はレスポンスをパースして提供するだけで実装コストが軽量** であり、利用者が結果を確認できると有用なもの。

| パラメータ | 入力 (該当 API) | 出力 (該当 *Output) | 理由 |
|---|---|---|---|
| `bucket_key_enabled` | PutObject 等の入力で対応予定無し | `GetObjectOutput` / `HeadObjectOutput` / `PutObjectOutput` 等で対応 | 入力は KMS 機能本格対応とセット、出力は `x-amz-server-side-encryption-bucket-key-enabled` ヘッダーをパースするだけ |
| `ssekms_encryption_context` | PutObject 等の入力で対応予定無し | `CopyObjectOutput` 等で対応 | 入力は KMS 詳細機能、出力は `x-amz-server-side-encryption-context` ヘッダーをパースするだけ |
| `ssekms_key_id` | PutObject 等の入力で対応予定無し | `GetObjectOutput` / `PutObjectOutput` 等で対応 | 入力は KMS 詳細機能、出力は `x-amz-server-side-encryption-aws-kms-key-id` ヘッダーをパースするだけ |
| `sse_customer_algorithm` / `sse_customer_key_md5` | 既存の SSE-C 入力対応の延長 | 上記 Output で対応 | 出力ヘッダーをパースするだけ |

### 「対応予定無し」維持

機能本格対応とセットになるため保留する。

| パラメータ | 理由 |
|---|---|
| `object_lock_legal_hold_status`, `object_lock_mode`, `object_lock_retain_until_date` | Object Lock 機能本格対応とセット (別 issue) |
| `object_lock_enabled_for_bucket` | 同上 |
| `mfa`, `bypass_governance_retention` | Object Lock / Versioning MFA 機能とセット |
| `if_match_initiated_time`, `if_match_last_modified_time`, `if_match_size` | S3 固有の条件付き機能、AWS SDK でも限定 |
| `mpu_object_size`, `write_offset_bytes` | S3 Express One Zone 専用機能 |
| `optional_object_attributes`, `fetch_owner` | AWS IAM ベース、S3 互換ストレージで意味が薄い |
| `confirm_remove_self_bucket_access` | AWS 固有の安全装置 |
| `object_ownership` | バケットオーナーシップコントロール (別 issue) |
| `checksum_type` | issue 0059 後続で型化検討 |

## メソッド名の差異

| aws-sdk-rust | shiguredo_s3 | 備考 |
|---|---|---|
| `versioning_configuration` | `status` | aws-sdk-rust は構造体を受けるが shiguredo_s3 は文字列で指定 |
| `public_access_block_configuration` | 4 つの bool フィールドに分解 | 構造体ではなく個別指定 |
| `tagging` (PutBucketTagging) | `tag` | aws-sdk-rust は構造体を受けるが shiguredo_s3 は Tag を個別追加 |

## shiguredo_s3 独自パラメータ

(該当なし: 旧 `checksum_value` は issue 0062 で aws-sdk-rust と同じ個別 `checksum_*` フィールドに置換済み)

---

## オブジェクト操作

### GetObject

| aws-sdk-rust | shiguredo_s3 |
|---|---|
| bucket | 対応済み |
| key | 対応済み |
| range | 対応済み |
| part_number | 対応済み |
| if_match | 対応済み |
| if_none_match | 対応済み |
| if_modified_since | 対応済み |
| if_unmodified_since | 対応済み |
| sse_customer_algorithm | 対応済み |
| sse_customer_key | 対応済み |
| version_id | 対応済み |
| checksum_mode | **未対応** |
| response_cache_control | 対応済み |
| response_content_disposition | 対応済み |
| response_content_encoding | 対応済み |
| response_content_language | 対応済み |
| response_content_type | 対応済み |
| response_expires | 対応済み |
| expected_bucket_owner | 対応予定無し |
| request_payer | 対応予定無し |

### HeadObject

| aws-sdk-rust | shiguredo_s3 |
|---|---|
| bucket | 対応済み |
| key | 対応済み |
| range | 対応済み |
| part_number | 対応済み |
| if_match | 対応済み |
| if_none_match | 対応済み |
| if_modified_since | 対応済み |
| if_unmodified_since | 対応済み |
| sse_customer_algorithm | 対応済み |
| sse_customer_key | 対応済み |
| version_id | 対応済み |
| checksum_mode | **未対応** |
| expected_bucket_owner | 対応予定無し |
| request_payer | 対応予定無し |

### PutObject

| aws-sdk-rust | shiguredo_s3 |
|---|---|
| bucket | 対応済み |
| key | 対応済み |
| body | 対応済み |
| content_type | 対応済み |
| content_encoding | 対応済み |
| content_disposition | 対応済み |
| content_language | 対応済み |
| cache_control | 対応済み |
| expires | 対応済み |
| checksum_algorithm | 対応済み |
| acl | 対応済み |
| metadata | 対応済み |
| storage_class | 対応済み |
| server_side_encryption | 対応済み |
| ssekms_key_id | 対応済み |
| sse_customer_algorithm | 対応済み |
| sse_customer_key | 対応済み |
| content_length | **未対応** |
| if_match | **未対応** |
| if_none_match | **未対応** |
| tagging | **未対応** |
| expected_bucket_owner | 対応予定無し |
| request_payer | 対応予定無し |
| grant_full_control | 対応予定無し |
| grant_read | 対応予定無し |
| grant_read_acp | 対応予定無し |
| grant_write_acp | 対応予定無し |
| object_lock_legal_hold_status | 対応予定無し |
| object_lock_mode | 対応予定無し |
| object_lock_retain_until_date | 対応予定無し |
| bucket_key_enabled | 対応予定無し |
| ssekms_encryption_context | 対応予定無し |
| website_redirect_location | 対応予定無し |
| write_offset_bytes | 対応予定無し |

### DeleteObject

| aws-sdk-rust | shiguredo_s3 |
|---|---|
| bucket | 対応済み |
| key | 対応済み |
| version_id | 対応済み |
| if_match | **未対応** |
| expected_bucket_owner | 対応予定無し |
| request_payer | 対応予定無し |
| mfa | 対応予定無し |
| bypass_governance_retention | 対応予定無し |
| if_match_last_modified_time | 対応予定無し |
| if_match_size | 対応予定無し |

### DeleteObjects

| aws-sdk-rust | shiguredo_s3 |
|---|---|
| bucket | 対応済み |
| delete | 対応済み (object + quiet) |
| checksum_algorithm | 対応済み |
| expected_bucket_owner | 対応予定無し |
| request_payer | 対応予定無し |
| mfa | 対応予定無し |
| bypass_governance_retention | 対応予定無し |

### CopyObject

| aws-sdk-rust | shiguredo_s3 |
|---|---|
| bucket | 対応済み |
| key | 対応済み |
| copy_source | 対応済み |
| metadata_directive | 対応済み |
| content_type | 対応済み |
| content_encoding | 対応済み |
| content_disposition | 対応済み |
| content_language | 対応済み |
| cache_control | 対応済み |
| expires | 対応済み |
| acl | 対応済み |
| metadata | 対応済み |
| storage_class | 対応済み |
| server_side_encryption | 対応済み |
| ssekms_key_id | 対応済み |
| sse_customer_algorithm | 対応済み |
| sse_customer_key | 対応済み |
| copy_source_sse_customer_algorithm | 対応済み |
| copy_source_sse_customer_key | 対応済み |
| checksum_algorithm | 対応済み |
| copy_source_if_match | **未対応** |
| copy_source_if_modified_since | **未対応** |
| copy_source_if_none_match | **未対応** |
| copy_source_if_unmodified_since | **未対応** |
| if_match | **未対応** |
| if_none_match | **未対応** |
| tagging | **未対応** |
| tagging_directive | **未対応** |
| expected_bucket_owner | 対応予定無し |
| expected_source_bucket_owner | 対応予定無し |
| request_payer | 対応予定無し |
| grant_full_control | 対応予定無し |
| grant_read | 対応予定無し |
| grant_read_acp | 対応予定無し |
| grant_write_acp | 対応予定無し |
| object_lock_legal_hold_status | 対応予定無し |
| object_lock_mode | 対応予定無し |
| object_lock_retain_until_date | 対応予定無し |
| bucket_key_enabled | 対応予定無し |
| ssekms_encryption_context | 対応予定無し |
| website_redirect_location | 対応予定無し |

### ListObjectsV2

| aws-sdk-rust | shiguredo_s3 |
|---|---|
| bucket | 対応済み |
| prefix | 対応済み |
| delimiter | 対応済み |
| max_keys | 対応済み |
| continuation_token | 対応済み |
| start_after | 対応済み |
| encoding_type | **未対応** |
| expected_bucket_owner | 対応予定無し |
| request_payer | 対応予定無し |
| fetch_owner | 対応予定無し |
| optional_object_attributes | 対応予定無し |

---

## マルチパート

### CreateMultipartUpload

| aws-sdk-rust | shiguredo_s3 |
|---|---|
| bucket | 対応済み |
| key | 対応済み |
| content_type | 対応済み |
| content_encoding | 対応済み |
| content_disposition | 対応済み |
| content_language | 対応済み |
| cache_control | 対応済み |
| expires | 対応済み |
| metadata | 対応済み |
| acl | 対応済み |
| storage_class | 対応済み |
| server_side_encryption | 対応済み |
| ssekms_key_id | 対応済み |
| sse_customer_algorithm | 対応済み |
| sse_customer_key | 対応済み |
| checksum_algorithm | 対応済み |
| tagging | **未対応** |
| expected_bucket_owner | 対応予定無し |
| request_payer | 対応予定無し |
| grant_full_control | 対応予定無し |
| grant_read | 対応予定無し |
| grant_read_acp | 対応予定無し |
| grant_write_acp | 対応予定無し |
| object_lock_legal_hold_status | 対応予定無し |
| object_lock_mode | 対応予定無し |
| object_lock_retain_until_date | 対応予定無し |
| bucket_key_enabled | 対応予定無し |
| checksum_type | 対応予定無し |
| ssekms_encryption_context | 対応予定無し |
| website_redirect_location | 対応予定無し |

### UploadPart

| aws-sdk-rust | shiguredo_s3 |
|---|---|
| bucket | 対応済み |
| key | 対応済み |
| upload_id | 対応済み |
| part_number | 対応済み |
| body | 対応済み |
| checksum_algorithm | 対応済み |
| sse_customer_algorithm | 対応済み |
| sse_customer_key | 対応済み |
| content_length | **未対応** |
| expected_bucket_owner | 対応予定無し |
| request_payer | 対応予定無し |

### CompleteMultipartUpload

| aws-sdk-rust | shiguredo_s3 |
|---|---|
| bucket | 対応済み |
| key | 対応済み |
| upload_id | 対応済み |
| multipart_upload | 対応済み |
| if_match | **未対応** |
| if_none_match | **未対応** |
| sse_customer_algorithm | 対応済み |
| sse_customer_key | 対応済み |
| expected_bucket_owner | 対応予定無し |
| request_payer | 対応予定無し |
| checksum_type | 対応予定無し |
| mpu_object_size | 対応予定無し |

### AbortMultipartUpload

| aws-sdk-rust | shiguredo_s3 |
|---|---|
| bucket | 対応済み |
| key | 対応済み |
| upload_id | 対応済み |
| expected_bucket_owner | 対応予定無し |
| request_payer | 対応予定無し |
| if_match_initiated_time | 対応予定無し |

### ListParts

| aws-sdk-rust | shiguredo_s3 |
|---|---|
| bucket | 対応済み |
| key | 対応済み |
| upload_id | 対応済み |
| max_parts | 対応済み |
| part_number_marker | 対応済み |
| sse_customer_algorithm | 対応済み |
| sse_customer_key | 対応済み |
| expected_bucket_owner | 対応予定無し |
| request_payer | 対応予定無し |

### ListMultipartUploads

| aws-sdk-rust | shiguredo_s3 |
|---|---|
| bucket | 対応済み |
| prefix | 対応済み |
| delimiter | 対応済み |
| max_uploads | 対応済み |
| key_marker | 対応済み |
| upload_id_marker | 対応済み |
| encoding_type | **未対応** |
| expected_bucket_owner | 対応予定無し |
| request_payer | 対応予定無し |

---

## バケット操作

### HeadBucket

| aws-sdk-rust | shiguredo_s3 |
|---|---|
| bucket | 対応済み |
| expected_bucket_owner | 対応予定無し |

### CreateBucket

| aws-sdk-rust | shiguredo_s3 |
|---|---|
| bucket | 対応済み |
| create_bucket_configuration | 対応済み |
| acl | **未対応** |
| expected_bucket_owner | 対応予定無し |
| grant_full_control | 対応予定無し |
| grant_read | 対応予定無し |
| grant_read_acp | 対応予定無し |
| grant_write | 対応予定無し |
| grant_write_acp | 対応予定無し |
| object_lock_enabled_for_bucket | 対応予定無し |
| object_ownership | 対応予定無し |

### DeleteBucket

| aws-sdk-rust | shiguredo_s3 |
|---|---|
| bucket | 対応済み |
| expected_bucket_owner | 対応予定無し |

### ListBuckets (完全対応)

| aws-sdk-rust | shiguredo_s3 |
|---|---|
| max_buckets | 対応済み |
| continuation_token | 対応済み |
| prefix | 対応済み |
| bucket_region | 対応済み |

---

## バケット設定

### GetBucketVersioning

| aws-sdk-rust | shiguredo_s3 |
|---|---|
| bucket | 対応済み |
| expected_bucket_owner | 対応予定無し |

### PutBucketVersioning

| aws-sdk-rust | shiguredo_s3 |
|---|---|
| bucket | 対応済み |
| versioning_configuration | 対応済み (status で簡略化) |
| checksum_algorithm | 対応済み |
| expected_bucket_owner | 対応予定無し |
| mfa | 対応予定無し |

### GetBucketTagging

| aws-sdk-rust | shiguredo_s3 |
|---|---|
| bucket | 対応済み |
| expected_bucket_owner | 対応予定無し |

### PutBucketTagging

| aws-sdk-rust | shiguredo_s3 |
|---|---|
| bucket | 対応済み |
| tagging | 対応済み (tag で個別追加) |
| checksum_algorithm | 対応済み |
| expected_bucket_owner | 対応予定無し |

### DeleteBucketTagging

| aws-sdk-rust | shiguredo_s3 |
|---|---|
| bucket | 対応済み |
| expected_bucket_owner | 対応予定無し |

### GetBucketPolicy

| aws-sdk-rust | shiguredo_s3 |
|---|---|
| bucket | 対応済み |
| expected_bucket_owner | 対応予定無し |

### PutBucketPolicy

| aws-sdk-rust | shiguredo_s3 |
|---|---|
| bucket | 対応済み |
| policy | 対応済み |
| checksum_algorithm | 対応済み |
| expected_bucket_owner | 対応予定無し |
| confirm_remove_self_bucket_access | 対応予定無し |

### DeleteBucketPolicy

| aws-sdk-rust | shiguredo_s3 |
|---|---|
| bucket | 対応済み |
| expected_bucket_owner | 対応予定無し |

### GetPublicAccessBlock

| aws-sdk-rust | shiguredo_s3 |
|---|---|
| bucket | 対応済み |
| expected_bucket_owner | 対応予定無し |

### PutPublicAccessBlock

| aws-sdk-rust | shiguredo_s3 |
|---|---|
| bucket | 対応済み |
| public_access_block_configuration | 対応済み (4 つの bool に分解) |
| checksum_algorithm | 対応済み |
| expected_bucket_owner | 対応予定無し |

### DeletePublicAccessBlock

| aws-sdk-rust | shiguredo_s3 |
|---|---|
| bucket | 対応済み |
| expected_bucket_owner | 対応予定無し |

---

## レスポンスフィールド対応状況

リクエストパラメータだけでなく、レスポンスフィールドの対応状況も追跡する。

### GetObjectOutput

| aws-sdk-rust | shiguredo_s3 |
|---|---|
| body | 対応済み |
| content_type | 対応済み |
| content_length | 対応済み |
| e_tag | 対応済み |
| last_modified | 対応済み |
| version_id | 対応済み |
| metadata | 対応済み |
| content_encoding | **未対応** |
| content_disposition | **未対応** |
| content_language | **未対応** |
| cache_control | **未対応** |
| expires | **未対応** |
| storage_class | **未対応** |
| server_side_encryption | **未対応** |
| sse_customer_algorithm | **未対応** |
| ssekms_key_id | **未対応** |
| parts_count | **未対応** |
| checksum_crc32 | **未対応** |
| checksum_crc32c | **未対応** |
| checksum_sha1 | **未対応** |
| checksum_sha256 | **未対応** |
| checksum_crc64nvme | **未対応** |

### HeadObjectOutput

| aws-sdk-rust | shiguredo_s3 |
|---|---|
| content_type | 対応済み |
| content_length | 対応済み |
| e_tag | 対応済み |
| last_modified | 対応済み |
| version_id | 対応済み |
| metadata | 対応済み |
| content_encoding | **未対応** |
| content_disposition | **未対応** |
| content_language | **未対応** |
| cache_control | **未対応** |
| expires | **未対応** |
| storage_class | **未対応** |
| server_side_encryption | **未対応** |
| sse_customer_algorithm | **未対応** |
| ssekms_key_id | **未対応** |
| parts_count | **未対応** |
| checksum_crc32 | **未対応** |
| checksum_crc32c | **未対応** |
| checksum_sha1 | **未対応** |
| checksum_sha256 | **未対応** |
| checksum_crc64nvme | **未対応** |

### PutObjectOutput

| aws-sdk-rust | shiguredo_s3 |
|---|---|
| e_tag | 対応済み |
| version_id | 対応済み |
| server_side_encryption | **未対応** |
| sse_customer_algorithm | **未対応** |
| ssekms_key_id | **未対応** |
| checksum_crc32 | **未対応** |
| checksum_crc32c | **未対応** |
| checksum_sha1 | **未対応** |
| checksum_sha256 | **未対応** |
| checksum_crc64nvme | **未対応** |

### CopyObjectOutput

| aws-sdk-rust | shiguredo_s3 |
|---|---|
| e_tag | 対応済み |
| last_modified | 対応済み |
| version_id | 対応済み |
| copy_source_version_id | 対応済み |
| server_side_encryption | **未対応** |
| sse_customer_algorithm | **未対応** |
| ssekms_key_id | **未対応** |

### CompleteMultipartUploadOutput

| aws-sdk-rust | shiguredo_s3 |
|---|---|
| location | 対応済み |
| bucket | 対応済み |
| key | 対応済み |
| e_tag | 対応済み |
| version_id | 対応済み |
| server_side_encryption | **未対応** |
| ssekms_key_id | **未対応** |

### DeleteObjectOutput

| aws-sdk-rust | shiguredo_s3 |
|---|---|
| delete_marker | 対応済み |
| version_id | 対応済み |
