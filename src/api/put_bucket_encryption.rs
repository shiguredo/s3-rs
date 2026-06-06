//! PutBucketEncryption API
//!
//! バケットのデフォルト暗号化設定を設定する。既存の設定は上書きされる。
//!
//! <https://docs.aws.amazon.com/AmazonS3/latest/API/API_PutBucketEncryption.html>

use crate::client::Client;
use crate::error::Error;
use crate::types::{
    ChecksumAlgorithm, PutBucketEncryptionOutput, ServerSideEncryptionConfiguration,
    ServerSideEncryptionRule,
};

use super::{S3Request, base64_md5, build_signed_request, parse_error_response, required};

pub struct PutBucketEncryptionFluentBuilder<'a> {
    client: &'a Client,
    bucket: Option<String>,
    rules: Vec<ServerSideEncryptionRule>,
    checksum_algorithm: Option<ChecksumAlgorithm>,
}

impl<'a> PutBucketEncryptionFluentBuilder<'a> {
    pub(crate) fn new(client: &'a Client) -> Self {
        Self {
            client,
            bucket: None,
            rules: Vec::new(),
            checksum_algorithm: None,
        }
    }

    pub fn bucket(mut self, bucket: impl Into<String>) -> Self {
        self.bucket = Some(bucket.into());
        self
    }

    /// 暗号化ルールを追加する
    pub fn rule(mut self, rule: ServerSideEncryptionRule) -> Self {
        self.rules.push(rule);
        self
    }

    /// 暗号化設定を一括指定する
    pub fn server_side_encryption_configuration(
        mut self,
        config: ServerSideEncryptionConfiguration,
    ) -> Self {
        self.rules = config.rules;
        self
    }

    /// チェックサムアルゴリズムを指定する (CRC32, CRC32C, SHA1, SHA256, CRC64NVME)
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

        if self.rules.is_empty() {
            return Err(Error::InvalidInput(
                "encryption rules is required".to_string(),
            ));
        }
        for rule in &self.rules {
            if let Some(ref default) = rule.apply_server_side_encryption_by_default
                && default.sse_algorithm.is_empty()
            {
                return Err(Error::InvalidInput("sse_algorithm is required".to_string()));
            }
        }

        let xml_body = build_encryption_xml(&self.rules);
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
            Some(&[("encryption", "")]),
            now,
        )
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

fn build_encryption_xml(rules: &[ServerSideEncryptionRule]) -> String {
    let mut w = crate::xml::XmlWriter::new();
    w.start_ns("ServerSideEncryptionConfiguration", crate::xml::S3_NS);
    for rule in rules {
        w.start("Rule");
        if let Some(ref default) = rule.apply_server_side_encryption_by_default {
            w.start("ApplyServerSideEncryptionByDefault");
            w.element("SSEAlgorithm", &default.sse_algorithm);
            if let Some(ref key_id) = default.kms_master_key_id {
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
