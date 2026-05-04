use crate::api::{
    AbortMultipartUploadFluentBuilder, CompleteMultipartUploadFluentBuilder,
    CopyObjectFluentBuilder, CreateBucketFluentBuilder, CreateMultipartUploadFluentBuilder,
    DeleteBucketCorsFluentBuilder, DeleteBucketEncryptionFluentBuilder, DeleteBucketFluentBuilder,
    DeleteBucketLifecycleFluentBuilder, DeleteBucketOwnershipControlsFluentBuilder,
    DeleteBucketPolicyFluentBuilder, DeleteBucketTaggingFluentBuilder,
    DeleteBucketWebsiteFluentBuilder, DeleteObjectFluentBuilder, DeleteObjectTaggingFluentBuilder,
    DeleteObjectsFluentBuilder, DeletePublicAccessBlockFluentBuilder, GetBucketCorsFluentBuilder,
    GetBucketEncryptionFluentBuilder, GetBucketLifecycleConfigurationFluentBuilder,
    GetBucketNotificationConfigurationFluentBuilder, GetBucketOwnershipControlsFluentBuilder,
    GetBucketPolicyFluentBuilder, GetBucketTaggingFluentBuilder, GetBucketVersioningFluentBuilder,
    GetBucketWebsiteFluentBuilder, GetObjectFluentBuilder, GetObjectLegalHoldFluentBuilder,
    GetObjectLockConfigurationFluentBuilder, GetObjectRetentionFluentBuilder,
    GetObjectTaggingFluentBuilder, GetPublicAccessBlockFluentBuilder, HeadBucketFluentBuilder,
    HeadObjectFluentBuilder, ListBucketsFluentBuilder, ListMultipartUploadsFluentBuilder,
    ListObjectVersionsFluentBuilder, ListObjectsV2FluentBuilder, ListPartsFluentBuilder,
    PutBucketCorsFluentBuilder, PutBucketEncryptionFluentBuilder,
    PutBucketLifecycleConfigurationFluentBuilder, PutBucketNotificationConfigurationFluentBuilder,
    PutBucketOwnershipControlsFluentBuilder, PutBucketPolicyFluentBuilder,
    PutBucketTaggingFluentBuilder, PutBucketVersioningFluentBuilder, PutBucketWebsiteFluentBuilder,
    PutObjectFluentBuilder, PutObjectLegalHoldFluentBuilder,
    PutObjectLockConfigurationFluentBuilder, PutObjectRetentionFluentBuilder,
    PutObjectTaggingFluentBuilder, PutPublicAccessBlockFluentBuilder, UploadPartCopyFluentBuilder,
    UploadPartFluentBuilder,
};
use crate::credential::Credentials;
use crate::error::Error;

/// S3 クライアントの設定
///
/// aws-sdk-rust の `aws_sdk_s3::Config` と同等の API を提供する。
#[derive(Debug, Clone)]
pub struct Config {
    /// AWS リージョン (例: "ap-northeast-1")
    pub(crate) region: String,
    /// AWS クレデンシャル
    pub(crate) credentials_provider: Credentials,
    /// カスタムエンドポイント (None の場合は AWS デフォルトを使用する)
    pub(crate) endpoint: Option<String>,
    /// パススタイルのアクセスを使用する (MinIO 等の S3 互換サービス向け)
    pub(crate) force_path_style: bool,
    /// TLS 証明書の検証を無視する (テスト環境向け)
    pub(crate) ignore_cert_check: bool,
}

impl Config {
    /// `ConfigBuilder` を返す
    pub fn builder() -> ConfigBuilder {
        ConfigBuilder::default()
    }

    /// リージョンを返す
    pub fn region(&self) -> &str {
        &self.region
    }

    /// クレデンシャルを返す
    pub fn credentials_provider(&self) -> &Credentials {
        &self.credentials_provider
    }

    /// エンドポイントを返す
    pub fn endpoint(&self) -> Option<&str> {
        self.endpoint.as_deref()
    }

    /// パススタイルアクセスを使用するかを返す
    pub fn force_path_style(&self) -> bool {
        self.force_path_style
    }

