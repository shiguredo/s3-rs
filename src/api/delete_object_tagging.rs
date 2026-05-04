//! DeleteObjectTagging API
//!
//! オブジェクトのタグを全て削除する。
//!
//! <https://docs.aws.amazon.com/AmazonS3/latest/API/API_DeleteObjectTagging.html>

use crate::client::Client;
use crate::error::Error;
use crate::types::DeleteObjectTaggingOutput;

use super::{S3Request, build_signed_request, parse_error_response, required};

pub struct DeleteObjectTaggingFluentBuilder<'a> {
    client: &'a Client,
    bucket: Option<String>,
    key: Option<String>,
    version_id: Option<String>,
    expected_bucket_owner: Option<String>,
}

impl<'a> DeleteObjectTaggingFluentBuilder<'a> {
    pub(crate) fn new(client: &'a Client) -> Self {
        Self {
            client,
            bucket: None,
            key: None,
            version_id: None,
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

    /// 期待されるバケット所有者のアカウント ID を指定する
    pub fn expected_bucket_owner(mut self, expected_bucket_owner: impl Into<String>) -> Self {
        self.expected_bucket_owner = Some(expected_bucket_owner.into());
        self
    }

    pub fn build_request(&self, now: std::time::SystemTime) -> Result<S3Request, Error> {
        let bucket = required(self.bucket.as_deref(), "bucket")?;
        let key = required(self.key.as_deref(), "key")?;

        let mut extra_headers: Vec<(&str, &str)> = Vec::new();
        if let Some(ref owner) = self.expected_bucket_owner {
            extra_headers.push(("x-amz-expected-bucket-owner", owner.as_str()));
        }

        let mut query_params: Vec<(&str, &str)> = vec![("tagging", "")];
        if let Some(ref vid) = self.version_id {
            query_params.push(("versionId", vid.as_str()));
        }

        build_signed_request(
            &self.client.config_ref(),
            "DELETE",
            bucket,
            key,
            &extra_headers,
            b"",
            Some(&query_params),
            now,
        )
    }

    pub fn parse_response(
        response: &super::S3Response,
    ) -> Result<DeleteObjectTaggingOutput, Error> {
        if !response.is_success() {
            return Err(parse_error_response(response));
        }
        Ok(DeleteObjectTaggingOutput {
            version_id: response.get_header("x-amz-version-id").map(String::from),
        })
    }
}
