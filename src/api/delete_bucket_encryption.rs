//! DeleteBucketEncryption API
//!
//! バケットのデフォルト暗号化設定を削除する。
//!
//! <https://docs.aws.amazon.com/AmazonS3/latest/API/API_DeleteBucketEncryption.html>

use crate::client::Client;
use crate::error::Error;
use crate::types::DeleteBucketEncryptionOutput;

use super::{S3Request, build_signed_request, parse_error_response, required};

pub struct DeleteBucketEncryptionFluentBuilder<'a> {
    client: &'a Client,
    bucket: Option<String>,
}

impl<'a> DeleteBucketEncryptionFluentBuilder<'a> {
    pub(crate) fn new(client: &'a Client) -> Self {
        Self {
            client,
            bucket: None,
        }
    }

    pub fn bucket(mut self, bucket: impl Into<String>) -> Self {
        self.bucket = Some(bucket.into());
        self
    }

    pub fn build_request(&self) -> Result<S3Request, Error> {
        let bucket = required(self.bucket.as_deref(), "bucket")?;
        Ok(build_signed_request(
            &self.client.config_ref(),
            "DELETE",
            bucket,
            "",
            &[],
            b"",
            Some(&[("encryption", "")]),
        ))
    }

    pub fn parse_response(
        response: &super::S3Response,
    ) -> Result<DeleteBucketEncryptionOutput, Error> {
        if !response.is_success() {
            return Err(parse_error_response(response));
        }
        Ok(DeleteBucketEncryptionOutput {})
    }
}
