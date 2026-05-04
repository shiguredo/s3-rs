use std::fmt;
use std::time::SystemTime;

use crate::error::Error;

/// IMF-fixdate (RFC 9110 Section 5.6.7) フォーマットの文字列を検証する
///
/// 形式: `Day, DD Mon YYYY HH:MM:SS GMT` (29 バイト)
///
/// ```
/// use shiguredo_s3::types::validate_imf_fixdate;
///
/// assert!(validate_imf_fixdate("Thu, 01 Jan 1970 00:00:00 GMT").is_ok());
/// assert!(validate_imf_fixdate("invalid").is_err());
/// ```
pub fn validate_imf_fixdate(s: &str) -> Result<(), Error> {
    // IMF-fixdate は ASCII のみで構成される
    if !s.is_ascii() {
        return Err(Error::InvalidInput(
            "IMF-fixdate must be ASCII only".to_string(),
        ));
    }

    let bytes = s.as_bytes();
    // 長さチェック: "Day, DD Mon YYYY HH:MM:SS GMT" = 29 バイト
    if bytes.len() != 29 {
        return Err(Error::InvalidInput(format!(
            "IMF-fixdate must be 29 bytes, got {}",
            bytes.len()
        )));
    }

    // 以降は ASCII 確認済みのため、バイトインデックスと文字インデックスが一致する
    let weekday = &s[..3];
    if !crate::datetime::WEEKDAY_NAMES.contains(&weekday) {
        return Err(Error::InvalidInput(format!("invalid weekday: {weekday}")));
    }

    // 区切り文字チェック
    if &s[3..5] != ", " {
        return Err(Error::InvalidInput(
            "expected ', ' after weekday".to_string(),
        ));
    }

    // 月チェック (8..11)
    let month = &s[8..11];
    if !crate::datetime::MONTH_NAMES.contains(&month) {
        return Err(Error::InvalidInput(format!("invalid month: {month}")));
    }

    // GMT チェック (末尾)
    if !s.ends_with(" GMT") {
        return Err(Error::InvalidInput(
            "IMF-fixdate must end with ' GMT'".to_string(),
        ));
    }

    Ok(())
}

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
    pub checksum_crc32c: Option<String>,
    /// CRC64NVME チェックサム (checksum_mode=ENABLED 時)
    pub checksum_crc64nvme: Option<String>,
    /// SHA1 チェックサム (checksum_mode=ENABLED 時)
    pub checksum_sha1: Option<String>,
    /// SHA256 チェックサム (checksum_mode=ENABLED 時)
    pub checksum_sha256: Option<String>,
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
    pub checksum_crc32c: Option<String>,
    /// CRC64NVME チェックサム (checksum_mode=ENABLED 時)
    pub checksum_crc64nvme: Option<String>,
    /// SHA1 チェックサム (checksum_mode=ENABLED 時)
    pub checksum_sha1: Option<String>,
    /// SHA256 チェックサム (checksum_mode=ENABLED 時)
    pub checksum_sha256: Option<String>,
}

/// PutObject の結果
#[derive(Debug)]
pub struct PutObjectOutput {
    pub e_tag: Option<String>,
    /// オブジェクトのバージョン ID (バージョニング有効時)
    pub version_id: Option<String>,
}

/// DeleteObject の結果
#[derive(Debug)]
pub struct DeleteObjectOutput {
    pub delete_marker: Option<bool>,
    pub version_id: Option<String>,
}

/// CreateMultipartUpload の結果
#[derive(Debug)]
pub struct CreateMultipartUploadOutput {
    pub bucket: Option<String>,
    pub key: Option<String>,
    pub upload_id: Option<String>,
}

/// UploadPart の結果
#[derive(Debug)]
pub struct UploadPartOutput {
    pub e_tag: Option<String>,
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
    /// `<ChecksumType>` 要素 (issue 0064 後続で型化検討)
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

/// 削除対象のオブジェクト識別子
#[derive(Debug, Clone)]
pub struct ObjectIdentifier {
    pub key: String,
    pub version_id: Option<String>,
}

/// `DeleteObjects` のリクエストボディ
///
/// AWS S3 API の `<Delete>` 要素 (`Object` の配列と任意の `Quiet`) に対応する。
/// aws-sdk-rust の `aws_sdk_s3::types::Delete` と同じ構造。
#[derive(Debug, Clone, Default)]
pub struct Delete {
    /// 削除対象オブジェクトの一覧
    pub objects: Vec<ObjectIdentifier>,
    /// quiet モードを有効にすると、エラーのあったオブジェクトのみレスポンスに含まれる
    pub quiet: Option<bool>,
}

impl Delete {
    pub fn builder() -> DeleteBuilder {
        DeleteBuilder::default()
    }
}

/// `Delete` のビルダー (aws-sdk-rust 互換)
#[derive(Debug, Clone, Default)]
pub struct DeleteBuilder {
    objects: Vec<ObjectIdentifier>,
    quiet: Option<bool>,
}

impl DeleteBuilder {
    /// 削除対象オブジェクトを追加する
    pub fn objects(mut self, object: ObjectIdentifier) -> Self {
        self.objects.push(object);
        self
    }

