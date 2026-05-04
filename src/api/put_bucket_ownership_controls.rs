//! PutBucketOwnershipControls API
//!
//! バケットの Object Ownership ルールを設定する。
//!
//! <https://docs.aws.amazon.com/AmazonS3/latest/API/API_PutBucketOwnershipControls.html>

use crate::client::Client;
use crate::error::Error;
use crate::types::{OwnershipControlsRule, PutBucketOwnershipControlsOutput};

use super::{S3Request, base64_md5, build_signed_request, parse_error_response, required};

pub struct PutBucketOwnershipControlsFluentBuilder<'a> {
    client: &'a Client,
    bucket: Option<String>,
    rules: Vec<OwnershipControlsRule>,
}

impl<'a> PutBucketOwnershipControlsFluentBuilder<'a> {
    pub(crate) fn new(client: &'a Client) -> Self {
        Self {
            client,
            bucket: None,
            rules: Vec::new(),
        }
    }

    pub fn bucket(mut self, bucket: impl Into<String>) -> Self {
        self.bucket = Some(bucket.into());
        self
    }

    /// Object Ownership ルールを追加する
    pub fn rule(mut self, rule: OwnershipControlsRule) -> Self {
        self.rules.push(rule);
        self
    }

    pub fn build_request(&self) -> Result<S3Request, Error> {
        let bucket = required(self.bucket.as_deref(), "bucket")?;

        let xml_body = build_ownership_controls_xml(&self.rules);
        let content_md5 = base64_md5(xml_body.as_bytes());
        let extra_headers: Vec<(&str, &str)> = vec![
            ("content-type", "application/xml"),
            ("content-md5", content_md5.as_str()),
        ];

        Ok(build_signed_request(
            &self.client.config_ref(),
            "PUT",
            bucket,
            "",
            &extra_headers,
            xml_body.as_bytes(),
            Some(&[("ownershipControls", "")]),
        ))
    }

    pub fn parse_response(
        response: &super::S3Response,
    ) -> Result<PutBucketOwnershipControlsOutput, Error> {
        if !response.is_success() {
            return Err(parse_error_response(response));
        }
        Ok(PutBucketOwnershipControlsOutput {})
    }
}

fn build_ownership_controls_xml(rules: &[OwnershipControlsRule]) -> String {
    let mut w = crate::xml::XmlWriter::new();
    w.start_ns("OwnershipControls", crate::xml::S3_NS);
    for rule in rules {
        w.start("Rule");
        w.element("ObjectOwnership", &rule.object_ownership);
        w.end();
    }
    w.end();
    w.finish()
}
