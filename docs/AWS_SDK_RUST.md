# aws-sdk-rust スタイル対応一覧

aws-sdk-s3 と shiguredo_s3 の API パラメータ対応をまとめる。

## 凡例

- 対応済み: shiguredo_s3 で利用可能
- **未対応**: shiguredo_s3 で未実装
- 対応予定無し: S3 固有機能のため対応しない
- (*): shiguredo_s3 独自パラメータ (aws-sdk-rust に存在しない)

## 対応予定無しのパラメータ

S3 固有機能であり、S3 互換オブジェクトストレージでは基本的に不要なため対応しない。

| パラメータ | 理由 |
|---|---|
| `expected_bucket_owner` | AWS アカウント ID によるクロスアカウント保護 |
| `request_payer` | Requester Pays 課金機能 |
| `grant_*` (5 種) | ACL グラント (レガシー、AWS IAM 依存) |
| `object_lock_*` (3 種) | オブジェクトロック (AWS コンプライアンス用途) |
| `ssekms_encryption_context` | AWS KMS 固有の暗号化コンテキスト |
| `bucket_key_enabled` | SSE-KMS バケットキー最適化 |
| `website_redirect_location` | S3 静的ウェブサイトホスティング |
| `object_ownership` | バケットオーナーシップコントロール |
| `confirm_remove_self_bucket_access` | AWS 固有の安全装置 |
| `write_offset_bytes` | S3 Express One Zone 用 |
| `optional_object_attributes` | S3 固有の拡張属性 |
| `fetch_owner` | AWS IAM ベースのオーナー情報 |
| `mfa` | MFA Delete |
| `bypass_governance_retention` | オブジェクトロック関連 |
| `if_match_initiated_time` | AbortMultipartUpload の S3 固有条件 |
| `if_match_last_modified_time` | DeleteObject の S3 固有条件 |
| `if_match_size` | DeleteObject の S3 固有条件 |
| `mpu_object_size` | S3 固有のマルチパートサイズ指定 |
| `checksum_type` | S3 固有のチェックサムタイプ |

## メソッド名の差異

| aws-sdk-rust | shiguredo_s3 | 備考 |
|---|---|---|
| `versioning_configuration` | `status` | aws-sdk-rust は構造体を受けるが shiguredo_s3 は文字列で指定 |
| `public_access_block_configuration` | 4 つの bool フィールドに分解 | 構造体ではなく個別指定 |
| `tagging` (PutBucketTagging) | `tag` | aws-sdk-rust は構造体を受けるが shiguredo_s3 は Tag を個別追加 |
| `delete` (DeleteObjects) | `object` + `quiet` | aws-sdk-rust は構造体を受けるが shiguredo_s3 は ObjectIdentifier を個別追加 |

## shiguredo_s3 独自パラメータ

| パラメータ | 該当 API | 説明 |
|---|---|---|
| `checksum_value` | PutObject, UploadPart | aws-sdk-rust はアルゴリズム別メソッド (`checksum_crc32` 等) を持つ。shiguredo_s3 は汎用的に `checksum_value` で受ける |

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
