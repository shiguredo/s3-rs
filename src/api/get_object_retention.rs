//! GetObjectRetention API
//!
//! オブジェクトのリテンション（保持期限）を取得する。
//!
//! <https://docs.aws.amazon.com/AmazonS3/latest/API/API_GetObjectRetention.html>

use crate::client::Client;
use crate::error::Error;
use crate::types::{GetObjectRetentionOutput, ObjectLockRetention};

use super::{S3Request, build_signed_request, parse_error_response, required};

pub struct GetObjectRetentionFluentBuilder<'a> {
    client: &'a Client,
    bucket: Option<String>,
    key: Option<String>,
    version_id: Option<String>,
}

impl<'a> GetObjectRetentionFluentBuilder<'a> {
    pub(crate) fn new(client: &'a Client) -> Self {
        Self {
            client,
            bucket: None,
            key: None,
            version_id: None,
        }
    }

    pub fn bucket(mut self, bucket: impl Into<String>) -> Self {
        self.bucket = Some(bucket.into());
        self
    }

    pub fn key(mut self, key: impl Into<String>) -> Self {
        self.key = Some(key.into());
        self
    }

    pub fn version_id(mut self, version_id: impl Into<String>) -> Self {
        self.version_id = Some(version_id.into());
        self
    }

    pub fn build_request(&self) -> Result<S3Request, Error> {
        let bucket = required(self.bucket.as_deref(), "bucket")?;
        let key = required(self.key.as_deref(), "key")?;

        let mut query_params: Vec<(&str, &str)> = vec![("retention", "")];
        if let Some(ref vid) = self.version_id {
            query_params.push(("versionId", vid.as_str()));
        }

        Ok(build_signed_request(
            &self.client.config_ref(),
            "GET",
            bucket,
            key,
            &[],
            b"",
            Some(&query_params),
        ))
    }

    pub fn parse_response(response: &super::S3Response) -> Result<GetObjectRetentionOutput, Error> {
        if !response.is_success() {
            return Err(parse_error_response(response));
        }

        let body_text = super::xml_body_text(&response.body)?;
        let mode = crate::xml::extract_element(body_text, "Mode");
        let retain_until_date = crate::xml::extract_element(body_text, "RetainUntilDate");

        let retention = match (mode, retain_until_date) {
            (Some(m), Some(d)) => Some(ObjectLockRetention {
                mode: m,
                retain_until_date: d,
            }),
            _ => None,
        };

        Ok(GetObjectRetentionOutput { retention })
    }
}
