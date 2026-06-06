//! PutBucketCors API
//!
//! CORS ルールを設定する。既存の CORS 設定は全て上書きされる。
//!
//! <https://docs.aws.amazon.com/AmazonS3/latest/API/API_PutBucketCors.html>

use crate::client::Client;
use crate::error::Error;
use crate::types::{ChecksumAlgorithm, CorsConfiguration, CorsRule, PutBucketCorsOutput};

use super::{S3Request, base64_md5, build_signed_request, parse_error_response, required};

pub struct PutBucketCorsFluentBuilder<'a> {
    client: &'a Client,
    bucket: Option<String>,
    cors_rules: Vec<CorsRule>,
    checksum_algorithm: Option<ChecksumAlgorithm>,
}

impl<'a> PutBucketCorsFluentBuilder<'a> {
    pub(crate) fn new(client: &'a Client) -> Self {
        Self {
            client,
            bucket: None,
            cors_rules: Vec::new(),
            checksum_algorithm: None,
        }
    }

    pub fn bucket(mut self, bucket: impl Into<String>) -> Self {
        self.bucket = Some(bucket.into());
        self
    }

    /// CORS ルールを追加する
    pub fn cors_rule(mut self, rule: CorsRule) -> Self {
        self.cors_rules.push(rule);
        self
    }

    /// CORS 設定を一括指定する
    pub fn cors_configuration(mut self, config: CorsConfiguration) -> Self {
        self.cors_rules = config.cors_rules;
        self
    }

    /// チェックサムアルゴリズムを指定する (CRC32, CRC32C, SHA1, SHA256, CRC64NVME)
    pub fn checksum_algorithm(mut self, input: ChecksumAlgorithm) -> Self {
        self.checksum_algorithm = Some(input);
        self
    }

    /// チェックサムアルゴリズムを Option で設定する (`set_*` バリアント)
    pub fn set_checksum_algorithm(mut self, input: Option<ChecksumAlgorithm>) -> Self {
        self.checksum_algorithm = input;
        self
    }

    pub fn build_request(&self, now: std::time::SystemTime) -> Result<S3Request, Error> {
        let bucket = required(self.bucket.as_deref(), "bucket")?;

        if self.cors_rules.is_empty() {
            return Err(Error::InvalidInput("cors_rules is required".to_string()));
        }

        let xml_body = build_cors_xml(&self.cors_rules);
        let content_md5 = base64_md5(xml_body.as_bytes());
        let mut extra_headers: Vec<(&str, &str)> = vec![
            ("content-type", "application/xml"),
            ("content-md5", content_md5.as_str()),
        ];

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
            Some(&[("cors", "")]),
            now,
        )
    }

    pub fn parse_response(response: &super::S3Response) -> Result<PutBucketCorsOutput, Error> {
        if !response.is_success() {
            return Err(parse_error_response(response));
        }
        Ok(PutBucketCorsOutput {})
    }
}

fn build_cors_xml(rules: &[CorsRule]) -> String {
    let mut w = crate::xml::XmlWriter::new();
    w.start_ns("CORSConfiguration", crate::xml::S3_NS);
    for rule in rules {
        w.start("CORSRule");
        if let Some(ref id) = rule.id {
            w.element("ID", id);
        }
        for origin in &rule.allowed_origins {
            w.element("AllowedOrigin", origin);
        }
        for method in &rule.allowed_methods {
            w.element("AllowedMethod", method);
        }
        if let Some(ref headers) = rule.allowed_headers {
            for header in headers {
                w.element("AllowedHeader", header);
            }
        }
        if let Some(max_age) = rule.max_age_seconds {
            w.element("MaxAgeSeconds", &max_age.to_string());
        }
        if let Some(ref headers) = rule.expose_headers {
            for header in headers {
                w.element("ExposeHeader", header);
            }
        }
        w.end();
    }
    w.end();
    w.finish()
}
