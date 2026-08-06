use std::time::SystemTime;

use super::enums::{ChecksumAlgorithm, EncodingType, ServerSideEncryption, StorageClass};
use super::model::{
    CorsRule, ErrorDocument, EventBridgeConfiguration, IndexDocument, LambdaFunctionConfiguration,
    LifecycleRule, ObjectLockConfiguration, ObjectLockLegalHold, ObjectLockRetention,
    OwnershipControlsRule, QueueConfiguration, RedirectAllRequestsTo, RoutingRule,
    ServerSideEncryptionConfiguration, Tag, TopicConfiguration,
};

/// GetObject の結果
#[derive(Debug)]
pub struct GetObjectOutput {
    pub body: Vec<u8>,
    pub content_type: Option<String>,
    pub content_length: Option<i64>,
    pub e_tag: Option<String>,
    pub last_modified: Option<SystemTime>,
    /// オブジェクトのバージョン ID (バージョニング有効時)
    pub version_id: Option<String>,
    /// カスタムメタデータ (x-amz-meta-* ヘッダーから抽出)
    pub metadata: Option<std::collections::HashMap<String, String>>,
    /// CRC32 チェックサム (checksum_mode=ENABLED 時)
    pub checksum_crc32: Option<String>,
    /// CRC32C チェックサム (checksum_mode=ENABLED 時)
    pub checksum_crc32_c: Option<String>,
    /// CRC64NVME チェックサム (checksum_mode=ENABLED 時)
    pub checksum_crc64_nvme: Option<String>,
    /// SHA1 チェックサム (checksum_mode=ENABLED 時)
    pub checksum_sha1: Option<String>,
    /// SHA256 チェックサム (checksum_mode=ENABLED 時)
    pub checksum_sha256: Option<String>,
    pub content_encoding: Option<String>,
    pub content_disposition: Option<String>,
    pub content_language: Option<String>,
    pub cache_control: Option<String>,
    /// `Expires` ヘッダー (IMF-fixdate)
    pub expires: Option<SystemTime>,
    /// `x-amz-storage-class` (STANDARD では省略されることがある)
    pub storage_class: Option<StorageClass>,
    /// `x-amz-mp-parts-count`
    pub parts_count: Option<i32>,
    /// `Accept-Ranges`
    pub accept_ranges: Option<String>,
    /// `x-amz-delete-marker` (true の場合は対象オブジェクトが削除マーカー)
    pub delete_marker: Option<bool>,
    /// `x-amz-replication-status`
    pub replication_status: Option<String>,
    /// `x-amz-restore` (Glacier 復元状態)
    pub restore: Option<String>,
    /// `x-amz-expiration` (ライフサイクル失効情報)
    pub expiration: Option<String>,
    pub server_side_encryption: Option<ServerSideEncryption>,
    pub sse_customer_algorithm: Option<String>,
    pub sse_customer_key_md5: Option<String>,
    /// `x-amz-server-side-encryption-aws-kms-key-id`
    pub ssekms_key_id: Option<String>,
    /// `x-amz-server-side-encryption-bucket-key-enabled`
    pub bucket_key_enabled: Option<bool>,
    /// `x-amz-request-charged`
    pub request_charged: Option<String>,
    /// `x-amz-tagging-count`
    pub tag_count: Option<i32>,
    /// `x-amz-website-redirect-location`
    pub website_redirect_location: Option<String>,
}

