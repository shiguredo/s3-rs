use std::time::SystemTime;

use super::enums::{ExpirationStatus, StorageClass};
use crate::error::Error;

/// 削除対象のオブジェクト識別子
#[derive(Debug, Clone)]
pub struct ObjectIdentifier {
    pub key: String,
    pub version_id: Option<String>,
    /// 削除対象オブジェクトの ETag (Conditional Delete 用)
    pub e_tag: Option<String>,
    /// 削除対象オブジェクトの最終更新時刻 (Conditional Delete 用)
    pub last_modified_time: Option<SystemTime>,
    /// 削除対象オブジェクトのサイズ (Conditional Delete 用)
    pub size: Option<i64>,
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
    pub checksum_crc32: Option<String>,
    pub checksum_crc32_c: Option<String>,
    pub checksum_crc64_nvme: Option<String>,
    pub checksum_sha1: Option<String>,
    pub checksum_sha256: Option<String>,
}

/// マルチパートアップロードの完了情報
#[derive(Debug, Clone)]
pub struct CompletedMultipartUpload {
    pub parts: Option<Vec<CompletedPart>>,
}

/// CreateBucket のリクエストボディ
///
/// バケット作成時のリージョン制約を指定する。
/// `us-east-1` 以外のリージョンにバケットを作成する場合は `location_constraint` の指定が必要。
#[derive(Debug, Clone)]
pub struct CreateBucketConfiguration {
    pub location_constraint: Option<String>,
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

/// Object Ownership ルール
#[derive(Debug, Clone)]
pub struct OwnershipControlsRule {
    /// "BucketOwnerEnforced", "BucketOwnerPreferred", "ObjectWriter"
    pub object_ownership: String,
}

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
    /// ルール ID を設定する (最大 255 文字)
    ///
    /// 長さ制限は `build()` で検証される。
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
    ///
    /// `allowed_methods` / `allowed_origins` が空の場合、または `id` が 255 文字を超える場合は
    /// `Error::InvalidInput` を返す。
    pub fn build(self) -> Result<CorsRule, Error> {
        if self.allowed_methods.is_empty() {
            return Err(Error::InvalidInput(
                "allowed_methods must not be empty".to_string(),
            ));
        }
        if self.allowed_origins.is_empty() {
            return Err(Error::InvalidInput(
                "allowed_origins must not be empty".to_string(),
            ));
        }
        if let Some(ref id) = self.id
            && id.len() > 255
        {
            return Err(Error::InvalidInput(format!(
                "CorsRule ID must not be longer than 255 characters, got {}",
                id.len()
            )));
        }

        Ok(CorsRule {
            id: self.id,
            allowed_headers: self.allowed_headers,
            allowed_methods: self.allowed_methods,
            allowed_origins: self.allowed_origins,
            expose_headers: self.expose_headers,
            max_age_seconds: self.max_age_seconds,
        })
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

// -------------------------------------------------------
// ライフサイクル設定
// -------------------------------------------------------

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
