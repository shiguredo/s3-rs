//! DeleteBucketTagging API
//!
//! バケットのタグを全て削除する。
//!
//! <https://docs.aws.amazon.com/AmazonS3/latest/API/API_DeleteBucketTagging.html>

use crate::client::Client;
use crate::error::Error;
use crate::types::DeleteBucketTaggingOutput;

use super::{S3Request, build_signed_request, parse_error_response, required};

pub struct DeleteBucketTaggingFluentBuilder<'a> {
    client: &'a Client,
    bucket: Option<String>,
    expected_bucket_owner: Option<String>,
}

impl<'a> DeleteBucketTaggingFluentBuilder<'a> {
    pub(crate) fn new(client: &'a Client) -> Self {
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

    /// 期待されるバケット所有者のアカウント ID を指定する
    pub fn expected_bucket_owner(mut self, expected_bucket_owner: impl Into<String>) -> Self {
        self.expected_bucket_owner = Some(expected_bucket_owner.into());
        self
    }

    pub fn build_request(&self) -> Result<S3Request, Error> {
        let bucket = required(self.bucket.as_deref(), "bucket")?;

        let mut extra_headers: Vec<(&str, &str)> = Vec::new();
        if let Some(ref owner) = self.expected_bucket_owner {
            extra_headers.push(("x-amz-expected-bucket-owner", owner.as_str()));
        }

        Ok(build_signed_request(
            &self.client.config_ref(),
            "DELETE",
            bucket,
            "",
            &extra_headers,
            b"",
            Some(&[("tagging", "")]),
        ))
    }

    pub fn parse_response(
        response: &super::S3Response,
    ) -> Result<DeleteBucketTaggingOutput, Error> {
        if !response.is_success() {
            return Err(parse_error_response(response));
        }
        Ok(DeleteBucketTaggingOutput {})
    }
}