    /// 削除対象オブジェクトの配列を一括設定する (`set_*` バリアント)
    pub fn set_objects(mut self, objects: Vec<ObjectIdentifier>) -> Self {
        self.objects = objects;
        self
    }

    /// quiet モードを設定する
    pub fn quiet(mut self, quiet: bool) -> Self {
        self.quiet = Some(quiet);
        self
    }

    /// quiet を Option で設定する (`set_*` バリアント)
    pub fn set_quiet(mut self, quiet: Option<bool>) -> Self {
        self.quiet = quiet;
        self
    }

    /// `Delete` を構築する
    pub fn build(self) -> Delete {
        Delete {
            objects: self.objects,
            quiet: self.quiet,
        }
    }
}

/// マルチパートアップロードの完了済みパート
#[derive(Debug, Clone)]
pub struct CompletedPart {
    pub e_tag: Option<String>,
    pub part_number: Option<i32>,
}

/// マルチパートアップロードの完了情報
#[derive(Debug, Clone)]
pub struct CompletedMultipartUpload {
    pub parts: Option<Vec<CompletedPart>>,
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
}

/// CreateBucket のリクエストボディ
///
/// バケット作成時のリージョン制約を指定する。
/// `us-east-1` 以外のリージョンにバケットを作成する場合は `location_constraint` の指定が必要。
#[derive(Debug, Clone)]
pub struct CreateBucketConfiguration {
    pub location_constraint: Option<String>,
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

/// タグ (キーと値のペア)
#[derive(Debug, Clone)]
pub struct Tag {
    pub key: String,
    pub value: String,
}

/// タグセット
///
/// PutBucketTagging / PutObjectTagging で使用する。
/// aws-sdk-rust の `Tagging` 型に対応する。
#[derive(Debug, Clone)]
pub struct Tagging {
    pub tag_set: Vec<Tag>,
}

impl Tagging {
    /// Tagging を構築するビルダーを返す
    pub fn builder() -> TaggingBuilder {
        TaggingBuilder::default()
    }
}

/// Tagging のビルダー
#[derive(Debug, Clone, Default)]
pub struct TaggingBuilder {
    tag_set: Vec<Tag>,
}

impl TaggingBuilder {
    /// タグを追加する
    pub fn tag_set(mut self, tag: Tag) -> Self {
        self.tag_set.push(tag);
        self
    }

    /// タグセットを一括設定する
    pub fn set_tag_set(mut self, tag_set: Vec<Tag>) -> Self {
        self.tag_set = tag_set;
        self
    }

