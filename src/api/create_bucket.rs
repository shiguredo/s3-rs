//! CreateBucket API
//!
//! 新しいバケットを作成する。
//!
//! <https://docs.aws.amazon.com/AmazonS3/latest/API/API_CreateBucket.html>

use crate::client::S3Client;
use crate::error::Error;
use crate::types::{CreateBucketConfiguration, CreateBucketOutput};

use super::{S3Request, build_signed_request, parse_error_response, required};

pub struct CreateBucketFluentBuilder<'a> {
    client: &'a S3Client,
    bucket: Option<String>,
    create_bucket_configuration: Option<CreateBucketConfiguration>,
    acl: Option<String>,
}

impl<'a> CreateBucketFluentBuilder<'a> {
    pub(crate) fn new(client: &'a S3Client) -> Self {
        Self {
            client,
            bucket: None,
            create_bucket_configuration: None,
            acl: None,
        }
    }

    pub fn bucket(mut self, bucket: impl Into<String>) -> Self {
        self.bucket = Some(bucket.into());
        self
    }

    pub fn create_bucket_configuration(mut self, config: CreateBucketConfiguration) -> Self {
        self.create_bucket_configuration = Some(config);
        self
    }

    /// ACL を指定する (private, public-read, public-read-write 等)
    pub fn acl(mut self, acl: impl Into<String>) -> Self {
        self.acl = Some(acl.into());
        self
    }

    pub fn build_request(&self) -> Result<S3Request, Error> {
        let bucket = required(self.bucket.as_deref(), "bucket")?;
        let config = self.client.config_ref();

        let body = match &self.create_bucket_configuration {
            Some(cbc) => match &cbc.location_constraint {
                Some(constraint) => {
                    let mut w = crate::xml::XmlWriter::new();
                    w.start_ns("CreateBucketConfiguration", crate::xml::S3_NS);
                    w.element("LocationConstraint", constraint);
                    w.end();
                    w.finish().into_bytes()
                }
                None => Vec::new(),
            },
            None => Vec::new(),
        };

        let mut extra_headers = Vec::new();
        if let Some(ref v) = self.acl {
            extra_headers.push(("x-amz-acl", v.as_str()));
        }

        Ok(build_signed_request(
            &config,
            "PUT",
            bucket,
            "",
            &extra_headers,
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