/// HeadObject の結果
#[derive(Debug)]
pub struct HeadObjectOutput {
    pub content_type: Option<String>,
    pub content_length: Option<i64>,
    pub e_tag: Option<String>,
    pub last_modified: Option<SystemTime>,
    /// `x-amz-storage-class` (STANDARD では省略されることがある)
    pub storage_class: Option<StorageClass>,
    /// オブジェクトのバージョン ID (バージョニング有効時)
    pub version_id: Option<String>,
    /// カスタムメタデータ (x-amz-meta-* ヘッダーから抽出)
    pub metadata: Option<std::collections::HashMap<String, String>>,
    /// CRC32 チェックサム (checksum_mode=ENABLED 時)
    pub checksum_crc32: Option<String>,
    /// CRC32C チェックサム (checksum_mode=ENABLED 時)
    pub checksum_crc32_c: Option<String>,
    /// CRC64NVME チェックサム (checksum_mode=ENABLED 時)
    pub checksum_crc64_nvme: Option<String>,
    /// SHA1 チェックサム (checksum_mode=ENABLED 時)
    pub checksum_sha1: Option<String>,
    /// SHA256 チェックサム (checksum_mode=ENABLED 時)
    pub checksum_sha256: Option<String>,
    pub content_encoding: Option<String>,
    pub content_disposition: Option<String>,
    pub content_language: Option<String>,
    pub cache_control: Option<String>,
    /// `Expires` ヘッダー (IMF-fixdate)
    pub expires: Option<SystemTime>,
    /// `x-amz-mp-parts-count`
    pub parts_count: Option<i32>,
    /// `Accept-Ranges`
    pub accept_ranges: Option<String>,
    /// `x-amz-replication-status`
    pub replication_status: Option<String>,
    /// `x-amz-restore` (Glacier 復元状態)
    pub restore: Option<String>,
    /// `x-amz-expiration` (ライフサイクル失効情報)
    pub expiration: Option<String>,
    pub server_side_encryption: Option<ServerSideEncryption>,
    pub sse_customer_algorithm: Option<String>,
    pub sse_customer_key_md5: Option<String>,
    /// `x-amz-server-side-encryption-aws-kms-key-id`
    pub ssekms_key_id: Option<String>,
    /// `x-amz-server-side-encryption-bucket-key-enabled`
    pub bucket_key_enabled: Option<bool>,
    /// `x-amz-request-charged`
    pub request_charged: Option<String>,
    /// `x-amz-website-redirect-location`
    pub website_redirect_location: Option<String>,
}

/// PutObject の結果
#[derive(Debug)]
pub struct PutObjectOutput {
    pub e_tag: Option<String>,
    /// オブジェクトのバージョン ID (バージョニング有効時)
    pub version_id: Option<String>,
    pub expiration: Option<String>,
    pub server_side_encryption: Option<ServerSideEncryption>,
    pub sse_customer_algorithm: Option<String>,
    pub sse_customer_key_md5: Option<String>,
    pub ssekms_key_id: Option<String>,
    pub bucket_key_enabled: Option<bool>,
    pub request_charged: Option<String>,
    pub checksum_crc32: Option<String>,
    pub checksum_crc32_c: Option<String>,
    pub checksum_crc64_nvme: Option<String>,
    pub checksum_sha1: Option<String>,
    pub checksum_sha256: Option<String>,
    pub checksum_type: Option<String>,
}

/// DeleteObject の結果
#[derive(Debug)]
pub struct DeleteObjectOutput {
    pub delete_marker: Option<bool>,
    pub version_id: Option<String>,
    /// `x-amz-request-charged`
    pub request_charged: Option<String>,
}

/// CreateMultipartUpload の結果
#[derive(Debug)]
pub struct CreateMultipartUploadOutput {
    pub bucket: Option<String>,
    pub key: Option<String>,
    pub upload_id: Option<String>,
    /// `x-amz-request-charged`
    pub request_charged: Option<String>,
}

/// UploadPart の結果
#[derive(Debug)]
pub struct UploadPartOutput {
    pub e_tag: Option<String>,
    pub server_side_encryption: Option<ServerSideEncryption>,
    pub sse_customer_algorithm: Option<String>,
    pub sse_customer_key_md5: Option<String>,
    pub ssekms_key_id: Option<String>,
    pub bucket_key_enabled: Option<bool>,
    pub request_charged: Option<String>,
    pub checksum_crc32: Option<String>,
    pub checksum_crc32_c: Option<String>,
    pub checksum_crc64_nvme: Option<String>,
    pub checksum_sha1: Option<String>,
    pub checksum_sha256: Option<String>,
}

/// UploadPartCopy の結果
#[derive(Debug)]
pub struct UploadPartCopyOutput {
    pub e_tag: Option<String>,
    pub last_modified: Option<SystemTime>,
    /// コピー元オブジェクトのバージョン ID
    pub copy_source_version_id: Option<String>,
}