    /// Tagging を構築する
    pub fn build(self) -> Tagging {
        Tagging {
            tag_set: self.tag_set,
        }
    }
}

/// PutBucketTagging の結果
#[derive(Debug)]
pub struct PutBucketTaggingOutput {}

/// DeleteBucketTagging の結果
#[derive(Debug)]
pub struct DeleteBucketTaggingOutput {}

/// Object Lock のリーガルホールド状態
#[derive(Debug, Clone)]
pub struct ObjectLockLegalHold {
    /// "ON" または "OFF"
    pub status: String,
}

/// Object Lock のリテンション
#[derive(Debug, Clone)]
pub struct ObjectLockRetention {
    /// "GOVERNANCE" または "COMPLIANCE"
    pub mode: String,
    /// 保持期限 (ISO 8601 形式)
    pub retain_until_date: String,
}

/// Object Lock のデフォルトリテンション
#[derive(Debug, Clone)]
pub struct DefaultRetention {
    /// "GOVERNANCE" または "COMPLIANCE"
    pub mode: Option<String>,
    /// 保持日数
    pub days: Option<i32>,
    /// 保持年数
    pub years: Option<i32>,
}

/// Object Lock ルール
#[derive(Debug, Clone)]
pub struct ObjectLockRule {
    pub default_retention: Option<DefaultRetention>,
}

/// Object Lock 設定
#[derive(Debug, Clone)]
pub struct ObjectLockConfiguration {
    /// "Enabled"
    pub object_lock_enabled: Option<String>,
    pub rule: Option<ObjectLockRule>,
}

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

/// Object Ownership ルール
#[derive(Debug, Clone)]
pub struct OwnershipControlsRule {
    /// "BucketOwnerEnforced", "BucketOwnerPreferred", "ObjectWriter"
    pub object_ownership: String,
}

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

/// インデックスドキュメント
#[derive(Debug, Clone)]
pub struct IndexDocument {
    pub suffix: String,
}

/// エラードキュメント
#[derive(Debug, Clone)]
pub struct ErrorDocument {
    pub key: String,
}

/// 全リクエストのリダイレクト先
#[derive(Debug, Clone)]
pub struct RedirectAllRequestsTo {
    pub host_name: String,
    pub protocol: Option<String>,
}

/// ルーティングルールの条件
#[derive(Debug, Clone)]
pub struct RoutingRuleCondition {
    pub http_error_code_returned_equals: Option<String>,
    pub key_prefix_equals: Option<String>,
}

/// ルーティングルールのリダイレクト先
#[derive(Debug, Clone)]
pub struct RoutingRuleRedirect {
    pub host_name: Option<String>,
    pub http_redirect_code: Option<String>,
    pub protocol: Option<String>,
    pub replace_key_prefix_with: Option<String>,
    pub replace_key_with: Option<String>,
}

/// ルーティングルール
#[derive(Debug, Clone)]
pub struct RoutingRule {
    pub condition: Option<RoutingRuleCondition>,
    pub redirect: Option<RoutingRuleRedirect>,
}

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

/// 通知設定フィルタルール
#[derive(Debug, Clone)]
pub struct FilterRule {
    /// フィルタ名 ("prefix" または "suffix")
    pub name: String,
    /// フィルタ値
    pub value: String,
}

/// S3 キーフィルタ
#[derive(Debug, Clone)]
pub struct S3KeyFilter {
    pub filter_rules: Vec<FilterRule>,
}

/// 通知設定フィルタ
#[derive(Debug, Clone)]
pub struct NotificationConfigurationFilter {
    pub key: Option<S3KeyFilter>,
}

/// SNS トピック通知設定
#[derive(Debug, Clone)]
pub struct TopicConfiguration {
    pub id: Option<String>,
    pub topic_arn: String,
    pub events: Vec<String>,
    pub filter: Option<NotificationConfigurationFilter>,
}

/// SQS キュー通知設定
#[derive(Debug, Clone)]
pub struct QueueConfiguration {
    pub id: Option<String>,
    pub queue_arn: String,
    pub events: Vec<String>,
    pub filter: Option<NotificationConfigurationFilter>,
}

/// Lambda 関数通知設定
#[derive(Debug, Clone)]
pub struct LambdaFunctionConfiguration {
    pub id: Option<String>,
    pub lambda_function_arn: String,
    pub events: Vec<String>,
    pub filter: Option<NotificationConfigurationFilter>,
}

/// EventBridge 通知設定
#[derive(Debug, Clone)]
pub struct EventBridgeConfiguration {}

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
}

/// 進行中のマルチパートアップロード
#[derive(Debug, Clone)]
pub struct MultipartUpload {
    pub upload_id: Option<String>,
    pub key: Option<String>,
    pub initiated: Option<SystemTime>,
    pub storage_class: Option<StorageClass>,
}

// -------------------------------------------------------
// 暗号化設定
// -------------------------------------------------------

/// サーバサイド暗号化のデフォルト設定
///
/// aws-sdk-rust の `ServerSideEncryptionByDefault` 型に対応する。
#[derive(Debug, Clone)]
pub struct ServerSideEncryptionByDefault {
    /// 暗号化アルゴリズム (AES256, aws:kms, aws:kms:dsse)
    pub sse_algorithm: String,
    /// KMS マスターキー ID (SSE-KMS の場合)
    pub kms_master_key_id: Option<String>,
}

impl ServerSideEncryptionByDefault {
    /// ServerSideEncryptionByDefault を構築するビルダーを返す
    pub fn builder() -> ServerSideEncryptionByDefaultBuilder {
        ServerSideEncryptionByDefaultBuilder::default()
    }
}

/// ServerSideEncryptionByDefault のビルダー
#[derive(Debug, Clone, Default)]
pub struct ServerSideEncryptionByDefaultBuilder {
    sse_algorithm: Option<String>,
    kms_master_key_id: Option<String>,
}

impl ServerSideEncryptionByDefaultBuilder {
    /// 暗号化アルゴリズムを設定する (AES256, aws:kms, aws:kms:dsse)
    pub fn sse_algorithm(mut self, algorithm: impl Into<String>) -> Self {
        self.sse_algorithm = Some(algorithm.into());
        self
    }

    /// KMS マスターキー ID を設定する
    pub fn kms_master_key_id(mut self, key_id: impl Into<String>) -> Self {
        self.kms_master_key_id = Some(key_id.into());
        self
    }

