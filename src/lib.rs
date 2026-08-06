// rust-crypto と aws_lc_rs の両方が無効の場合はコンパイルエラー
#[cfg(not(any(feature = "rust-crypto", feature = "aws_lc_rs")))]
compile_error!("either feature \"rust-crypto\" or \"aws_lc_rs\" must be enabled");

pub mod api;
mod checksum;
mod client;
mod credential;
mod datetime;
mod error;
mod request;
mod signing;
pub mod types;
mod xml;

pub use api::{PresignedRequest, S3Request, S3Response};
pub use client::{Client, Config, ConfigBuilder};
pub use credential::Credentials;
pub use error::{Error, s3_error_code};
pub use types::{
    ChecksumAlgorithm, ChecksumMode, CopyObjectResult, CreateBucketConfiguration, Delete,
    DeleteBuilder, EncodingType, ListObjectsOutput, MetadataDirective, ObjectCannedAcl,
    ObjectIdentifier, Owner, RestoreStatus, ServerSideEncryption, ServerSideEncryptionByDefault,
    ServerSideEncryptionRule, StorageClass, TaggingDirective, validate_imf_fixdate,
};

/// `unix_timestamp_from_civil` → `civil_from_unix_timestamp` のラウンドトリップ (PBT 用)
#[doc(hidden)]
pub fn datetime_round_trip(
    year: i32,
    month: u32,
    day: u32,
    hour: u32,
    minute: u32,
    second: u32,
) -> Result<(i32, u32, u32, u32, u32, u32), Error> {
    let secs = crate::datetime::unix_timestamp_from_civil(year, month, day, hour, minute, second)?;
    let civil = crate::datetime::civil_from_unix_timestamp(secs)?;
    Ok((
        civil.year,
        civil.month,
        civil.day,
        civil.hour,
        civil.minute,
        civil.second,
    ))
}
