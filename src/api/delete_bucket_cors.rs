//! DeleteBucketCors API
//!
//! バケットの CORS 設定を削除する。
//!
//! <https://docs.aws.amazon.com/AmazonS3/latest/API/API_DeleteBucketCors.html>

use crate::client::S3Client;
use crate::error::Error;
use crate::types::DeleteBucketCorsOutput;

use super::{S3Request, build_signed_request, parse_error_response, required};

pub struct DeleteBucketCorsFluentBuilder<'a> {
    client: &'a S3Client,
    bucket: Option<String>,
    expected_bucket_owner: Option<String>,
}

impl<'a> DeleteBucketCorsFluentBuilder<'a> {
    pub(crate) fn new(client: &'a S3Client) -> Self {
        Self {
            client,
            bucket: None,
            expected_bucket_owner: None,
        }
    }

    pub fn bucket(mut self, bucket: impl Into<String>) -> Self {
        self.bucket = Some(bucket.into());
        self
    }

    /// バケット所有者のアカウント ID を指定する (検証用)
    pub fn expected_bucket_owner(mut self, expected_bucket_owner: impl Into<String>) -> Self {
        self.expected_bucket_owner = Some(expected_bucket_owner.into());
        self
    }

    pub fn build_request(&self) -> Result<S3Request, Error> {
        let bucket = required(self.bucket.as_deref(), "bucket")?;

        let mut extra_headers: Vec<(&str, &str)> = Vec::new();
        if let Some(ref v) = self.expected_bucket_owner {
            extra_headers.push(("x-amz-expected-bucket-owner", v.as_str()));
        }

        Ok(build_signed_request(
            &self.client.config_ref(),
            "DELETE",
            bucket,
            "",
            &extra_headers,
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
