use std::fmt;

use crate::error::Error;

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
//
// 注意: `ExpirationStatus` はこの方針の例外 (Unknown variant を持たず、
// `FromStr` を実装する)。不明な値は `Error::InvalidResponse` を返す。

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
