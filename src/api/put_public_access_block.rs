//! PutPublicAccessBlock API
//!
//! バケットのパブリックアクセスブロック設定を変更する。
//!
//! <https://docs.aws.amazon.com/AmazonS3/latest/API/API_PutPublicAccessBlock.html>

use crate::client::Client;
use crate::error::Error;
use crate::types::PutPublicAccessBlockOutput;

use super::{S3Request, base64_md5, build_signed_request, parse_error_response, required};

pub struct PutPublicAccessBlockFluentBuilder<'a> {
    client: &'a Client,
    bucket: Option<String>,
    block_public_acls: Option<bool>,
    ignore_public_acls: Option<bool>,
    block_public_policy: Option<bool>,
    restrict_public_buckets: Option<bool>,
    checksum_algorithm: Option<String>,
}

impl<'a> PutPublicAccessBlockFluentBuilder<'a> {
    pub(crate) fn new(client: &'a Client) -> Self {
        Self {
            client,
            bucket: None,
            block_public_acls: None,
            ignore_public_acls: None,
            block_public_policy: None,
            restrict_public_buckets: None,
            checksum_algorithm: None,
        }
    }

    pub fn bucket(mut self, bucket: impl Into<String>) -> Self {
        self.bucket = Some(bucket.into());
        self
    }

    pub fn block_public_acls(mut self, v: bool) -> Self {
        self.block_public_acls = Some(v);
        self
    }

    pub fn ignore_public_acls(mut self, v: bool) -> Self {
        self.ignore_public_acls = Some(v);
        self
    }

    pub fn block_public_policy(mut self, v: bool) -> Self {
        self.block_public_policy = Some(v);
        self
    }

    pub fn restrict_public_buckets(mut self, v: bool) -> Self {
        self.restrict_public_buckets = Some(v);
        self
    }

    /// チェックサムアルゴリズムを指定する (CRC32, CRC32C, SHA1, SHA256, CRC64NVME)
    pub fn checksum_algorithm(mut self, algorithm: impl Into<String>) -> Self {
        self.checksum_algorithm = Some(algorithm.into());
        self
    }

    pub fn build_request(&self) -> Result<S3Request, Error> {
        let bucket = required(self.bucket.as_deref(), "bucket")?;

        // AWS SDK 互換: Some の項目だけ XML 要素を出力する
        let mut w = crate::xml::XmlWriter::new();
        w.start_ns("PublicAccessBlockConfiguration", crate::xml::S3_NS);
        if let Some(v) = self.block_public_acls {
            w.element("BlockPublicAcls", &v.to_string());
        }
        if let Some(v) = self.ignore_public_acls {
            w.element("IgnorePublicAcls", &v.to_string());
        }
        if let Some(v) = self.block_public_policy {
            w.element("BlockPublicPolicy", &v.to_string());
        }
        if let Some(v) = self.restrict_public_buckets {
            w.element("RestrictPublicBuckets", &v.to_string());
        }
        w.end();
        let xml_body = w.finish();
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

        Ok(build_signed_request(
            &self.client.config_ref(),
            "PUT",
            bucket,
            "",
            &extra_headers,
            xml_body.as_bytes(),
            Some(&[("publicAccessBlock", "")]),
        ))
    }

    pub fn parse_response(
        response: &super::S3Response,
    ) -> Result<PutPublicAccessBlockOutput, Error> {
        if !response.is_success() {
            return Err(parse_error_response(response));
        }
        Ok(PutPublicAccessBlockOutput {})
    }
}