    /// ServerSideEncryptionByDefault を構築する
    ///
    /// sse_algorithm が未設定の場合はデフォルトで空文字列になる。
    pub fn build(self) -> ServerSideEncryptionByDefault {
        ServerSideEncryptionByDefault {
            sse_algorithm: self.sse_algorithm.unwrap_or_default(),
            kms_master_key_id: self.kms_master_key_id,
        }
    }
}

/// サーバサイド暗号化ルール
///
/// aws-sdk-rust の `ServerSideEncryptionRule` 型に対応する。
#[derive(Debug, Clone)]
pub struct ServerSideEncryptionRule {
    /// デフォルト暗号化の適用設定
    pub apply_server_side_encryption_by_default: Option<ServerSideEncryptionByDefault>,
    /// S3 Bucket Key の有効/無効
    pub bucket_key_enabled: Option<bool>,
}

impl ServerSideEncryptionRule {
    /// ServerSideEncryptionRule を構築するビルダーを返す
    pub fn builder() -> ServerSideEncryptionRuleBuilder {
        ServerSideEncryptionRuleBuilder::default()
    }
}

/// ServerSideEncryptionRule のビルダー
#[derive(Debug, Clone, Default)]
pub struct ServerSideEncryptionRuleBuilder {
    apply_server_side_encryption_by_default: Option<ServerSideEncryptionByDefault>,
    bucket_key_enabled: Option<bool>,
}

impl ServerSideEncryptionRuleBuilder {
    /// デフォルト暗号化の適用設定を指定する
    pub fn apply_server_side_encryption_by_default(
        mut self,
        value: ServerSideEncryptionByDefault,
    ) -> Self {
        self.apply_server_side_encryption_by_default = Some(value);
        self
    }

    /// S3 Bucket Key の有効/無効を指定する
    pub fn bucket_key_enabled(mut self, enabled: bool) -> Self {
        self.bucket_key_enabled = Some(enabled);
        self
    }

    /// ServerSideEncryptionRule を構築する
    pub fn build(self) -> ServerSideEncryptionRule {
        ServerSideEncryptionRule {
            apply_server_side_encryption_by_default: self.apply_server_side_encryption_by_default,
            bucket_key_enabled: self.bucket_key_enabled,
        }
    }
}

/// サーバサイド暗号化設定
///
/// PutBucketEncryption で使用する。
/// aws-sdk-rust の `ServerSideEncryptionConfiguration` 型に対応する。
#[derive(Debug, Clone)]
pub struct ServerSideEncryptionConfiguration {
    pub rules: Vec<ServerSideEncryptionRule>,
}

impl ServerSideEncryptionConfiguration {
    /// ServerSideEncryptionConfiguration を構築するビルダーを返す
    pub fn builder() -> ServerSideEncryptionConfigurationBuilder {
        ServerSideEncryptionConfigurationBuilder::default()
    }
}

/// ServerSideEncryptionConfiguration のビルダー
#[derive(Debug, Clone, Default)]
pub struct ServerSideEncryptionConfigurationBuilder {
    rules: Vec<ServerSideEncryptionRule>,
}

impl ServerSideEncryptionConfigurationBuilder {
    /// 暗号化ルールを追加する
    pub fn rules(mut self, rule: ServerSideEncryptionRule) -> Self {
        self.rules.push(rule);
        self
    }

    /// 暗号化ルールを一括設定する
    pub fn set_rules(mut self, rules: Vec<ServerSideEncryptionRule>) -> Self {
        self.rules = rules;
        self
    }

    /// ServerSideEncryptionConfiguration を構築する
    pub fn build(self) -> ServerSideEncryptionConfiguration {
        ServerSideEncryptionConfiguration { rules: self.rules }
    }
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

// -------------------------------------------------------
// CORS 設定
// -------------------------------------------------------

/// CORS ルール
///
/// aws-sdk-rust の `CorsRule` 型に対応する。
/// `allowed_methods` と `allowed_origins` は必須フィールド。
#[derive(Debug, Clone)]
pub struct CorsRule {
    /// ルール ID (最大 255 文字)
    pub id: Option<String>,
    /// 許可するリクエストヘッダー
    pub allowed_headers: Option<Vec<String>>,
    /// 許可する HTTP メソッド (GET, PUT, HEAD, POST, DELETE)
    pub allowed_methods: Vec<String>,
    /// 許可するオリジン
    pub allowed_origins: Vec<String>,
    /// クライアントに公開するレスポンスヘッダー
    pub expose_headers: Option<Vec<String>>,
    /// プリフライトレスポンスのキャッシュ秒数
    pub max_age_seconds: Option<i32>,
}

impl CorsRule {
    /// CorsRule を構築するビルダーを返す
    pub fn builder() -> CorsRuleBuilder {
        CorsRuleBuilder::default()
    }
}

/// CorsRule のビルダー
#[derive(Debug, Clone, Default)]
pub struct CorsRuleBuilder {
    id: Option<String>,
    allowed_headers: Option<Vec<String>>,
    allowed_methods: Vec<String>,
    allowed_origins: Vec<String>,
    expose_headers: Option<Vec<String>>,
    max_age_seconds: Option<i32>,
}

impl CorsRuleBuilder {
    /// ルール ID を設定する
    pub fn id(mut self, id: impl Into<String>) -> Self {
        self.id = Some(id.into());
        self
    }

