//! PutBucketEncryption API
//!
//! バケットのデフォルト暗号化設定を作成・更新する。
//!
//! <https://docs.aws.amazon.com/AmazonS3/latest/API/API_PutBucketEncryption.html>

use crate::client::S3Client;
use crate::error::Error;
use crate::types::{PutBucketEncryptionOutput, ServerSideEncryptionConfiguration};

use super::{S3Request, base64_md5, build_signed_request, parse_error_response, required};

pub struct PutBucketEncryptionFluentBuilder<'a> {
    client: &'a S3Client,
    bucket: Option<String>,
    server_side_encryption_configuration: Option<ServerSideEncryptionConfiguration>,
    checksum_algorithm: Option<String>,
    expected_bucket_owner: Option<String>,
}

impl<'a> PutBucketEncryptionFluentBuilder<'a> {
    pub(crate) fn new(client: &'a S3Client) -> Self {
        Self {
            client,
            bucket: None,
            server_side_encryption_configuration: None,
            checksum_algorithm: None,
            expected_bucket_owner: None,
        }
    }

    pub fn bucket(mut self, bucket: impl Into<String>) -> Self {
        self.bucket = Some(bucket.into());
        self
    }

    /// 暗号化設定を指定する
    pub fn server_side_encryption_configuration(
        mut self,
        config: ServerSideEncryptionConfiguration,
    ) -> Self {
        self.server_side_encryption_configuration = Some(config);
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
        let config = self
            .server_side_encryption_configuration
            .as_ref()
            .ok_or_else(|| {
                Error::InvalidInput("server_side_encryption_configuration is required".into())
            })?;

        let xml_body = build_encryption_configuration_xml(config);
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
            Some(&[("encryption", "")]),
        ))
    }

    pub fn parse_response(
        response: &super::S3Response,
    ) -> Result<PutBucketEncryptionOutput, Error> {
        if !response.is_success() {
            return Err(parse_error_response(response));
        }

        Ok(PutBucketEncryptionOutput {})
    }
}

/// ServerSideEncryptionConfiguration を XML に変換する
fn build_encryption_configuration_xml(config: &ServerSideEncryptionConfiguration) -> String {
    let mut w = crate::xml::XmlWriter::new();
    w.start_ns("ServerSideEncryptionConfiguration", crate::xml::S3_NS);
    for rule in &config.rules {
        w.start("Rule");
        if let Some(ref by_default) = rule.apply_server_side_encryption_by_default {
            w.start("ApplyServerSideEncryptionByDefault");
            w.element("SSEAlgorithm", &by_default.sse_algorithm);
            if let Some(ref key_id) = by_default.kms_master_key_id {
                w.element("KMSMasterKeyID", key_id);
            }
            w.end();
        }
        if let Some(enabled) = rule.bucket_key_enabled {
            w.element("BucketKeyEnabled", if enabled { "true" } else { "false" });
        }
        w.end();
    }
    w.end();
    w.finish()
}
