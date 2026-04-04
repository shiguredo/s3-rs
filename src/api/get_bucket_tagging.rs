//! GetBucketTagging API
//!
//! バケットに設定されているタグの一覧を取得する。
//!
//! <https://docs.aws.amazon.com/AmazonS3/latest/API/API_GetBucketTagging.html>

use crate::client::S3Client;
use crate::error::Error;
use crate::types::{GetBucketTaggingOutput, Tag};

use super::{S3Request, build_signed_request, parse_error_response, required};

pub struct GetBucketTaggingFluentBuilder<'a> {
    client: &'a S3Client,
    bucket: Option<String>,
    expected_bucket_owner: Option<String>,
}

impl<'a> GetBucketTaggingFluentBuilder<'a> {
    pub(crate) fn new(client: &'a S3Client) -> Self {
        Self {
            client,
            bucket: None,
            expected_bucket_owner: None,
        }
    }

    pub fn bucket(mut self, bucket: impl Into<String>) -> Self {
        self.bucket = Some(bucket.into());
        self
    }

    /// 期待されるバケット所有者のアカウント ID を指定する
    pub fn expected_bucket_owner(mut self, expected_bucket_owner: impl Into<String>) -> Self {
        self.expected_bucket_owner = Some(expected_bucket_owner.into());
        self
    }

    pub fn build_request(&self) -> Result<S3Request, Error> {
        let bucket = required(self.bucket.as_deref(), "bucket")?;

        let mut extra_headers: Vec<(&str, &str)> = Vec::new();
        if let Some(ref owner) = self.expected_bucket_owner {
            extra_headers.push(("x-amz-expected-bucket-owner", owner.as_str()));
        }

        Ok(build_signed_request(
            &self.client.config_ref(),
            "GET",
            bucket,
            "",
            &extra_headers,
            b"",
            Some(&[("tagging", "")]),
        ))
    }

    pub fn parse_response(response: &super::S3Response) -> Result<GetBucketTaggingOutput, Error> {
        if !response.is_success() {
            return Err(parse_error_response(response));
        }

        let body_text = super::xml_body_text(&response.body)?;

        Ok(GetBucketTaggingOutput {
            tag_set: extract_xml_tags(body_text),
        })
    }
}

fn extract_xml_tags(text: &str) -> Vec<Tag> {
    let mut tags = Vec::new();
    crate::xml::for_each_element(text, "Tag", |elem| {
        if let (Some(key), Some(value)) = (elem.get("Key"), elem.get("Value")) {
            tags.push(Tag {
                key: key.to_string(),
                value: value.to_string(),
            });
        }
    });
    tags
}