/// CompleteMultipartUpload の結果
#[derive(Debug)]
pub struct CompleteMultipartUploadOutput {
    pub location: Option<String>,
    pub bucket: Option<String>,
    pub key: Option<String>,
    pub e_tag: Option<String>,
    /// オブジェクトのバージョン ID (バージョニング有効時)
    pub version_id: Option<String>,
    pub expiration: Option<String>,
    pub server_side_encryption: Option<ServerSideEncryption>,
    pub ssekms_key_id: Option<String>,
    pub bucket_key_enabled: Option<bool>,
    pub request_charged: Option<String>,
    pub checksum_crc32: Option<String>,
    pub checksum_crc32_c: Option<String>,
    pub checksum_crc64_nvme: Option<String>,
    pub checksum_sha1: Option<String>,
    pub checksum_sha256: Option<String>,
    pub checksum_type: Option<String>,
}

/// AbortMultipartUpload の結果
#[derive(Debug)]
pub struct AbortMultipartUploadOutput {}

/// CopyObject の結果
#[derive(Debug)]
pub struct CopyObjectOutput {
    /// コピー結果 (ETag / LastModified / 各 checksum / checksum_type)
    ///
    /// AWS S3 API のレスポンス XML `<CopyObjectResult>` 要素に対応する。
    pub copy_object_result: Option<CopyObjectResult>,
    /// コピー先オブジェクトのバージョン ID (バージョニング有効時)
    pub version_id: Option<String>,
    /// コピー元オブジェクトのバージョン ID
    pub copy_source_version_id: Option<String>,
    pub expiration: Option<String>,
    pub server_side_encryption: Option<ServerSideEncryption>,
    pub sse_customer_algorithm: Option<String>,
    pub sse_customer_key_md5: Option<String>,
    pub ssekms_key_id: Option<String>,
    /// `x-amz-server-side-encryption-context`
    pub ssekms_encryption_context: Option<String>,
    pub bucket_key_enabled: Option<bool>,
    pub request_charged: Option<String>,
}

/// CopyObject の結果の中身
///
/// AWS S3 API のレスポンス XML `<CopyObjectResult>` 要素に対応する。
/// aws-sdk-rust の `aws_sdk_s3::types::CopyObjectResult` と同じ構造。
#[derive(Debug, Clone)]
pub struct CopyObjectResult {
    pub e_tag: Option<String>,
    pub last_modified: Option<SystemTime>,
    pub checksum_crc32: Option<String>,
    pub checksum_crc32_c: Option<String>,
    pub checksum_crc64_nvme: Option<String>,
    pub checksum_sha1: Option<String>,
    pub checksum_sha256: Option<String>,
    /// `<ChecksumType>` 要素
    pub checksum_type: Option<String>,
}

/// DeleteObjects の結果
#[derive(Debug)]
pub struct DeleteObjectsOutput {
    pub deleted: Option<Vec<DeletedObject>>,
    pub errors: Option<Vec<DeleteError>>,
}

/// 削除されたオブジェクト
#[derive(Debug, Clone)]
pub struct DeletedObject {
    pub key: Option<String>,
    pub version_id: Option<String>,
    pub delete_marker: Option<bool>,
    pub delete_marker_version_id: Option<String>,
}

/// 削除エラー
#[derive(Debug, Clone)]
pub struct DeleteError {
    pub key: Option<String>,
    pub code: Option<String>,
    pub message: Option<String>,
}

/// ListObjects の結果
///
/// aws-sdk-rust の `ListObjectsOutput` と同じ構造。
/// ListObjects v1 は v2 と異なり `Marker` / `NextMarker` でページングする。
#[derive(Debug)]
pub struct ListObjectsOutput {
    pub is_truncated: Option<bool>,
    /// リクエストで送信した marker のエコー
    pub marker: Option<String>,
    /// 次ページの開始位置 (delimiter 指定時のみ返る)
    ///
    /// delimiter 未指定で切り詰められた場合は最後の `Key` を marker に使う。
    pub next_marker: Option<String>,
    pub contents: Option<Vec<Object>>,
    pub name: Option<String>,
    pub prefix: Option<String>,
    pub delimiter: Option<String>,
    pub max_keys: Option<i32>,
    pub common_prefixes: Option<Vec<CommonPrefix>>,
    /// `<EncodingType>` 要素 (url)
    pub encoding_type: Option<EncodingType>,
    /// `x-amz-request-charged` レスポンスヘッダー
    pub request_charged: Option<String>,
}

