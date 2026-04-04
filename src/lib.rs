// rust-crypto と aws-lc-rs の両方が無効の場合はコンパイルエラー
#[cfg(not(any(feature = "rust-crypto", feature = "aws-lc-rs")))]
compile_error!("feature \"rust-crypto\" または \"aws-lc-rs\" のどちらかを有効にしてください。");

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
pub use client::{S3Client, S3Config, S3ConfigBuilder};
pub use credential::Credential;
pub use error::{Error, s3_error_code};
pub use types::{
    CreateBucketConfiguration, HttpDate, ServerSideEncryptionByDefault, ServerSideEncryptionRule,
};
