//! PutObjectLegalHold API
//!
//! オブジェクトのリーガルホールドを設定する。
//!
//! <https://docs.aws.amazon.com/AmazonS3/latest/API/API_PutObjectLegalHold.html>

use crate::client::Client;
use crate::error::Error;
use crate::types::PutObjectLegalHoldOutput;

use super::{S3Request, base64_md5, build_signed_request, parse_error_response, required};

pub struct PutObjectLegalHoldFluentBuilder<'a> {
    client: &'a Client,
    bucket: Option<String>,
    key: Option<String>,
    version_id: Option<String>,
    /// "ON" または "OFF"
    legal_hold_status: Option<String>,
}

impl<'a> PutObjectLegalHoldFluentBuilder<'a> {
    pub(crate) fn new(client: &'a Client) -> Self {
        Self {
            client,
            bucket: None,
            key: None,
            version_id: None,
            legal_hold_status: None,
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

    /// リーガルホールドの状態を設定する ("ON" / "OFF")
    pub fn legal_hold_status(mut self, status: impl Into<String>) -> Self {
        self.legal_hold_status = Some(status.into());
        self
    }

    pub fn build_request(&self, now: std::time::SystemTime) -> Result<S3Request, Error> {
        let bucket = required(self.bucket.as_deref(), "bucket")?;
        let key = required(self.key.as_deref(), "key")?;

        let mut query_params: Vec<(&str, &str)> = vec![("legal-hold", "")];
        if let Some(ref vid) = self.version_id {
            query_params.push(("versionId", vid.as_str()));
        }

        let status = required(
            self.legal_hold_status.as_deref(),
            "legal_hold_status is required",
        )?;

        let xml_body = build_legal_hold_xml(status);
        let content_md5 = base64_md5(xml_body.as_bytes());
        let extra_headers: Vec<(&str, &str)> = vec![
            ("content-type", "application/xml"),
            ("content-md5", content_md5.as_str()),
        ];

        build_signed_request(
            &self.client.config_ref(),
            "PUT",
            bucket,
            key,
            &extra_headers,
            xml_body.as_bytes(),
            Some(&query_params),
            now,
        )
    }

    pub fn parse_response(response: &super::S3Response) -> Result<PutObjectLegalHoldOutput, Error> {
        if !response.is_success() {
            return Err(parse_error_response(response));
        }
        Ok(PutObjectLegalHoldOutput {})
    }
}

fn build_legal_hold_xml(status: &str) -> String {
    let mut w = crate::xml::XmlWriter::new();
    w.start_ns("LegalHold", crate::xml::S3_NS);
    w.element("Status", status);
    w.end();
    w.finish()
}