/// ListObjectsV2 の結果
#[derive(Debug)]
pub struct ListObjectsV2Output {
    pub is_truncated: Option<bool>,
    pub contents: Option<Vec<Object>>,
    pub name: Option<String>,
    pub prefix: Option<String>,
    pub delimiter: Option<String>,
    pub max_keys: Option<i32>,
    pub common_prefixes: Option<Vec<CommonPrefix>>,
    /// `<EncodingType>` 要素 (url)
    pub encoding_type: Option<EncodingType>,
    pub key_count: Option<i32>,
    pub continuation_token: Option<String>,
    pub next_continuation_token: Option<String>,
    pub start_after: Option<String>,
}

/// ListObjectVersions の結果
#[derive(Debug)]
pub struct ListObjectVersionsOutput {
    pub is_truncated: Option<bool>,
    pub next_key_marker: Option<String>,
    pub next_version_id_marker: Option<String>,
    pub versions: Option<Vec<ObjectVersion>>,
    pub delete_markers: Option<Vec<DeleteMarkerEntry>>,
    pub common_prefixes: Option<Vec<CommonPrefix>>,
    pub name: Option<String>,
    pub prefix: Option<String>,
    pub delimiter: Option<String>,
    pub max_keys: Option<i32>,
    pub key_marker: Option<String>,
    pub version_id_marker: Option<String>,
    pub encoding_type: Option<EncodingType>,
}

/// オブジェクトバージョンのメタデータ
#[derive(Debug, Clone)]
pub struct ObjectVersion {
    pub key: Option<String>,
    pub version_id: Option<String>,
    pub is_latest: Option<bool>,
    pub last_modified: Option<SystemTime>,
    pub e_tag: Option<String>,
    pub size: Option<i64>,
    pub storage_class: Option<StorageClass>,
    pub owner: Option<Owner>,
    pub restore_status: Option<RestoreStatus>,
    /// 各オブジェクトに有効なチェックサムアルゴリズムのリスト
    pub checksum_algorithm: Option<Vec<ChecksumAlgorithm>>,
    /// `<ChecksumType>` 要素
    pub checksum_type: Option<String>,
}

/// 削除マーカーのメタデータ
#[derive(Debug, Clone)]
pub struct DeleteMarkerEntry {
    pub key: Option<String>,
    pub version_id: Option<String>,
    pub is_latest: Option<bool>,
    pub last_modified: Option<SystemTime>,
}

/// S3 オブジェクトのメタデータ
#[derive(Debug, Clone)]
pub struct Object {
    pub key: Option<String>,
    pub last_modified: Option<SystemTime>,
    pub e_tag: Option<String>,
    pub size: Option<i64>,
    pub storage_class: Option<StorageClass>,
    pub owner: Option<Owner>,
    pub restore_status: Option<RestoreStatus>,
    /// 各オブジェクトに有効なチェックサムアルゴリズムのリスト
    pub checksum_algorithm: Option<Vec<ChecksumAlgorithm>>,
    /// `<ChecksumType>` 要素
    pub checksum_type: Option<String>,
}

/// バケット / オブジェクトのオーナー情報
///
/// AWS S3 API のレスポンス XML `<Owner>` 要素に対応する。
/// aws-sdk-rust の `aws_sdk_s3::types::Owner` と同じ構造。
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Owner {
    pub display_name: Option<String>,
    pub id: Option<String>,
}

/// オブジェクトの復元状態 (Glacier / Deep Archive)
///
/// AWS S3 API のレスポンス XML `<RestoreStatus>` 要素に対応する。
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct RestoreStatus {
    pub is_restore_in_progress: Option<bool>,
    pub restore_expiry_date: Option<SystemTime>,
}

/// 共通プレフィックス
#[derive(Debug, Clone)]
pub struct CommonPrefix {
    pub prefix: Option<String>,
}

/// HeadBucket の結果
///
/// HeadBucket の成功レスポンス
#[derive(Debug)]
pub struct HeadBucketOutput {
    /// バケットが存在するリージョン
    pub bucket_region: Option<String>,
    /// バケットの ARN
    pub bucket_arn: Option<String>,
    /// バケットのロケーションタイプ
    pub bucket_location_type: Option<String>,
    /// バケットのロケーション名
    pub bucket_location_name: Option<String>,
    /// アクセスポイント alias であるかどうか
    pub access_point_alias: Option<bool>,
}

/// CreateBucket の結果
#[derive(Debug)]
pub struct CreateBucketOutput {
    pub location: Option<String>,
}

/// DeleteBucket の結果
#[derive(Debug)]
pub struct DeleteBucketOutput {}

