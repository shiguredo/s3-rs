// rust-crypto と aws_lc_rs の両方が無効の場合はコンパイルエラー
#[cfg(not(any(feature = "rust-crypto", feature = "aws_lc_rs")))]
compile_error!("feature \"rust-crypto\" または \"aws_lc_rs\" のどちらかを有効にしてください。");

pub mod api;
mod checksum;
mod client;
mod credential;
mod datetime;
mod error;
mod signing;
pub mod types;
mod xml;

pub use api::{PresignedRequest, S3Request, S3Response};
pub use client::{Client, Config, ConfigBuilder};
pub use credential::Credentials;
pub use error::{Error, s3_error_code};
pub use types::{
    ChecksumAlgorithm, ChecksumMode, CreateBucketConfiguration, EncodingType, MetadataDirective,
    ObjectCannedAcl, ServerSideEncryption, ServerSideEncryptionByDefault, ServerSideEncryptionRule,
    StorageClass, TaggingDirective, validate_imf_fixdate,
};