    /// 許可するリクエストヘッダーを追加する
    pub fn allowed_headers(mut self, header: impl Into<String>) -> Self {
        self.allowed_headers
            .get_or_insert_with(Vec::new)
            .push(header.into());
        self
    }

    /// 許可するリクエストヘッダーを一括設定する
    pub fn set_allowed_headers(mut self, headers: Option<Vec<String>>) -> Self {
        self.allowed_headers = headers;
        self
    }

    /// 許可する HTTP メソッドを追加する
    pub fn allowed_methods(mut self, method: impl Into<String>) -> Self {
        self.allowed_methods.push(method.into());
        self
    }

    /// 許可する HTTP メソッドを一括設定する
    pub fn set_allowed_methods(mut self, methods: Vec<String>) -> Self {
        self.allowed_methods = methods;
        self
    }

    /// 許可するオリジンを追加する
    pub fn allowed_origins(mut self, origin: impl Into<String>) -> Self {
        self.allowed_origins.push(origin.into());
        self
    }

    /// 許可するオリジンを一括設定する
    pub fn set_allowed_origins(mut self, origins: Vec<String>) -> Self {
        self.allowed_origins = origins;
        self
    }

    /// クライアントに公開するレスポンスヘッダーを追加する
    pub fn expose_headers(mut self, header: impl Into<String>) -> Self {
        self.expose_headers
            .get_or_insert_with(Vec::new)
            .push(header.into());
        self
    }

    /// クライアントに公開するレスポンスヘッダーを一括設定する
    pub fn set_expose_headers(mut self, headers: Option<Vec<String>>) -> Self {
        self.expose_headers = headers;
        self
    }

    /// プリフライトレスポンスのキャッシュ秒数を設定する
    pub fn max_age_seconds(mut self, seconds: i32) -> Self {
        self.max_age_seconds = Some(seconds);
        self
    }

    /// CorsRule を構築する
    pub fn build(self) -> CorsRule {
        CorsRule {
            id: self.id,
            allowed_headers: self.allowed_headers,
            allowed_methods: self.allowed_methods,
            allowed_origins: self.allowed_origins,
            expose_headers: self.expose_headers,
            max_age_seconds: self.max_age_seconds,
        }
    }
}

/// CORS 設定
///
/// PutBucketCors で使用する。
/// aws-sdk-rust の `CorsConfiguration` 型に対応する。
#[derive(Debug, Clone)]
pub struct CorsConfiguration {
    pub cors_rules: Vec<CorsRule>,
}

impl CorsConfiguration {
    /// CorsConfiguration を構築するビルダーを返す
    pub fn builder() -> CorsConfigurationBuilder {
        CorsConfigurationBuilder::default()
    }
}

/// CorsConfiguration のビルダー
#[derive(Debug, Clone, Default)]
pub struct CorsConfigurationBuilder {
    cors_rules: Vec<CorsRule>,
}

impl CorsConfigurationBuilder {
    /// CORS ルールを追加する
    pub fn cors_rules(mut self, rule: CorsRule) -> Self {
        self.cors_rules.push(rule);
        self
    }

    /// CORS ルールを一括設定する
    pub fn set_cors_rules(mut self, rules: Vec<CorsRule>) -> Self {
        self.cors_rules = rules;
        self
    }

    /// CorsConfiguration を構築する
    pub fn build(self) -> CorsConfiguration {
        CorsConfiguration {
            cors_rules: self.cors_rules,
        }
    }
}

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

// -------------------------------------------------------
// ライフサイクル設定
// -------------------------------------------------------

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

/// ライフサイクルルール
#[derive(Debug, Clone)]
pub struct LifecycleRule {
    /// ルール ID (最大 255 文字)
    pub id: Option<String>,
    /// ルールの有効/無効
    pub status: ExpirationStatus,
    /// ルール適用フィルタ
    pub filter: Option<LifecycleRuleFilter>,
    /// オブジェクトの失効設定
    pub expiration: Option<LifecycleExpiration>,
    /// ストレージクラス移行設定
    pub transitions: Option<Vec<Transition>>,
    /// 非カレントバージョンの移行設定
    pub noncurrent_version_transitions: Option<Vec<NoncurrentVersionTransition>>,
    /// 非カレントバージョンの失効設定
    pub noncurrent_version_expiration: Option<NoncurrentVersionExpiration>,
    /// 不完全マルチパートアップロードの自動中止設定
    pub abort_incomplete_multipart_upload: Option<AbortIncompleteMultipartUpload>,
}

/// ライフサイクルルールの有効/無効
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ExpirationStatus {
    Enabled,
    Disabled,
}

impl ExpirationStatus {
    /// S3 API の文字列表現を返す
    pub fn as_str(&self) -> &str {
        match self {
            Self::Enabled => "Enabled",
            Self::Disabled => "Disabled",
        }
    }
}

impl std::str::FromStr for ExpirationStatus {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "Enabled" => Ok(Self::Enabled),
            "Disabled" => Ok(Self::Disabled),
            _ => Err(Error::InvalidResponse(format!(
                "unknown ExpirationStatus: {s}"
            ))),
        }
    }
}