/// ListBuckets の結果
#[derive(Debug)]
pub struct ListBucketsOutput {
    pub buckets: Vec<Bucket>,
    /// 次ページを取得するための継続トークン (結果が切り詰められた場合のみ)
    pub continuation_token: Option<String>,
    /// 今回のリクエストで使用したプレフィックスフィルタ
    pub prefix: Option<String>,
    /// バケット一覧の所有者情報
    pub owner: Option<Owner>,
}

/// バケット情報
#[derive(Debug, Clone)]
pub struct Bucket {
    pub name: Option<String>,
    pub creation_date: Option<SystemTime>,
    /// バケットが存在するリージョン
    pub bucket_region: Option<String>,
    /// バケットの ARN
    pub bucket_arn: Option<String>,
}

/// GetBucketVersioning の結果
#[derive(Debug)]
pub struct GetBucketVersioningOutput {
    /// "Enabled" または "Suspended"。設定されていない場合は None
    pub status: Option<String>,
    /// "Enabled" または "Disabled"
    pub mfa_delete: Option<String>,
}

/// PutBucketVersioning の結果
#[derive(Debug)]
pub struct PutBucketVersioningOutput {}

/// GetBucketTagging の結果
#[derive(Debug)]
pub struct GetBucketTaggingOutput {
    pub tag_set: Vec<Tag>,
}

/// PutBucketTagging の結果
#[derive(Debug)]
pub struct PutBucketTaggingOutput {}

/// DeleteBucketTagging の結果
#[derive(Debug)]
pub struct DeleteBucketTaggingOutput {}

/// GetObjectLegalHold の結果
#[derive(Debug)]
pub struct GetObjectLegalHoldOutput {
    pub legal_hold: Option<ObjectLockLegalHold>,
}

/// PutObjectLegalHold の結果
#[derive(Debug)]
pub struct PutObjectLegalHoldOutput {}

/// GetObjectRetention の結果
#[derive(Debug)]
pub struct GetObjectRetentionOutput {
    pub retention: Option<ObjectLockRetention>,
}

/// PutObjectRetention の結果
#[derive(Debug)]
pub struct PutObjectRetentionOutput {}

/// GetObjectLockConfiguration の結果
#[derive(Debug)]
pub struct GetObjectLockConfigurationOutput {
    pub object_lock_configuration: Option<ObjectLockConfiguration>,
}

/// PutObjectLockConfiguration の結果
#[derive(Debug)]
pub struct PutObjectLockConfigurationOutput {}

/// GetBucketOwnershipControls の結果
#[derive(Debug)]
pub struct GetBucketOwnershipControlsOutput {
    pub rules: Vec<OwnershipControlsRule>,
}

/// PutBucketOwnershipControls の結果
#[derive(Debug)]
pub struct PutBucketOwnershipControlsOutput {}

/// DeleteBucketOwnershipControls の結果
#[derive(Debug)]
pub struct DeleteBucketOwnershipControlsOutput {}

/// GetBucketWebsite の結果
#[derive(Debug)]
pub struct GetBucketWebsiteOutput {
    pub index_document: Option<IndexDocument>,
    pub error_document: Option<ErrorDocument>,
    pub redirect_all_requests_to: Option<RedirectAllRequestsTo>,
    pub routing_rules: Vec<RoutingRule>,
}

/// PutBucketWebsite の結果
#[derive(Debug)]
pub struct PutBucketWebsiteOutput {}

/// DeleteBucketWebsite の結果
#[derive(Debug)]
pub struct DeleteBucketWebsiteOutput {}

/// GetBucketNotificationConfiguration の結果
#[derive(Debug)]
pub struct GetBucketNotificationConfigurationOutput {
    pub topic_configurations: Vec<TopicConfiguration>,
    pub queue_configurations: Vec<QueueConfiguration>,
    pub lambda_function_configurations: Vec<LambdaFunctionConfiguration>,
    pub event_bridge_configuration: Option<EventBridgeConfiguration>,
}

/// PutBucketNotificationConfiguration の結果
#[derive(Debug)]
pub struct PutBucketNotificationConfigurationOutput {}

/// GetObjectTagging の結果
#[derive(Debug)]
pub struct GetObjectTaggingOutput {
    pub version_id: Option<String>,
    pub tag_set: Vec<Tag>,
}

