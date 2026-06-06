//! PutObjectRetention API
//!
//! オブジェクトのリテンション（保持期限）を設定する。
//!
//! <https://docs.aws.amazon.com/AmazonS3/latest/API/API_PutObjectRetention.html>

use crate::client::Client;
use crate::error::Error;
use crate::types::PutObjectRetentionOutput;

use super::{S3Request, base64_md5, build_signed_request, parse_error_response, required};

pub struct PutObjectRetentionFluentBuilder<'a> {
    client: &'a Client,
    bucket: Option<String>,
    key: Option<String>,
    version_id: Option<String>,
    /// "GOVERNANCE" または "COMPLIANCE"
    mode: Option<String>,
    /// 保持期限 (ISO 8601 形式)
    retain_until_date: Option<String>,
    /// GOVERNANCE モードのリテンションを上書きする
    bypass_governance_retention: Option<bool>,
}

impl<'a> PutObjectRetentionFluentBuilder<'a> {
    pub(crate) fn new(client: &'a Client) -> Self {
        Self {
            client,
            bucket: None,
            key: None,
            version_id: None,
            mode: None,
            retain_until_date: None,
            bypass_governance_retention: None,
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

    /// リテンションモードを設定する ("GOVERNANCE" / "COMPLIANCE")
    pub fn mode(mut self, mode: impl Into<String>) -> Self {
        self.mode = Some(mode.into());
        self
    }

    /// 保持期限を設定する (ISO 8601 形式)
    pub fn retain_until_date(mut self, date: impl Into<String>) -> Self {
        self.retain_until_date = Some(date.into());
        self
    }

    /// GOVERNANCE モードのリテンションを上書きする
    pub fn bypass_governance_retention(mut self, bypass: bool) -> Self {
        self.bypass_governance_retention = Some(bypass);
        self
    }

    pub fn build_request(&self, now: std::time::SystemTime) -> Result<S3Request, Error> {
        let bucket = required(self.bucket.as_deref(), "bucket")?;
        let key = required(self.key.as_deref(), "key")?;

        if self.mode.is_none() && self.retain_until_date.is_none() {
            return Err(Error::InvalidInput(
                "mode or retain_until_date is required".to_string(),
            ));
        }

        let mut query_params: Vec<(&str, &str)> = vec![("retention", "")];
        if let Some(ref vid) = self.version_id {
            query_params.push(("versionId", vid.as_str()));
        }

        let xml_body = build_retention_xml(self.mode.as_deref(), self.retain_until_date.as_deref());
        let content_md5 = base64_md5(xml_body.as_bytes());
        let mut extra_headers: Vec<(&str, &str)> = vec![
            ("content-type", "application/xml"),
            ("content-md5", content_md5.as_str()),
        ];

        if self.bypass_governance_retention == Some(true) {
            extra_headers.push(("x-amz-bypass-governance-retention", "true"));
        }

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

    pub fn parse_response(response: &super::S3Response) -> Result<PutObjectRetentionOutput, Error> {
        if !response.is_success() {
            return Err(parse_error_response(response));
        }
        Ok(PutObjectRetentionOutput {})
    }
}

fn build_retention_xml(mode: Option<&str>, retain_until_date: Option<&str>) -> String {
    let mut w = crate::xml::XmlWriter::new();
    w.start_ns("Retention", crate::xml::S3_NS);
    if let Some(m) = mode {
        w.element("Mode", m);
    }
    if let Some(d) = retain_until_date {
        w.element("RetainUntilDate", d);
    }
    w.end();
    w.finish()
}
