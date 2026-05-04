//! HeadBucket API
//!
//! バケットの存在確認とアクセス権限の検証を行う。
//!
//! <https://docs.aws.amazon.com/AmazonS3/latest/API/API_HeadBucket.html>

use crate::client::Client;
use crate::error::Error;
use crate::types::HeadBucketOutput;

use super::{S3Request, build_signed_request, head_error_from_status, required};

pub struct HeadBucketFluentBuilder<'a> {
    client: &'a Client,
    bucket: Option<String>,
}

impl<'a> HeadBucketFluentBuilder<'a> {
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
            "HEAD",
            bucket,
            "",
            &[],
            b"",
            None,
        ))
    }

    pub fn parse_response(response: &super::S3Response) -> Result<HeadBucketOutput, Error> {
        if !response.is_success() {
            return Err(head_error_from_status(response.status_code));
        }

        Ok(HeadBucketOutput {
            bucket_region: response.get_header("x-amz-bucket-region").map(String::from),
        })
    }
}
