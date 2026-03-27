//! PutBucketCors API
//!
//! バケットの CORS 設定を作成・更新する。
//!
//! <https://docs.aws.amazon.com/AmazonS3/latest/API/API_PutBucketCors.html>

use crate::client::S3Client;
use crate::error::Error;
use crate::types::{CorsConfiguration, PutBucketCorsOutput};

use super::{S3Request, base64_md5, build_signed_request, parse_error_response, required};

pub struct PutBucketCorsFluentBuilder<'a> {
    client: &'a S3Client,
    bucket: Option<String>,
    cors_configuration: Option<CorsConfiguration>,
    checksum_algorithm: Option<String>,
    expected_bucket_owner: Option<String>,
}

impl<'a> PutBucketCorsFluentBuilder<'a> {
    pub(crate) fn new(client: &'a S3Client) -> Self {
        Self {
            client,
            bucket: None,
            cors_configuration: None,
            checksum_algorithm: None,
            expected_bucket_owner: None,
        }
    }

    pub fn bucket(mut self, bucket: impl Into<String>) -> Self {
        self.bucket = Some(bucket.into());
        self
    }

    /// CORS 設定を指定する
    pub fn cors_configuration(mut self, cors_configuration: CorsConfiguration) -> Self {
        self.cors_configuration = Some(cors_configuration);
        self
    }

    /// チェックサムアルゴリズムを指定する (CRC32, CRC32C, SHA1, SHA256, CRC64NVME)
    pub fn checksum_algorithm(mut self, algorithm: impl Into<String>) -> Self {
        self.checksum_algorithm = Some(algorithm.into());
        self
    }

    /// バケット所有者のアカウント ID を指定する (検証用)
    pub fn expected_bucket_owner(mut self, expected_bucket_owner: impl Into<String>) -> Self {
        self.expected_bucket_owner = Some(expected_bucket_owner.into());
        self
    }

    pub fn build_request(&self) -> Result<S3Request, Error> {
        let bucket = required(self.bucket.as_deref(), "bucket")?;
        let cors_configuration = self
            .cors_configuration
            .as_ref()
            .ok_or_else(|| Error::InvalidInput("cors_configuration is required".into()))?;

        let xml_body = build_cors_configuration_xml(cors_configuration);
        let content_md5 = base64_md5(xml_body.as_bytes());

        let mut extra_headers: Vec<(&str, &str)> = vec![
            ("content-type", "application/xml"),
            ("content-md5", content_md5.as_str()),
        ];

        let computed_checksum;
        if let Some(ref algo_str) = self.checksum_algorithm {
            extra_headers.push(("x-amz-checksum-algorithm", algo_str.as_str()));
            let algorithm: crate::checksum::ChecksumAlgorithm = algo_str.parse()?;
            computed_checksum = crate::checksum::compute_checksum(algorithm, xml_body.as_bytes());
            extra_headers.push((algorithm.header_name(), &computed_checksum));
        }

        if let Some(ref v) = self.expected_bucket_owner {
            extra_headers.push(("x-amz-expected-bucket-owner", v.as_str()));
        }

        Ok(build_signed_request(
            &self.client.config_ref(),
            "PUT",
            bucket,
            "",
            &extra_headers,
            xml_body.as_bytes(),
            Some(&[("cors", "")]),
        ))
    }

    pub fn parse_response(response: &super::S3Response) -> Result<PutBucketCorsOutput, Error> {
        if !response.is_success() {
            return Err(parse_error_response(response));
        }

        Ok(PutBucketCorsOutput {})
    }
}

/// CorsConfiguration を XML に変換する
fn build_cors_configuration_xml(config: &CorsConfiguration) -> String {
    let mut w = crate::xml::XmlWriter::new();
    w.start_ns("CORSConfiguration", crate::xml::S3_NS);
    for rule in &config.cors_rules {
        w.start("CORSRule");
        if let Some(ref id) = rule.id {
            w.element("ID", id);
        }
        if let Some(ref headers) = rule.allowed_headers {
            for header in headers {
                w.element("AllowedHeader", header);
            }
        }
        for method in &rule.allowed_methods {
            w.element("AllowedMethod", method);
        }
        for origin in &rule.allowed_origins {
            w.element("AllowedOrigin", origin);
        }
        if let Some(ref headers) = rule.expose_headers {
            for header in headers {
                w.element("ExposeHeader", header);
            }
        }
        if let Some(seconds) = rule.max_age_seconds {
            w.element("MaxAgeSeconds", &seconds.to_string());
        }
        w.end();
    }
    w.end();
    w.finish()
}
