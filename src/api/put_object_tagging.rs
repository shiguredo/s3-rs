//! PutObjectTagging API
//!
//! オブジェクトにタグを設定する。既存のタグは全て上書きされる。
//!
//! <https://docs.aws.amazon.com/AmazonS3/latest/API/API_PutObjectTagging.html>

use crate::client::Client;
use crate::error::Error;
use crate::types::{ChecksumAlgorithm, PutObjectTaggingOutput, Tag, Tagging};

use super::{S3Request, base64_md5, build_signed_request, parse_error_response, required};

pub struct PutObjectTaggingFluentBuilder<'a> {
    client: &'a Client,
    bucket: Option<String>,
    key: Option<String>,
    version_id: Option<String>,
    tagging: Option<Tagging>,
    checksum_algorithm: Option<ChecksumAlgorithm>,
    expected_bucket_owner: Option<String>,
}

impl<'a> PutObjectTaggingFluentBuilder<'a> {
    pub(crate) fn new(client: &'a Client) -> Self {
        Self {
            client,
            bucket: None,
            key: None,
            version_id: None,
            tagging: None,
            checksum_algorithm: None,
            expected_bucket_owner: None,
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

    /// オブジェクトのバージョン ID を指定する
    pub fn version_id(mut self, version_id: impl Into<String>) -> Self {
        self.version_id = Some(version_id.into());
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

    pub fn build_request(&self) -> Result<S3Request, Error> {
        let bucket = required(self.bucket.as_deref(), "bucket")?;
        let key = required(self.key.as_deref(), "key")?;
        let tagging = self
            .tagging
            .as_ref()
            .ok_or_else(|| Error::InvalidInput("tagging is required".to_string()))?;

        let xml_body = build_tagging_xml(&tagging.tag_set);
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

        let mut query_params: Vec<(&str, &str)> = vec![("tagging", "")];
        if let Some(ref vid) = self.version_id {
            query_params.push(("versionId", vid.as_str()));
        }

        Ok(build_signed_request(
            &self.client.config_ref(),
            "PUT",
            bucket,
            key,
            &extra_headers,
            xml_body.as_bytes(),
            Some(&query_params),
        ))
    }

    pub fn parse_response(response: &super::S3Response) -> Result<PutObjectTaggingOutput, Error> {
        if !response.is_success() {
            return Err(parse_error_response(response));
        }
        Ok(PutObjectTaggingOutput {
            version_id: response.get_header("x-amz-version-id").map(String::from),
        })
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
