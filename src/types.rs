use std::fmt;

/// HTTP 日時 (IMF-fixdate 形式)
///
/// RFC 9110 Section 5.6.7 で定義される IMF-fixdate 形式の日時。
/// 条件付きリクエストヘッダー (`If-Modified-Since`, `If-Unmodified-Since`) で使用する。
///
/// # 構築方法
///
/// ```
/// use shiguredo_s3::types::HttpDate;
///
/// // UNIX タイムスタンプから生成する
/// let date = HttpDate::from_unix_timestamp(0);
/// assert_eq!(date.as_str(), "Thu, 01 Jan 1970 00:00:00 GMT");
///
/// // S3 レスポンスの Last-Modified ヘッダー値をそのまま渡す
/// let date = HttpDate::from_imf_fixdate("Mon, 09 Mar 2026 12:00:00 GMT");
/// assert_eq!(date.as_str(), "Mon, 09 Mar 2026 12:00:00 GMT");
/// ```
#[derive(Debug, Clone)]
pub struct HttpDate {
    value: String,
}

const WEEKDAY_NAMES: [&str; 7] = ["Sun", "Mon", "Tue", "Wed", "Thu", "Fri", "Sat"];
const MONTH_NAMES: [&str; 12] = [
    "Jan", "Feb", "Mar", "Apr", "May", "Jun", "Jul", "Aug", "Sep", "Oct", "Nov", "Dec",
];

impl HttpDate {
    /// UNIX タイムスタンプ (秒) から生成する
    pub fn from_unix_timestamp(secs: u64) -> Self {
        let c = crate::datetime::civil_from_unix_timestamp(secs);

        // 曜日: UNIX epoch (1970-01-01) は木曜日 (4)
        let weekday = ((c.days_since_epoch % 7 + 4 + 7) % 7) as usize;

        let value = format!(
            "{}, {:02} {} {:04} {:02}:{:02}:{:02} GMT",
            WEEKDAY_NAMES[weekday],
            c.day,
            MONTH_NAMES[(c.month - 1) as usize],
            c.year,
            c.hour,
            c.minute,
            c.second
        );

        Self { value }
    }

    /// IMF-fixdate 文字列から生成する
    ///
    /// S3 レスポンスの `Last-Modified` ヘッダー値をそのまま渡すことができる。
    /// フォーマットのバリデーションは行わない。
    pub fn from_imf_fixdate(s: impl Into<String>) -> Self {
        Self { value: s.into() }
    }

    /// IMF-fixdate 形式の文字列を返す
    pub fn as_str(&self) -> &str {
        &self.value
    }
}

impl fmt::Display for HttpDate {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.value)
    }
}

/// GetObject の結果
#[derive(Debug)]
pub struct GetObjectOutput {
    pub body: Vec<u8>,
    pub content_type: Option<String>,
    pub content_length: Option<i64>,
    pub e_tag: Option<String>,
    pub last_modified: Option<String>,
    /// オブジェクトのバージョン ID (バージョニング有効時)
    pub version_id: Option<String>,
    /// カスタムメタデータ (x-amz-meta-* ヘッダーから抽出)
    pub metadata: Option<std::collections::HashMap<String, String>>,
}

/// HeadObject の結果
#[derive(Debug)]
pub struct HeadObjectOutput {
    pub content_type: Option<String>,
    pub content_length: Option<i64>,
    pub e_tag: Option<String>,
    pub last_modified: Option<String>,
    /// オブジェクトのバージョン ID (バージョニング有効時)
    pub version_id: Option<String>,
    /// カスタムメタデータ (x-amz-meta-* ヘッダーから抽出)
    pub metadata: Option<std::collections::HashMap<String, String>>,
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
    pub e_tag: Option<String>,
    pub last_modified: Option<String>,
    /// コピー先オブジェクトのバージョン ID (バージョニング有効時)
    pub version_id: Option<String>,
    /// コピー元オブジェクトのバージョン ID
    pub copy_source_version_id: Option<String>,
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

/// S3 オブジェクトのメタデータ
#[derive(Debug, Clone)]
pub struct Object {
    pub key: Option<String>,
    pub last_modified: Option<String>,
    pub e_tag: Option<String>,
    pub size: Option<i64>,
    pub storage_class: Option<String>,
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
    pub creation_date: Option<String>,
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

/// PutBucketTagging の結果
#[derive(Debug)]
pub struct PutBucketTaggingOutput {}

/// DeleteBucketTagging の結果
#[derive(Debug)]
pub struct DeleteBucketTaggingOutput {}

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
    pub storage_class: Option<String>,
}

/// アップロード済みパート
#[derive(Debug, Clone)]
pub struct Part {
    pub part_number: Option<i32>,
    pub last_modified: Option<String>,
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
    pub initiated: Option<String>,
    pub storage_class: Option<String>,
}
