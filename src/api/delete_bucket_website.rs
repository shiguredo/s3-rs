//! DeleteBucketWebsite API
//!
//! バケットのウェブサイト設定を削除する。
//!
//! <https://docs.aws.amazon.com/AmazonS3/latest/API/API_DeleteBucketWebsite.html>

use crate::client::Client;
use crate::error::Error;
use crate::types::DeleteBucketWebsiteOutput;

use super::{S3Request, build_signed_request, parse_error_response, required};

pub struct DeleteBucketWebsiteFluentBuilder<'a> {
    client: &'a Client,
    bucket: Option<String>,
}

impl<'a> DeleteBucketWebsiteFluentBuilder<'a> {
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
            Some(&[("website", "")]),
        ))
    }

    pub fn parse_response(
        response: &super::S3Response,
    ) -> Result<DeleteBucketWebsiteOutput, Error> {
        if !response.is_success() {
            return Err(parse_error_response(response));
        }
        Ok(DeleteBucketWebsiteOutput {})
    }
}