/// PutObjectTagging の結果
#[derive(Debug)]
pub struct PutObjectTaggingOutput {
    pub version_id: Option<String>,
}

/// DeleteObjectTagging の結果
#[derive(Debug)]
pub struct DeleteObjectTaggingOutput {
    pub version_id: Option<String>,
}

/// GetPublicAccessBlock の結果
#[derive(Debug)]
pub struct GetPublicAccessBlockOutput {
    pub block_public_acls: Option<bool>,
    pub ignore_public_acls: Option<bool>,
    pub block_public_policy: Option<bool>,
    pub restrict_public_buckets: Option<bool>,
}

/// PutPublicAccessBlock の結果
#[derive(Debug)]
pub struct PutPublicAccessBlockOutput {}

/// DeletePublicAccessBlock の結果
#[derive(Debug)]
pub struct DeletePublicAccessBlockOutput {}

/// GetBucketPolicy の結果
#[derive(Debug)]
pub struct GetBucketPolicyOutput {
    /// バケットポリシーの JSON 文字列
    pub policy: Option<String>,
}

/// PutBucketPolicy の結果
#[derive(Debug)]
pub struct PutBucketPolicyOutput {}

/// DeleteBucketPolicy の結果
#[derive(Debug)]
pub struct DeleteBucketPolicyOutput {}

/// ListParts の結果
#[derive(Debug)]
pub struct ListPartsOutput {
    pub bucket: Option<String>,
    pub key: Option<String>,
    pub upload_id: Option<String>,
    pub part_number_marker: Option<i32>,
    pub next_part_number_marker: Option<i32>,
    pub max_parts: Option<i32>,
    pub is_truncated: Option<bool>,
    pub parts: Option<Vec<Part>>,
    pub storage_class: Option<StorageClass>,
}

/// アップロード済みパート
#[derive(Debug, Clone)]
pub struct Part {
    pub part_number: Option<i32>,
    pub last_modified: Option<SystemTime>,
    pub e_tag: Option<String>,
    pub size: Option<i64>,
}

/// ListMultipartUploads の結果
#[derive(Debug)]
pub struct ListMultipartUploadsOutput {
    pub bucket: Option<String>,
    pub key_marker: Option<String>,
    pub upload_id_marker: Option<String>,
    pub next_key_marker: Option<String>,
    pub next_upload_id_marker: Option<String>,
    pub prefix: Option<String>,
    pub delimiter: Option<String>,
    pub max_uploads: Option<i32>,
    pub is_truncated: Option<bool>,
    pub uploads: Option<Vec<MultipartUpload>>,
    pub common_prefixes: Option<Vec<CommonPrefix>>,
    /// `<EncodingType>` 要素 (url)
    pub encoding_type: Option<EncodingType>,
    /// `x-amz-request-charged`
    pub request_charged: Option<String>,
}

/// 進行中のマルチパートアップロード
#[derive(Debug, Clone)]
pub struct MultipartUpload {
    pub upload_id: Option<String>,
    pub key: Option<String>,
    pub initiated: Option<SystemTime>,
    pub storage_class: Option<StorageClass>,
}

/// GetBucketEncryption の結果
#[derive(Debug)]
pub struct GetBucketEncryptionOutput {
    pub server_side_encryption_configuration: Option<ServerSideEncryptionConfiguration>,
}

/// PutBucketEncryption の結果
#[derive(Debug)]
pub struct PutBucketEncryptionOutput {}

/// DeleteBucketEncryption の結果
#[derive(Debug)]
pub struct DeleteBucketEncryptionOutput {}

/// GetBucketCors の結果
#[derive(Debug)]
pub struct GetBucketCorsOutput {
    pub cors_rules: Option<Vec<CorsRule>>,
}

/// PutBucketCors の結果
#[derive(Debug)]
pub struct PutBucketCorsOutput {}

/// DeleteBucketCors の結果
#[derive(Debug)]
pub struct DeleteBucketCorsOutput {}

/// GetBucketLifecycleConfiguration の結果
#[derive(Debug)]
pub struct GetBucketLifecycleConfigurationOutput {
    pub rules: Vec<LifecycleRule>,
}

/// PutBucketLifecycleConfiguration の結果
#[derive(Debug)]
pub struct PutBucketLifecycleConfigurationOutput {}

/// DeleteBucketLifecycleConfiguration の結果
#[derive(Debug)]
pub struct DeleteBucketLifecycleOutput {}
