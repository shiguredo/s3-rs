//! GetObjectLegalHold API
//!
//! オブジェクトのリーガルホールド状態を取得する。
//!
//! <https://docs.aws.amazon.com/AmazonS3/latest/API/API_GetObjectLegalHold.html>

use crate::client::S3Client;
use crate::error::Error;
use crate::types::{GetObjectLegalHoldOutput, ObjectLockLegalHold};

use super::{S3Request, build_signed_request, parse_error_response, required};

pub struct GetObjectLegalHoldFluentBuilder<'a> {
    client: &'a S3Client,
    bucket: Option<String>,
    key: Option<String>,
    version_id: Option<String>,
}

impl<'a> GetObjectLegalHoldFluentBuilder<'a> {
    pub(crate) fn new(client: &'a S3Client) -> Self {
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

        let mut query_params: Vec<(&str, &str)> = vec![("legal-hold", "")];
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

    pub fn parse_response(response: &super::S3Response) -> Result<GetObjectLegalHoldOutput, Error> {
        if !response.is_success() {
            return Err(parse_error_response(response));
        }

        let body_text = super::xml_body_text(&response.body)?;
        let status = crate::xml::extract_element(body_text, "Status");
        let legal_hold = status.map(|s| ObjectLockLegalHold { status: s });

        Ok(GetObjectLegalHoldOutput { legal_hold })
    }
}