    /// TLS 証明書の検証を無視するかを返す
    pub fn ignore_cert_check(&self) -> bool {
        self.ignore_cert_check
    }
}

/// `Config` のビルダー
#[derive(Debug, Clone, Default)]
pub struct ConfigBuilder {
    region: Option<String>,
    credentials_provider: Option<Credentials>,
    endpoint: Option<String>,
    force_path_style: bool,
    ignore_cert_check: bool,
}

impl ConfigBuilder {
    /// AWS リージョンを設定する (必須)
    pub fn region(mut self, region: impl Into<String>) -> Self {
        self.region = Some(region.into());
        self
    }

    /// AWS クレデンシャルを設定する (必須)
    pub fn credentials_provider(mut self, credentials_provider: Credentials) -> Self {
        self.credentials_provider = Some(credentials_provider);
        self
    }

    /// カスタムエンドポイントを設定する
    pub fn endpoint(mut self, endpoint: impl Into<String>) -> Self {
        self.endpoint = Some(endpoint.into());
        self
    }

    /// パススタイルのアクセスを使用する (MinIO 等の S3 互換サービス向け)
    pub fn force_path_style(mut self, force_path_style: bool) -> Self {
        self.force_path_style = force_path_style;
        self
    }

    /// TLS 証明書の検証を無視する (テスト環境向け、shiguredo_s3 独自フィールド)
    pub fn ignore_cert_check(mut self, ignore_cert_check: bool) -> Self {
        self.ignore_cert_check = ignore_cert_check;
        self
    }

    /// `Config` を構築する
    ///
    /// region と credentials_provider が未設定の場合はエラーを返す
    pub fn build(self) -> Result<Config, Error> {
        let region = self
            .region
            .ok_or_else(|| Error::InvalidInput("region is required".to_string()))?;
        let credentials_provider = self
            .credentials_provider
            .ok_or_else(|| Error::InvalidInput("credentials_provider is required".to_string()))?;

        Ok(Config {
            region,
            credentials_provider,
            endpoint: self.endpoint,
            force_path_style: self.force_path_style,
            ignore_cert_check: self.ignore_cert_check,
        })
    }
}

/// S3 クライアント (Sans I/O)
///
/// 設定を保持し、各 API の Fluent Builder を生成する。
/// I/O は行わない。aws-sdk-rust の `aws_sdk_s3::Client` と同等の API を提供する。
#[derive(Debug, Clone)]
pub struct Client {
    pub(crate) config: Config,
}

impl Client {
    /// 設定から S3 クライアントを作成する
    ///
    /// aws-sdk-rust の `Client::from_conf(Config)` と同等。
    pub fn from_conf(config: Config) -> Self {
        Self { config }
    }

    // -------------------------------------------------------
    // Fluent Builder ファクトリメソッド
    // -------------------------------------------------------

