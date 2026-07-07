//! PutBucketTagging API
//!
//! バケットにタグを設定する。既存のタグは全て上書きされる。
//!
//! <https://docs.aws.amazon.com/AmazonS3/latest/API/API_PutBucketTagging.html>

use crate::client::Client;
use crate::error::Error;
use crate::types::{ChecksumAlgorithm, PutBucketTaggingOutput, Tag, Tagging};

use super::{S3Request, base64_md5, build_signed_request, parse_error_response, required};

pub struct PutBucketTaggingFluentBuilder<'a> {
    client: &'a Client,
    bucket: Option<String>,
    tagging: Option<Tagging>,
    checksum_algorithm: Option<ChecksumAlgorithm>,
    expected_bucket_owner: Option<String>,
}

impl<'a> PutBucketTaggingFluentBuilder<'a> {
    pub(crate) fn new(client: &'a Client) -> Self {
        Self {
            client,
            bucket: None,
            tagging: None,
            checksum_algorithm: None,
            expected_bucket_owner: None,
        }
    }

    pub fn bucket(mut self, bucket: impl Into<String>) -> Self {
        self.bucket = Some(bucket.into());
        self
    }

    /// タグセットを設定する
    pub fn tagging(mut self, tagging: Tagging) -> Self {
        self.tagging = Some(tagging);
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

    /// 期待されるバケット所有者のアカウント ID を指定する
    pub fn expected_bucket_owner(mut self, expected_bucket_owner: impl Into<String>) -> Self {
        self.expected_bucket_owner = Some(expected_bucket_owner.into());
        self
    }

    pub fn build_request(&self, now: std::time::SystemTime) -> Result<S3Request, Error> {
        let bucket = required(self.bucket.as_deref(), "bucket")?;
        let tagging = self
            .tagging
            .as_ref()
            .ok_or_else(|| Error::InvalidInput("tagging is required".to_string()))?;

        let xml_body = build_tagging_xml(&tagging.tag_set)?;
        let content_md5 = base64_md5(xml_body.as_bytes());
        let mut extra_headers: Vec<(&str, &str)> = vec![
            ("content-type", "application/xml"),
            ("content-md5", content_md5.as_str()),
        ];

        if let Some(ref owner) = self.expected_bucket_owner {
            extra_headers.push(("x-amz-expected-bucket-owner", owner.as_str()));
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
            Some(&[("tagging", "")]),
            now,
        )
    }

    pub fn parse_response(response: &super::S3Response) -> Result<PutBucketTaggingOutput, Error> {
        if !response.is_success() {
            return Err(parse_error_response(response));
        }
        Ok(PutBucketTaggingOutput {})
    }
}

fn build_tagging_xml(tags: &[Tag]) -> Result<String, Error> {
    let mut w = crate::xml::XmlWriter::new();
    w.start_ns("Tagging", crate::xml::S3_NS);
    w.start("TagSet");
    for tag in tags {
        w.start("Tag");
        w.element("Key", &tag.key)?;
        w.element("Value", &tag.value)?;
        w.end();
    }
    w.end();
    w.end();
    Ok(w.finish())
}
