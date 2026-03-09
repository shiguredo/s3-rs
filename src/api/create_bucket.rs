//! CreateBucket API
//!
//! 新しいバケットを作成する。
//! us-east-1 以外のリージョンでは LocationConstraint が自動的に設定される。
//!
//! <https://docs.aws.amazon.com/AmazonS3/latest/API/API_CreateBucket.html>

use crate::client::S3Client;
use crate::error::Error;
use crate::types::CreateBucketOutput;

use super::{S3Request, build_signed_request, parse_error_response, required};

pub struct CreateBucketFluentBuilder<'a> {
    client: &'a S3Client,
    bucket: Option<String>,
}

impl<'a> CreateBucketFluentBuilder<'a> {
    pub(crate) fn new(client: &'a S3Client) -> Self {
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
        let config = self.client.config_ref();

        // us-east-1 以外のリージョンでは LocationConstraint が必要
        let body = if config.region == "us-east-1" {
            Vec::new()
        } else {
            let mut w = crate::xml::XmlWriter::new();
            w.start_ns("CreateBucketConfiguration", crate::xml::S3_NS);
            w.element("LocationConstraint", config.region);
            w.end();
            w.finish().into_bytes()
        };

        Ok(build_signed_request(
            &config,
            "PUT",
            bucket,
            "",
            &[],
            &body,
            None,
        ))
    }

    pub fn parse_response(response: &super::S3Response) -> Result<CreateBucketOutput, Error> {
        if !response.is_success() {
            return Err(parse_error_response(response));
        }

        Ok(CreateBucketOutput {
            location: response.get_header("location").map(String::from),
        })
    }
}