/// ライフサイクルルールのフィルタ
///
/// フィルタが空の場合はすべてのオブジェクトに適用される。
/// 複数条件を組み合わせるには `and` を使用する。
#[derive(Debug, Clone, Default)]
pub struct LifecycleRuleFilter {
    /// プレフィックスフィルタ
    pub prefix: Option<String>,
    /// タグフィルタ
    pub tag: Option<Tag>,
    /// 最小オブジェクトサイズ (バイト)
    pub object_size_greater_than: Option<i64>,
    /// 最大オブジェクトサイズ (バイト)
    pub object_size_less_than: Option<i64>,
    /// AND 条件
    pub and: Option<LifecycleRuleAndOperator>,
}

/// ライフサイクルルールの AND 条件
#[derive(Debug, Clone, Default)]
pub struct LifecycleRuleAndOperator {
    /// プレフィックス
    pub prefix: Option<String>,
    /// タグのリスト
    pub tags: Option<Vec<Tag>>,
    /// 最小オブジェクトサイズ (バイト)
    pub object_size_greater_than: Option<i64>,
    /// 最大オブジェクトサイズ (バイト)
    pub object_size_less_than: Option<i64>,
}

/// オブジェクトの失効設定
#[derive(Debug, Clone, Default)]
pub struct LifecycleExpiration {
    /// 失効日 (ISO 8601 形式)
    pub date: Option<String>,
    /// 作成後の経過日数
    pub days: Option<i32>,
    /// 期限切れオブジェクト削除マーカーを削除するか
    pub expired_object_delete_marker: Option<bool>,
}

/// ストレージクラス移行設定
#[derive(Debug, Clone, Default)]
pub struct Transition {
    /// 移行日 (ISO 8601 形式)
    pub date: Option<String>,
    /// 作成後の経過日数
    pub days: Option<i32>,
    /// 移行先ストレージクラス
    pub storage_class: Option<StorageClass>,
}

/// 非カレントバージョンの移行設定
#[derive(Debug, Clone, Default)]
pub struct NoncurrentVersionTransition {
    /// 非カレント状態の経過日数
    pub noncurrent_days: Option<i32>,
    /// 移行先ストレージクラス
    pub storage_class: Option<StorageClass>,
    /// 保持する非カレントバージョン数 (最大 100)
    pub newer_noncurrent_versions: Option<i32>,
}

/// 非カレントバージョンの失効設定
#[derive(Debug, Clone, Default)]
pub struct NoncurrentVersionExpiration {
    /// 非カレント状態の経過日数
    pub noncurrent_days: Option<i32>,
    /// 保持する非カレントバージョン数 (最大 100)
    pub newer_noncurrent_versions: Option<i32>,
}

/// 不完全マルチパートアップロードの自動中止設定
#[derive(Debug, Clone, Default)]
pub struct AbortIncompleteMultipartUpload {
    /// 初期化後の経過日数
    pub days_after_initiation: Option<i32>,
}

// -------------------------------------------------------
// 型付き enum (aws-sdk-rust 互換)
// -------------------------------------------------------
//
// 各 enum は以下の方針で実装する:
//
// - `#[non_exhaustive]` を付けて将来の値追加で網羅エラーを起こさないようにする
// - `Unknown(String)` variant により S3 互換ストレージが返す未知の値を保持する (前方互換性)
// - `as_str()` / `From<&str>` を提供する (aws-sdk-rust 互換)
// - variants は aws-sdk-rust の対応 enum と完全に揃える

/// チェックサムアルゴリズム
///
/// PutObject / UploadPart / CopyObject / DeleteObjects 等の `checksum_algorithm`、
/// および GetObject / HeadObject の `checksum_*` レスポンスヘッダー解析で使う。
///
/// shiguredo_s3 内部の署名計算で実際に使えるアルゴリズムは
/// `Crc32`, `Crc32C`, `Crc64Nvme`, `Sha1`, `Sha256` の 5 種で、
/// `Md5` / `Sha512` / `Xxhash128` / `Xxhash3` / `Xxhash64` を指定すると
/// `Error::InvalidInput` を返す。型としては aws-sdk-rust と揃え、
/// 将来の実装拡張に備える。
#[non_exhaustive]
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum ChecksumAlgorithm {
    Crc32,
    Crc32C,
    Crc64Nvme,
    Md5,
    Sha1,
    Sha256,
    Sha512,
    Xxhash128,
    Xxhash3,
    Xxhash64,
    /// 未知の値を保持する (前方互換性のため)
    Unknown(String),
}

