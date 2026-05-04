//! DeleteBucketCors API
//!
//! CORS 設定を削除する。
//!
//! <https://docs.aws.amazon.com/AmazonS3/latest/API/API_DeleteBucketCors.html>

use crate::client::Client;
use crate::error::Error;
use crate::types::DeleteBucketCorsOutput;

use super::{S3Request, build_signed_request, parse_error_response, required};

pub struct DeleteBucketCorsFluentBuilder<'a> {
    client: &'a Client,
    bucket: Option<String>,
}

impl<'a> DeleteBucketCorsFluentBuilder<'a> {
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
            Some(&[("cors", "")]),
        ))
    }

    pub fn parse_response(response: &super::S3Response) -> Result<DeleteBucketCorsOutput, Error> {
        if !response.is_success() {
            return Err(parse_error_response(response));
        }
        Ok(DeleteBucketCorsOutput {})
    }
}
