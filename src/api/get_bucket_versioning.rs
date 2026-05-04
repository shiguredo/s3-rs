//! GetBucketVersioning API
//!
//! バケットのバージョニング設定を取得する。
//!
//! <https://docs.aws.amazon.com/AmazonS3/latest/API/API_GetBucketVersioning.html>

use crate::client::Client;
use crate::error::Error;
use crate::types::GetBucketVersioningOutput;

use super::{S3Request, build_signed_request, parse_error_response, required};

pub struct GetBucketVersioningFluentBuilder<'a> {
    client: &'a Client,
    bucket: Option<String>,
}

impl<'a> GetBucketVersioningFluentBuilder<'a> {
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
            "GET",
            bucket,
            "",
            &[],
            b"",
            Some(&[("versioning", "")]),
            now,
        )
    }

    pub fn parse_response(
        response: &super::S3Response,
    ) -> Result<GetBucketVersioningOutput, Error> {
        if !response.is_success() {
            return Err(parse_error_response(response));
        }

        let body_text = super::xml_body_text(&response.body)?;

        Ok(GetBucketVersioningOutput {
            status: crate::xml::extract_element(body_text, "Status"),
            mfa_delete: crate::xml::extract_element(body_text, "MfaDelete"),
        })
    }
}