    pub fn get_object(&self) -> GetObjectFluentBuilder<'_> {
        GetObjectFluentBuilder::new(self)
    }

    pub fn head_object(&self) -> HeadObjectFluentBuilder<'_> {
        HeadObjectFluentBuilder::new(self)
    }

    pub fn put_object(&self) -> PutObjectFluentBuilder<'_> {
        PutObjectFluentBuilder::new(self)
    }

    pub fn delete_object(&self) -> DeleteObjectFluentBuilder<'_> {
        DeleteObjectFluentBuilder::new(self)
    }

    pub fn create_multipart_upload(&self) -> CreateMultipartUploadFluentBuilder<'_> {
        CreateMultipartUploadFluentBuilder::new(self)
    }

    pub fn upload_part(&self) -> UploadPartFluentBuilder<'_> {
        UploadPartFluentBuilder::new(self)
    }

    pub fn upload_part_copy(&self) -> UploadPartCopyFluentBuilder<'_> {
        UploadPartCopyFluentBuilder::new(self)
    }

    pub fn complete_multipart_upload(&self) -> CompleteMultipartUploadFluentBuilder<'_> {
        CompleteMultipartUploadFluentBuilder::new(self)
    }

    pub fn abort_multipart_upload(&self) -> AbortMultipartUploadFluentBuilder<'_> {
        AbortMultipartUploadFluentBuilder::new(self)
    }

    pub fn list_objects_v2(&self) -> ListObjectsV2FluentBuilder<'_> {
        ListObjectsV2FluentBuilder::new(self)
    }

    pub fn list_object_versions(&self) -> ListObjectVersionsFluentBuilder<'_> {
        ListObjectVersionsFluentBuilder::new(self)
    }

    pub fn copy_object(&self) -> CopyObjectFluentBuilder<'_> {
        CopyObjectFluentBuilder::new(self)
    }

    pub fn delete_objects(&self) -> DeleteObjectsFluentBuilder<'_> {
        DeleteObjectsFluentBuilder::new(self)
    }

    pub fn head_bucket(&self) -> HeadBucketFluentBuilder<'_> {
        HeadBucketFluentBuilder::new(self)
    }

    pub fn create_bucket(&self) -> CreateBucketFluentBuilder<'_> {
        CreateBucketFluentBuilder::new(self)
    }

    pub fn delete_bucket(&self) -> DeleteBucketFluentBuilder<'_> {
        DeleteBucketFluentBuilder::new(self)
    }

    pub fn list_buckets(&self) -> ListBucketsFluentBuilder<'_> {
        ListBucketsFluentBuilder::new(self)
    }

    pub fn list_parts(&self) -> ListPartsFluentBuilder<'_> {
        ListPartsFluentBuilder::new(self)
    }

    pub fn list_multipart_uploads(&self) -> ListMultipartUploadsFluentBuilder<'_> {
        ListMultipartUploadsFluentBuilder::new(self)
    }

    pub fn get_bucket_versioning(&self) -> GetBucketVersioningFluentBuilder<'_> {
        GetBucketVersioningFluentBuilder::new(self)
    }

    pub fn put_bucket_versioning(&self) -> PutBucketVersioningFluentBuilder<'_> {
        PutBucketVersioningFluentBuilder::new(self)
    }

    pub fn get_bucket_tagging(&self) -> GetBucketTaggingFluentBuilder<'_> {
        GetBucketTaggingFluentBuilder::new(self)
    }

    pub fn put_bucket_tagging(&self) -> PutBucketTaggingFluentBuilder<'_> {
        PutBucketTaggingFluentBuilder::new(self)
    }

    pub fn delete_bucket_tagging(&self) -> DeleteBucketTaggingFluentBuilder<'_> {
        DeleteBucketTaggingFluentBuilder::new(self)
    }

    pub fn get_object_tagging(&self) -> GetObjectTaggingFluentBuilder<'_> {
        GetObjectTaggingFluentBuilder::new(self)
    }

    pub fn put_object_tagging(&self) -> PutObjectTaggingFluentBuilder<'_> {
        PutObjectTaggingFluentBuilder::new(self)
    }

    pub fn delete_object_tagging(&self) -> DeleteObjectTaggingFluentBuilder<'_> {
        DeleteObjectTaggingFluentBuilder::new(self)
    }

    pub fn get_public_access_block(&self) -> GetPublicAccessBlockFluentBuilder<'_> {
        GetPublicAccessBlockFluentBuilder::new(self)
    }

    pub fn put_public_access_block(&self) -> PutPublicAccessBlockFluentBuilder<'_> {
        PutPublicAccessBlockFluentBuilder::new(self)
    }

    pub fn delete_public_access_block(&self) -> DeletePublicAccessBlockFluentBuilder<'_> {
        DeletePublicAccessBlockFluentBuilder::new(self)
    }

    pub fn get_bucket_policy(&self) -> GetBucketPolicyFluentBuilder<'_> {
        GetBucketPolicyFluentBuilder::new(self)
    }

    pub fn put_bucket_policy(&self) -> PutBucketPolicyFluentBuilder<'_> {
        PutBucketPolicyFluentBuilder::new(self)
    }

    pub fn delete_bucket_policy(&self) -> DeleteBucketPolicyFluentBuilder<'_> {
        DeleteBucketPolicyFluentBuilder::new(self)
    }

    pub fn get_bucket_cors(&self) -> GetBucketCorsFluentBuilder<'_> {
        GetBucketCorsFluentBuilder::new(self)
    }

    pub fn put_bucket_cors(&self) -> PutBucketCorsFluentBuilder<'_> {
        PutBucketCorsFluentBuilder::new(self)
    }

    pub fn delete_bucket_cors(&self) -> DeleteBucketCorsFluentBuilder<'_> {
        DeleteBucketCorsFluentBuilder::new(self)
    }

    pub fn get_bucket_encryption(&self) -> GetBucketEncryptionFluentBuilder<'_> {
        GetBucketEncryptionFluentBuilder::new(self)
    }

    pub fn put_bucket_encryption(&self) -> PutBucketEncryptionFluentBuilder<'_> {
        PutBucketEncryptionFluentBuilder::new(self)
    }

    pub fn delete_bucket_encryption(&self) -> DeleteBucketEncryptionFluentBuilder<'_> {
        DeleteBucketEncryptionFluentBuilder::new(self)
    }

    pub fn get_bucket_lifecycle_configuration(
        &self,
    ) -> GetBucketLifecycleConfigurationFluentBuilder<'_> {
        GetBucketLifecycleConfigurationFluentBuilder::new(self)
    }

    pub fn put_bucket_lifecycle_configuration(
        &self,
    ) -> PutBucketLifecycleConfigurationFluentBuilder<'_> {
        PutBucketLifecycleConfigurationFluentBuilder::new(self)
    }

    pub fn delete_bucket_lifecycle(&self) -> DeleteBucketLifecycleFluentBuilder<'_> {
        DeleteBucketLifecycleFluentBuilder::new(self)
    }

    pub fn get_bucket_notification_configuration(
        &self,
    ) -> GetBucketNotificationConfigurationFluentBuilder<'_> {
        GetBucketNotificationConfigurationFluentBuilder::new(self)
    }

    pub fn put_bucket_notification_configuration(
        &self,
    ) -> PutBucketNotificationConfigurationFluentBuilder<'_> {
        PutBucketNotificationConfigurationFluentBuilder::new(self)
    }

    pub fn get_bucket_website(&self) -> GetBucketWebsiteFluentBuilder<'_> {
        GetBucketWebsiteFluentBuilder::new(self)
    }

    pub fn put_bucket_website(&self) -> PutBucketWebsiteFluentBuilder<'_> {
        PutBucketWebsiteFluentBuilder::new(self)
    }

    pub fn delete_bucket_website(&self) -> DeleteBucketWebsiteFluentBuilder<'_> {
        DeleteBucketWebsiteFluentBuilder::new(self)
    }

    pub fn get_bucket_ownership_controls(&self) -> GetBucketOwnershipControlsFluentBuilder<'_> {
        GetBucketOwnershipControlsFluentBuilder::new(self)
    }

    pub fn put_bucket_ownership_controls(&self) -> PutBucketOwnershipControlsFluentBuilder<'_> {
        PutBucketOwnershipControlsFluentBuilder::new(self)
    }

    pub fn delete_bucket_ownership_controls(
        &self,
    ) -> DeleteBucketOwnershipControlsFluentBuilder<'_> {
        DeleteBucketOwnershipControlsFluentBuilder::new(self)
    }

    pub fn get_object_legal_hold(&self) -> GetObjectLegalHoldFluentBuilder<'_> {
        GetObjectLegalHoldFluentBuilder::new(self)
    }

    pub fn put_object_legal_hold(&self) -> PutObjectLegalHoldFluentBuilder<'_> {
        PutObjectLegalHoldFluentBuilder::new(self)
    }

    pub fn get_object_retention(&self) -> GetObjectRetentionFluentBuilder<'_> {
        GetObjectRetentionFluentBuilder::new(self)
    }

    pub fn put_object_retention(&self) -> PutObjectRetentionFluentBuilder<'_> {
        PutObjectRetentionFluentBuilder::new(self)
    }

    pub fn get_object_lock_configuration(&self) -> GetObjectLockConfigurationFluentBuilder<'_> {
        GetObjectLockConfigurationFluentBuilder::new(self)
    }

    pub fn put_object_lock_configuration(&self) -> PutObjectLockConfigurationFluentBuilder<'_> {
        PutObjectLockConfigurationFluentBuilder::new(self)
    }
}
