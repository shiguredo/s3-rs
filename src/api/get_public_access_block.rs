//! GetPublicAccessBlock API
//!
//! バケットのパブリックアクセスブロック設定を取得する。
//!
//! <https://docs.aws.amazon.com/AmazonS3/latest/API/API_GetPublicAccessBlock.html>

use crate::client::Client;
use crate::error::Error;
use crate::types::GetPublicAccessBlockOutput;

use super::{S3Request, build_signed_request, parse_error_response, required};

pub struct GetPublicAccessBlockFluentBuilder<'a> {
    client: &'a Client,
    bucket: Option<String>,
}

impl<'a> GetPublicAccessBlockFluentBuilder<'a> {
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
            Some(&[("publicAccessBlock", "")]),
            now,
        )
    }

    pub fn parse_response(
        response: &super::S3Response,
    ) -> Result<GetPublicAccessBlockOutput, Error> {
        if !response.is_success() {
            return Err(parse_error_response(response));
        }

        let body_text = super::xml_body_text(&response.body)?;

        Ok(GetPublicAccessBlockOutput {
            block_public_acls: crate::xml::extract_element(body_text, "BlockPublicAcls")?
                .map(|v| crate::xml::parse_xml_bool(&v))
                .transpose()?,
            ignore_public_acls: crate::xml::extract_element(body_text, "IgnorePublicAcls")?
                .map(|v| crate::xml::parse_xml_bool(&v))
                .transpose()?,
            block_public_policy: crate::xml::extract_element(body_text, "BlockPublicPolicy")?
                .map(|v| crate::xml::parse_xml_bool(&v))
                .transpose()?,
            restrict_public_buckets: crate::xml::extract_element(
                body_text,
                "RestrictPublicBuckets",
            )?
            .map(|v| crate::xml::parse_xml_bool(&v))
            .transpose()?,
        })
    }
}
