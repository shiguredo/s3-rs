//! PutBucketVersioning API
//!
//! バケットのバージョニング設定を変更する。
//! status には "Enabled" または "Suspended" を指定する。
//!
//! <https://docs.aws.amazon.com/AmazonS3/latest/API/API_PutBucketVersioning.html>

use crate::client::Client;
use crate::error::Error;
use crate::types::{ChecksumAlgorithm, PutBucketVersioningOutput};

use super::{S3Request, base64_md5, build_signed_request, parse_error_response, required};

pub struct PutBucketVersioningFluentBuilder<'a> {
    client: &'a Client,
    bucket: Option<String>,
    status: Option<String>,
    checksum_algorithm: Option<ChecksumAlgorithm>,
}

impl<'a> PutBucketVersioningFluentBuilder<'a> {
    pub(crate) fn new(client: &'a Client) -> Self {
        Self {
            client,
            bucket: None,
            status: None,
            checksum_algorithm: None,
        }
    }

    pub fn bucket(mut self, bucket: impl Into<String>) -> Self {
        self.bucket = Some(bucket.into());
        self
    }

    /// バージョニング状態を設定する ("Enabled" または "Suspended")
    pub fn status(mut self, status: impl Into<String>) -> Self {
        self.status = Some(status.into());
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
        let status = required(self.status.as_deref(), "status")?;

        if status != "Enabled" && status != "Suspended" {
            return Err(Error::InvalidInput(
                "status must be \"Enabled\" or \"Suspended\"".to_string(),
            ));
        }

        let mut w = crate::xml::XmlWriter::new();
        w.start_ns("VersioningConfiguration", crate::xml::S3_NS);
        w.element("Status", status);
        w.end();
        let xml_body = w.finish();
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
            Some(&[("versioning", "")]),
            now,
        )
    }

    pub fn parse_response(
        response: &super::S3Response,
    ) -> Result<PutBucketVersioningOutput, Error> {
        if !response.is_success() {
            return Err(parse_error_response(response));
        }
        Ok(PutBucketVersioningOutput {})
    }
}
