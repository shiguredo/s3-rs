//! GetBucketOwnershipControls API
//!
//! バケットの Object Ownership 設定を取得する。
//!
//! <https://docs.aws.amazon.com/AmazonS3/latest/API/API_GetBucketOwnershipControls.html>

use crate::client::Client;
use crate::error::Error;
use crate::types::{GetBucketOwnershipControlsOutput, OwnershipControlsRule};

use super::{S3Request, build_signed_request, parse_error_response, required};

pub struct GetBucketOwnershipControlsFluentBuilder<'a> {
    client: &'a Client,
    bucket: Option<String>,
}

impl<'a> GetBucketOwnershipControlsFluentBuilder<'a> {
    pub(crate) fn new(client: &'a Client) -> Self {
        Self {
            client,
            bucket: None,
        }
    }

    pub fn bucket(mut self, bucket: impl Into<String>) -> Self {
        self.bucket = Some(bucket.into());
        self
    }

    pub fn build_request(&self) -> Result<S3Request, Error> {
        let bucket = required(self.bucket.as_deref(), "bucket")?;
        Ok(build_signed_request(
            &self.client.config_ref(),
            "GET",
            bucket,
            "",
            &[],
            b"",
            Some(&[("ownershipControls", "")]),
        ))
    }

    pub fn parse_response(
        response: &super::S3Response,
    ) -> Result<GetBucketOwnershipControlsOutput, Error> {
        if !response.is_success() {
            return Err(parse_error_response(response));
        }

        let body_text = super::xml_body_text(&response.body)?;
        let mut rules = Vec::new();
        crate::xml::for_each_element(body_text, "Rule", |children| {
            if let Some(ownership) = children.get("ObjectOwnership") {
                rules.push(OwnershipControlsRule {
                    object_ownership: ownership.to_string(),
                });
            }
        });

        Ok(GetBucketOwnershipControlsOutput { rules })
    }
}
