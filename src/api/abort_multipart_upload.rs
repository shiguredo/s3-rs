//! AbortMultipartUpload API
//!
//! 進行中のマルチパートアップロードを中止し、アップロード済みパートを破棄する。
//!
//! <https://docs.aws.amazon.com/AmazonS3/latest/API/API_AbortMultipartUpload.html>

use crate::client::Client;
use crate::error::Error;
use crate::types::AbortMultipartUploadOutput;

use super::{
    S3Request, build_presigned_url, build_signed_request, parse_error_response, required,
    validate_presign_expires,
};

pub struct AbortMultipartUploadFluentBuilder<'a> {
    client: &'a Client,
    bucket: Option<String>,
    key: Option<String>,
    upload_id: Option<String>,
}

impl<'a> AbortMultipartUploadFluentBuilder<'a> {
    pub(crate) fn new(client: &'a Client) -> Self {
        Self {
            client,
            bucket: None,
            key: None,
            upload_id: None,
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

    pub fn upload_id(mut self, upload_id: impl Into<String>) -> Self {
        self.upload_id = Some(upload_id.into());
        self
    }

    pub fn build_request(&self, now: std::time::SystemTime) -> Result<S3Request, Error> {
        let bucket = required(self.bucket.as_deref(), "bucket")?;
        let key = required(self.key.as_deref(), "key")?;
        let upload_id = required(self.upload_id.as_deref(), "upload_id")?;

        let query_params = [("uploadId", upload_id)];
        build_signed_request(
            &self.client.config_ref(),
            "DELETE",
            bucket,
            key,
            &[],
            b"",
            Some(&query_params),
            now,
        )
    }

    pub fn parse_response(
        response: &super::S3Response,
    ) -> Result<AbortMultipartUploadOutput, Error> {
        if !response.is_success() {
            return Err(parse_error_response(response));
        }
        Ok(AbortMultipartUploadOutput {})
    }

    /// Presigned リクエストを生成する (Sans I/O)
    pub fn presigned(
        self,
        expires_in_secs: u64,
        now: std::time::SystemTime,
    ) -> Result<super::PresignedRequest, Error> {
        validate_presign_expires(expires_in_secs)?;
        let bucket = required(self.bucket.as_deref(), "bucket")?;
        let key = required(self.key.as_deref(), "key")?;
        let upload_id = required(self.upload_id.as_deref(), "upload_id")?;
        let url = build_presigned_url(
            &self.client.config_ref(),
            "DELETE",
            bucket,
            key,
            expires_in_secs,
            &[("uploadId", upload_id)],
            &[],
            now,
        )?;
        Ok(super::PresignedRequest {
            url,
            method: "DELETE".to_string(),
            headers: Vec::new(),
            body: Vec::new(),
        })
    }
}