impl ChecksumAlgorithm {
    pub fn as_str(&self) -> &str {
        match self {
            Self::Crc32 => "CRC32",
            Self::Crc32C => "CRC32C",
            Self::Crc64Nvme => "CRC64NVME",
            Self::Md5 => "MD5",
            Self::Sha1 => "SHA1",
            Self::Sha256 => "SHA256",
            Self::Sha512 => "SHA512",
            Self::Xxhash128 => "XXHASH128",
            Self::Xxhash3 => "XXHASH3",
            Self::Xxhash64 => "XXHASH64",
            Self::Unknown(s) => s.as_str(),
        }
    }
}

impl From<&str> for ChecksumAlgorithm {
    fn from(s: &str) -> Self {
        match s {
            "CRC32" => Self::Crc32,
            "CRC32C" => Self::Crc32C,
            "CRC64NVME" => Self::Crc64Nvme,
            "MD5" => Self::Md5,
            "SHA1" => Self::Sha1,
            "SHA256" => Self::Sha256,
            "SHA512" => Self::Sha512,
            "XXHASH128" => Self::Xxhash128,
            "XXHASH3" => Self::Xxhash3,
            "XXHASH64" => Self::Xxhash64,
            _ => Self::Unknown(s.to_string()),
        }
    }
}

