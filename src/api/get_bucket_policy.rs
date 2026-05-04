//! GetBucketPolicy API
//!
//! バケットに設定されているポリシーを JSON 文字列として取得する。
//!
//! <https://docs.aws.amazon.com/AmazonS3/latest/API/API_GetBucketPolicy.html>

use crate::client::Client;
use crate::error::Error;
use crate::types::GetBucketPolicyOutput;

use super::{S3Request, build_signed_request, parse_error_response, required};

pub struct GetBucketPolicyFluentBuilder<'a> {
    client: &'a Client,
    bucket: Option<String>,
}

impl<'a> GetBucketPolicyFluentBuilder<'a> {
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
            "GET",
            bucket,
            "",
            &[],
            b"",
            Some(&[("policy", "")]),
        ))
    }

    pub fn parse_response(response: &super::S3Response) -> Result<GetBucketPolicyOutput, Error> {
        if !response.is_success() {
            return Err(parse_error_response(response));
        }

        let policy = if response.body.is_empty() {
            None
        } else {
            std::str::from_utf8(&response.body)
                .map(|s| Some(s.to_string()))
                .map_err(|_| Error::InvalidResponse("non-UTF-8 policy body".to_string()))?
        };

        Ok(GetBucketPolicyOutput { policy })
    }
}
