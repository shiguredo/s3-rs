//! PutBucketTagging API
//!
//! バケットにタグを設定する。既存のタグは全て上書きされる。
//!
//! <https://docs.aws.amazon.com/AmazonS3/latest/API/API_PutBucketTagging.html>

use crate::client::S3Client;
use crate::error::Error;
use crate::types::{PutBucketTaggingOutput, Tag};

use super::{S3Request, base64_md5, build_signed_request, parse_error_response, required};

pub struct PutBucketTaggingFluentBuilder<'a> {
    client: &'a S3Client,
    bucket: Option<String>,
    tags: Vec<Tag>,
    checksum_algorithm: Option<String>,
}

impl<'a> PutBucketTaggingFluentBuilder<'a> {
    pub(crate) fn new(client: &'a S3Client) -> Self {
        Self {
            client,
            bucket: None,
            tags: Vec::new(),
            checksum_algorithm: None,
        }
    }

    pub fn bucket(mut self, bucket: impl Into<String>) -> Self {
        self.bucket = Some(bucket.into());
        self
    }

    /// タグを追加する
    pub fn tag(mut self, tag: Tag) -> Self {
        self.tags.push(tag);
        self
    }

    /// チェックサムアルゴリズムを指定する (CRC32, CRC32C, SHA1, SHA256, CRC64NVME)
    pub fn checksum_algorithm(mut self, algorithm: impl Into<String>) -> Self {
        self.checksum_algorithm = Some(algorithm.into());
        self
    }

    pub fn build_request(&self) -> Result<S3Request, Error> {
        let bucket = required(self.bucket.as_deref(), "bucket")?;

        let xml_body = build_tagging_xml(&self.tags);
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
            Some(&[("tagging", "")]),
        ))
    }

    pub fn parse_response(response: &super::S3Response) -> Result<PutBucketTaggingOutput, Error> {
        if !response.is_success() {
            return Err(parse_error_response(response));
        }
        Ok(PutBucketTaggingOutput {})
    }
}

fn build_tagging_xml(tags: &[Tag]) -> String {
    let mut w = crate::xml::XmlWriter::new();
    w.start_ns("Tagging", crate::xml::S3_NS);
    w.start("TagSet");
    for tag in tags {
        w.start("Tag");
        w.element("Key", &tag.key);
        w.element("Value", &tag.value);
        w.end();
    }
    w.end();
    w.end();
    w.finish()
}
