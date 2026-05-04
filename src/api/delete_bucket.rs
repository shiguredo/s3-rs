//! DeleteBucket API
//!
//! 空のバケットを削除する。バケットが空でない場合はエラーになる。
//!
//! <https://docs.aws.amazon.com/AmazonS3/latest/API/API_DeleteBucket.html>

use crate::client::Client;
use crate::error::Error;
use crate::types::DeleteBucketOutput;

use super::{S3Request, build_signed_request, parse_error_response, required};

pub struct DeleteBucketFluentBuilder<'a> {
    client: &'a Client,
    bucket: Option<String>,
}

impl<'a> DeleteBucketFluentBuilder<'a> {
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

    pub fn build_request(&self, now: std::time::SystemTime) -> Result<S3Request, Error> {
        let bucket = required(self.bucket.as_deref(), "bucket")?;

        build_signed_request(
            &self.client.config_ref(),
            "DELETE",
            bucket,
            "",
            &[],
            b"",
            None,
            now,
        )
    }

    pub fn parse_response(response: &super::S3Response) -> Result<DeleteBucketOutput, Error> {
        if !response.is_success() {
            return Err(parse_error_response(response));
        }
        Ok(DeleteBucketOutput {})
    }
}
