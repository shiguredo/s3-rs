//! types モジュールのルートファイル
//!
//! 責務別に分割したサブモジュール (output / model / enums) の型を
//! `crate::types::Xxx` パスで参照できるように再エクスポートする。
//! サブモジュールは private のため、外部からは `shiguredo_s3::types::Xxx` のみが見える。

use crate::error::Error;

mod enums;
mod model;
mod output;

pub use enums::{
    ChecksumAlgorithm, ChecksumMode, EncodingType, ExpirationStatus, MetadataDirective,
    ObjectCannedAcl, ServerSideEncryption, StorageClass, TaggingDirective,
};

pub use model::{
    AbortIncompleteMultipartUpload, CompletedMultipartUpload, CompletedPart, CorsConfiguration,
    CorsConfigurationBuilder, CorsRule, CorsRuleBuilder, CreateBucketConfiguration,
    DefaultRetention, Delete, DeleteBuilder, ErrorDocument, EventBridgeConfiguration, FilterRule,
    IndexDocument, LambdaFunctionConfiguration, LifecycleExpiration, LifecycleRule,
    LifecycleRuleAndOperator, LifecycleRuleFilter, NoncurrentVersionExpiration,
    NoncurrentVersionTransition, NotificationConfigurationFilter, ObjectIdentifier,
    ObjectLockConfiguration, ObjectLockLegalHold, ObjectLockRetention, ObjectLockRule,
    OwnershipControlsRule, QueueConfiguration, RedirectAllRequestsTo, RoutingRule,
    RoutingRuleCondition, RoutingRuleRedirect, S3KeyFilter, ServerSideEncryptionByDefault,
    ServerSideEncryptionByDefaultBuilder, ServerSideEncryptionConfiguration,
    ServerSideEncryptionConfigurationBuilder, ServerSideEncryptionRule,
    ServerSideEncryptionRuleBuilder, Tag, Tagging, TaggingBuilder, TopicConfiguration, Transition,
};

pub use output::{
    AbortMultipartUploadOutput, Bucket, CommonPrefix, CompleteMultipartUploadOutput,
    CopyObjectOutput, CopyObjectResult, CreateBucketOutput, CreateMultipartUploadOutput,
    DeleteBucketCorsOutput, DeleteBucketEncryptionOutput, DeleteBucketLifecycleOutput,
    DeleteBucketOutput, DeleteBucketOwnershipControlsOutput, DeleteBucketPolicyOutput,
    DeleteBucketTaggingOutput, DeleteBucketWebsiteOutput, DeleteError, DeleteMarkerEntry,
    DeleteObjectOutput, DeleteObjectTaggingOutput, DeleteObjectsOutput,
    DeletePublicAccessBlockOutput, DeletedObject, GetBucketCorsOutput, GetBucketEncryptionOutput,
    GetBucketLifecycleConfigurationOutput, GetBucketNotificationConfigurationOutput,
    GetBucketOwnershipControlsOutput, GetBucketPolicyOutput, GetBucketTaggingOutput,
    GetBucketVersioningOutput, GetBucketWebsiteOutput, GetObjectLegalHoldOutput,
    GetObjectLockConfigurationOutput, GetObjectOutput, GetObjectRetentionOutput,
    GetObjectTaggingOutput, GetPublicAccessBlockOutput, HeadBucketOutput, HeadObjectOutput,
    ListBucketsOutput, ListMultipartUploadsOutput, ListObjectVersionsOutput, ListObjectsOutput,
    ListObjectsV2Output, ListPartsOutput, MultipartUpload, Object, ObjectVersion, Owner, Part,
    PutBucketCorsOutput, PutBucketEncryptionOutput, PutBucketLifecycleConfigurationOutput,
    PutBucketNotificationConfigurationOutput, PutBucketOwnershipControlsOutput,
    PutBucketPolicyOutput, PutBucketTaggingOutput, PutBucketVersioningOutput,
    PutBucketWebsiteOutput, PutObjectLegalHoldOutput, PutObjectLockConfigurationOutput,
    PutObjectOutput, PutObjectRetentionOutput, PutObjectTaggingOutput, PutPublicAccessBlockOutput,
    RestoreStatus, UploadPartCopyOutput, UploadPartOutput,
};

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

    // 日 (s[5..7]) が ASCII 数字
    if !s.as_bytes()[5..7].iter().all(|b| b.is_ascii_digit()) {
        return Err(Error::InvalidInput(
            "expected digits for day in IMF-fixdate".to_string(),
        ));
    }
    // 日と月の間のスペース
    if s.as_bytes()[7] != b' ' {
        return Err(Error::InvalidInput(
            "expected space after day in IMF-fixdate".to_string(),
        ));
    }

    // 月チェック (8..11)
    let month = &s[8..11];
    if !crate::datetime::MONTH_NAMES.contains(&month) {
        return Err(Error::InvalidInput(format!("invalid month: {month}")));
    }

    // 月と年の間のスペース
    if s.as_bytes()[11] != b' ' {
        return Err(Error::InvalidInput(
            "expected space after month in IMF-fixdate".to_string(),
        ));
    }
    // 年 (s[12..16]) が ASCII 数字
    if !s.as_bytes()[12..16].iter().all(|b| b.is_ascii_digit()) {
        return Err(Error::InvalidInput(
            "expected digits for year in IMF-fixdate".to_string(),
        ));
    }
    // 年と時刻の間のスペース
    if s.as_bytes()[16] != b' ' {
        return Err(Error::InvalidInput(
            "expected space after year in IMF-fixdate".to_string(),
        ));
    }
    // 時 (s[17..19]) が ASCII 数字
    if !s.as_bytes()[17..19].iter().all(|b| b.is_ascii_digit()) {
        return Err(Error::InvalidInput(
            "expected digits for hour in IMF-fixdate".to_string(),
        ));
    }
    // : 区切り
    if s.as_bytes()[19] != b':' {
        return Err(Error::InvalidInput(
            "expected ':' after hour in IMF-fixdate".to_string(),
        ));
    }
    // 分 (s[20..22]) が ASCII 数字
    if !s.as_bytes()[20..22].iter().all(|b| b.is_ascii_digit()) {
        return Err(Error::InvalidInput(
            "expected digits for minute in IMF-fixdate".to_string(),
        ));
    }
    // : 区切り
    if s.as_bytes()[22] != b':' {
        return Err(Error::InvalidInput(
            "expected ':' after minute in IMF-fixdate".to_string(),
        ));
    }
    // 秒 (s[23..25]) が ASCII 数字
    if !s.as_bytes()[23..25].iter().all(|b| b.is_ascii_digit()) {
        return Err(Error::InvalidInput(
            "expected digits for second in IMF-fixdate".to_string(),
        ));
    }

    // GMT チェック (末尾)
    if !s.ends_with(" GMT") {
        return Err(Error::InvalidInput(
            "IMF-fixdate must end with ' GMT'".to_string(),
        ));
    }

    Ok(())
}
