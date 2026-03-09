//! GetPublicAccessBlock API
//!
//! バケットのパブリックアクセスブロック設定を取得する。
//!
//! <https://docs.aws.amazon.com/AmazonS3/latest/API/API_GetPublicAccessBlock.html>

use crate::client::S3Client;
use crate::error::Error;
use crate::types::GetPublicAccessBlockOutput;

use super::{S3Request, build_signed_request, parse_error_response, required};

pub struct GetPublicAccessBlockFluentBuilder<'a> {
    client: &'a S3Client,
    bucket: Option<String>,
}

impl<'a> GetPublicAccessBlockFluentBuilder<'a> {
    pub(crate) fn new(client: &'a S3Client) -> Self {
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
            Some(&[("publicAccessBlock", "")]),
        ))
    }

    pub fn parse_response(
        response: &super::S3Response,
    ) -> Result<GetPublicAccessBlockOutput, Error> {
        if !response.is_success() {
            return Err(parse_error_response(response));
        }

        let body_text = std::str::from_utf8(&response.body)
            .map_err(|_| Error::InvalidResponse("non-UTF-8 response body".to_string()))?;

        Ok(GetPublicAccessBlockOutput {
            block_public_acls: crate::xml::extract_element(body_text, "BlockPublicAcls")
                .and_then(|v| v.parse::<bool>().ok()),
            ignore_public_acls: crate::xml::extract_element(body_text, "IgnorePublicAcls")
                .and_then(|v| v.parse::<bool>().ok()),
            block_public_policy: crate::xml::extract_element(body_text, "BlockPublicPolicy")
                .and_then(|v| v.parse::<bool>().ok()),
            restrict_public_buckets: crate::xml::extract_element(
                body_text,
                "RestrictPublicBuckets",
            )
            .and_then(|v| v.parse::<bool>().ok()),
        })
    }
}