impl fmt::Display for ChecksumAlgorithm {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

/// チェックサム取得モード
///
/// GetObject / HeadObject の `checksum_mode` で使う。
#[non_exhaustive]
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum ChecksumMode {
    Enabled,
    /// 未知の値を保持する (前方互換性のため)
    Unknown(String),
}

impl ChecksumMode {
    pub fn as_str(&self) -> &str {
        match self {
            Self::Enabled => "ENABLED",
            Self::Unknown(s) => s.as_str(),
        }
    }
}

impl From<&str> for ChecksumMode {
    fn from(s: &str) -> Self {
        match s {
            "ENABLED" => Self::Enabled,
            _ => Self::Unknown(s.to_string()),
        }
    }
}

impl fmt::Display for ChecksumMode {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

/// サーバーサイド暗号化アルゴリズム
///
/// PutObject / CopyObject / CreateMultipartUpload の `server_side_encryption` で使う。
#[non_exhaustive]
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum ServerSideEncryption {
    Aes256,
    AwsFsx,
    AwsKms,
    AwsKmsDsse,
    /// 未知の値を保持する (前方互換性のため)
    Unknown(String),
}

impl ServerSideEncryption {
    pub fn as_str(&self) -> &str {
        match self {
            Self::Aes256 => "AES256",
            Self::AwsFsx => "aws:fsx",
            Self::AwsKms => "aws:kms",
            Self::AwsKmsDsse => "aws:kms:dsse",
            Self::Unknown(s) => s.as_str(),
        }
    }
}

impl From<&str> for ServerSideEncryption {
    fn from(s: &str) -> Self {
        match s {
            "AES256" => Self::Aes256,
            "aws:fsx" => Self::AwsFsx,
            "aws:kms" => Self::AwsKms,
            "aws:kms:dsse" => Self::AwsKmsDsse,
            _ => Self::Unknown(s.to_string()),
        }
    }
}

impl fmt::Display for ServerSideEncryption {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

/// 既定 ACL
///
/// PutObject / CopyObject / CreateMultipartUpload / CreateBucket の `acl` で使う。
#[non_exhaustive]
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum ObjectCannedAcl {
    AuthenticatedRead,
    AwsExecRead,
    BucketOwnerFullControl,
    BucketOwnerRead,
    Private,
    PublicRead,
    PublicReadWrite,
    /// 未知の値を保持する (前方互換性のため)
    Unknown(String),
}

impl ObjectCannedAcl {
    pub fn as_str(&self) -> &str {
        match self {
            Self::AuthenticatedRead => "authenticated-read",
            Self::AwsExecRead => "aws-exec-read",
            Self::BucketOwnerFullControl => "bucket-owner-full-control",
            Self::BucketOwnerRead => "bucket-owner-read",
            Self::Private => "private",
            Self::PublicRead => "public-read",
            Self::PublicReadWrite => "public-read-write",
            Self::Unknown(s) => s.as_str(),
        }
    }
}

impl From<&str> for ObjectCannedAcl {
    fn from(s: &str) -> Self {
        match s {
            "authenticated-read" => Self::AuthenticatedRead,
            "aws-exec-read" => Self::AwsExecRead,
            "bucket-owner-full-control" => Self::BucketOwnerFullControl,
            "bucket-owner-read" => Self::BucketOwnerRead,
            "private" => Self::Private,
            "public-read" => Self::PublicRead,
            "public-read-write" => Self::PublicReadWrite,
            _ => Self::Unknown(s.to_string()),
        }
    }
}

impl fmt::Display for ObjectCannedAcl {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

/// オブジェクトのストレージクラス
///
/// PutObject / CopyObject / CreateMultipartUpload の `storage_class`、
/// および ListObjectsV2 / ListObjectVersions / GetObject 等のレスポンスで使う。
///
/// variants は aws-sdk-rust の `StorageClass` と完全に揃える。
#[non_exhaustive]
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum StorageClass {
    DeepArchive,
    ExpressOnezone,
    FsxOntap,
    FsxOpenzfs,
    Glacier,
    GlacierIr,
    IntelligentTiering,
    OnezoneIa,
    Outposts,
    ReducedRedundancy,
    Snow,
    Standard,
    StandardIa,
    /// 未知の値を保持する (前方互換性のため)
    Unknown(String),
}

impl StorageClass {
    pub fn as_str(&self) -> &str {
        match self {
            Self::DeepArchive => "DEEP_ARCHIVE",
            Self::ExpressOnezone => "EXPRESS_ONEZONE",
            Self::FsxOntap => "FSX_ONTAP",
            Self::FsxOpenzfs => "FSX_OPENZFS",
            Self::Glacier => "GLACIER",
            Self::GlacierIr => "GLACIER_IR",
            Self::IntelligentTiering => "INTELLIGENT_TIERING",
            Self::OnezoneIa => "ONEZONE_IA",
            Self::Outposts => "OUTPOSTS",
            Self::ReducedRedundancy => "REDUCED_REDUNDANCY",
            Self::Snow => "SNOW",
            Self::Standard => "STANDARD",
            Self::StandardIa => "STANDARD_IA",
            Self::Unknown(s) => s.as_str(),
        }
    }
}

impl From<&str> for StorageClass {
    fn from(s: &str) -> Self {
        match s {
            "DEEP_ARCHIVE" => Self::DeepArchive,
            "EXPRESS_ONEZONE" => Self::ExpressOnezone,
            "FSX_ONTAP" => Self::FsxOntap,
            "FSX_OPENZFS" => Self::FsxOpenzfs,
            "GLACIER" => Self::Glacier,
            "GLACIER_IR" => Self::GlacierIr,
            "INTELLIGENT_TIERING" => Self::IntelligentTiering,
            "ONEZONE_IA" => Self::OnezoneIa,
            "OUTPOSTS" => Self::Outposts,
            "REDUCED_REDUNDANCY" => Self::ReducedRedundancy,
            "SNOW" => Self::Snow,
            "STANDARD" => Self::Standard,
            "STANDARD_IA" => Self::StandardIa,
            _ => Self::Unknown(s.to_string()),
        }
    }
}

impl fmt::Display for StorageClass {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

/// CopyObject のメタデータ転送ディレクティブ
#[non_exhaustive]
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum MetadataDirective {
    Copy,
    Replace,
    /// 未知の値を保持する (前方互換性のため)
    Unknown(String),
}

impl MetadataDirective {
    pub fn as_str(&self) -> &str {
        match self {
            Self::Copy => "COPY",
            Self::Replace => "REPLACE",
            Self::Unknown(s) => s.as_str(),
        }
    }
}

impl From<&str> for MetadataDirective {
    fn from(s: &str) -> Self {
        match s {
            "COPY" => Self::Copy,
            "REPLACE" => Self::Replace,
            _ => Self::Unknown(s.to_string()),
        }
    }
}

impl fmt::Display for MetadataDirective {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

/// CopyObject のタグ転送ディレクティブ
#[non_exhaustive]
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum TaggingDirective {
    Copy,
    Replace,
    /// 未知の値を保持する (前方互換性のため)
    Unknown(String),
}

impl TaggingDirective {
    pub fn as_str(&self) -> &str {
        match self {
            Self::Copy => "COPY",
            Self::Replace => "REPLACE",
            Self::Unknown(s) => s.as_str(),
        }
    }
}

impl From<&str> for TaggingDirective {
    fn from(s: &str) -> Self {
        match s {
            "COPY" => Self::Copy,
            "REPLACE" => Self::Replace,
            _ => Self::Unknown(s.to_string()),
        }
    }
}

impl fmt::Display for TaggingDirective {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

/// レスポンスのオブジェクトキーエンコーディング種別
///
/// ListObjectsV2 / ListObjectVersions / ListMultipartUploads の `encoding_type` で使う。
#[non_exhaustive]
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum EncodingType {
    Url,
    /// 未知の値を保持する (前方互換性のため)
    Unknown(String),
}

impl EncodingType {
    pub fn as_str(&self) -> &str {
        match self {
            Self::Url => "url",
            Self::Unknown(s) => s.as_str(),
        }
    }
}

impl From<&str> for EncodingType {
    fn from(s: &str) -> Self {
        match s {
            "url" => Self::Url,
            _ => Self::Unknown(s.to_string()),
        }
    }
}

impl fmt::Display for EncodingType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}
