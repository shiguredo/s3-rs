//! DeleteObject API
//!
//! バケットからオブジェクトを削除する。
//!
//! <https://docs.aws.amazon.com/AmazonS3/latest/API/API_DeleteObject.html>

use crate::client::S3Client;
use crate::error::Error;
use crate::types::DeleteObjectOutput;

use super::{
    S3Request, build_presigned_url, build_signed_request, parse_error_response, required,
    validate_presign_expires,
};

pub struct DeleteObjectFluentBuilder<'a> {
    client: &'a S3Client,
    bucket: Option<String>,
    key: Option<String>,
    version_id: Option<String>,
}

impl<'a> DeleteObjectFluentBuilder<'a> {
    pub(crate) fn new(client: &'a S3Client) -> Self {
        Self {
            client,
            bucket: None,
            key: None,
            version_id: None,
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

    /// バージョン ID を指定する (バージョニング有効時)
    pub fn version_id(mut self, version_id: impl Into<String>) -> Self {
        self.version_id = Some(version_id.into());
        self
    }

    pub fn build_request(&self) -> Result<S3Request, Error> {
        let bucket = required(self.bucket.as_deref(), "bucket")?;
        let key = required(self.key.as_deref(), "key")?;

        let mut query_params = Vec::new();
        if let Some(ref v) = self.version_id {
            query_params.push(("versionId", v.as_str()));
        }
        let query = if query_params.is_empty() {
            None
        } else {
            Some(query_params.as_slice())
        };

        Ok(build_signed_request(
            &self.client.config_ref(),
            "DELETE",
            bucket,
            key,
            &[],
            b"",
            query,
        ))
    }

    pub fn parse_response(response: &super::S3Response) -> Result<DeleteObjectOutput, Error> {
        if !response.is_success() {
            return Err(parse_error_response(response));
        }

        Ok(DeleteObjectOutput {
            delete_marker: response
                .get_header("x-amz-delete-marker")
                .and_then(|v| v.parse().ok()),
            version_id: response.get_header("x-amz-version-id").map(String::from),
        })
    }

    /// Presigned リクエストを生成する (Sans I/O)
    pub fn presigned(self, expires_in_secs: u64) -> Result<super::PresignedRequest, Error> {
        validate_presign_expires(expires_in_secs)?;
        let bucket = required(self.bucket.as_deref(), "bucket")?;
        let key = required(self.key.as_deref(), "key")?;
        let mut extra_query_params = Vec::new();
        if let Some(ref v) = self.version_id {
            extra_query_params.push(("versionId", v.as_str()));
        }
        let url = build_presigned_url(
            &self.client.config_ref(),
            "DELETE",
            bucket,
            key,
            expires_in_secs,
            &extra_query_params,
            &[],
        );
        Ok(super::PresignedRequest {
            url,
            method: "DELETE".to_string(),
            headers: Vec::new(),
            body: Vec::new(),
        })
    }
}
