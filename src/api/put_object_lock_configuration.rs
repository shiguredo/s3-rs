//! PutObjectLockConfiguration API
//!
//! バケットの Object Lock 既定設定を行う。
//!
//! <https://docs.aws.amazon.com/AmazonS3/latest/API/API_PutObjectLockConfiguration.html>

use crate::client::Client;
use crate::error::Error;
use crate::types::{ChecksumAlgorithm, ObjectLockConfiguration, PutObjectLockConfigurationOutput};

use super::{S3Request, base64_md5, build_signed_request, parse_error_response, required};

pub struct PutObjectLockConfigurationFluentBuilder<'a> {
    client: &'a Client,
    bucket: Option<String>,
    object_lock_configuration: Option<ObjectLockConfiguration>,
    /// Object Lock 有効化トークン (既存バケットへの Object Lock 有効化時に必要)
    token: Option<String>,
    checksum_algorithm: Option<ChecksumAlgorithm>,
}

impl<'a> PutObjectLockConfigurationFluentBuilder<'a> {
    pub(crate) fn new(client: &'a Client) -> Self {
        Self {
            client,
            bucket: None,
            object_lock_configuration: None,
            token: None,
            checksum_algorithm: None,
        }
    }

    pub fn bucket(mut self, bucket: impl Into<String>) -> Self {
        self.bucket = Some(bucket.into());
        self
    }

    /// Object Lock 設定を指定する
    pub fn object_lock_configuration(mut self, config: ObjectLockConfiguration) -> Self {
        self.object_lock_configuration = Some(config);
        self
    }

    /// Object Lock 有効化トークンを指定する
    ///
    /// 既存バケットに対して Object Lock を有効化する際に必要。
    /// HTTP ヘッダー `x-amz-bucket-object-lock-token` に対応する。
    pub fn token(mut self, token: impl Into<String>) -> Self {
        self.token = Some(token.into());
        self
    }

    /// チェックサムアルゴリズムを指定する
    pub fn checksum_algorithm(mut self, input: ChecksumAlgorithm) -> Self {
        self.checksum_algorithm = Some(input);
        self
    }

    pub fn set_checksum_algorithm(mut self, input: Option<ChecksumAlgorithm>) -> Self {
        self.checksum_algorithm = input;
        self
    }

    pub fn build_request(&self, now: std::time::SystemTime) -> Result<S3Request, Error> {
        let bucket = required(self.bucket.as_deref(), "bucket")?;

        if self.object_lock_configuration.is_none() {
            return Err(Error::InvalidInput(
                "object_lock_configuration is required".to_string(),
            ));
        }

        let xml_body = build_object_lock_configuration_xml(&self.object_lock_configuration);
        let content_md5 = base64_md5(xml_body.as_bytes());
        let mut extra_headers: Vec<(&str, &str)> = vec![
            ("content-type", "application/xml"),
            ("content-md5", content_md5.as_str()),
        ];

        if let Some(ref token) = self.token {
            extra_headers.push(("x-amz-bucket-object-lock-token", token.as_str()));
        }

        let computed_checksum;
        if let Some(ref algorithm) = self.checksum_algorithm {
            extra_headers.push(("x-amz-checksum-algorithm", algorithm.as_str()));
            let header_name = crate::checksum::header_name(algorithm)?;
            computed_checksum = crate::checksum::compute_checksum(algorithm, xml_body.as_bytes())?;
            extra_headers.push((header_name, &computed_checksum));
        }

        build_signed_request(
            &self.client.config_ref(),
            "PUT",
            bucket,
            "",
            &extra_headers,
            xml_body.as_bytes(),
            Some(&[("object-lock", "")]),
            now,
        )
    }

    pub fn parse_response(
        response: &super::S3Response,
    ) -> Result<PutObjectLockConfigurationOutput, Error> {
        if !response.is_success() {
            return Err(parse_error_response(response));
        }
        Ok(PutObjectLockConfigurationOutput {})
    }
}

fn build_object_lock_configuration_xml(config: &Option<ObjectLockConfiguration>) -> String {
    let mut w = crate::xml::XmlWriter::new();
    w.start_ns("ObjectLockConfiguration", crate::xml::S3_NS);

    if let Some(cfg) = config {
        if let Some(ref enabled) = cfg.object_lock_enabled {
            w.element("ObjectLockEnabled", enabled);
        }
        if let Some(ref rule) = cfg.rule {
            w.start("Rule");
            if let Some(ref retention) = rule.default_retention {
                w.start("DefaultRetention");
                if let Some(ref mode) = retention.mode {
                    w.element("Mode", mode);
                }
                if let Some(days) = retention.days {
                    w.element("Days", &days.to_string());
                }
                if let Some(years) = retention.years {
                    w.element("Years", &years.to_string());
                }
                w.end();
            }
            w.end();
        }
    }

    w.end();
    w.finish()
}
