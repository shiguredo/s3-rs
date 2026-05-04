//! PutBucketPolicy API
//!
//! バケットポリシーを JSON 文字列として設定する。
//!
//! <https://docs.aws.amazon.com/AmazonS3/latest/API/API_PutBucketPolicy.html>

use crate::client::Client;
use crate::error::Error;
use crate::types::{ChecksumAlgorithm, PutBucketPolicyOutput};

use super::{S3Request, base64_md5, build_signed_request, parse_error_response, required};

pub struct PutBucketPolicyFluentBuilder<'a> {
    client: &'a Client,
    bucket: Option<String>,
    policy: Option<String>,
    checksum_algorithm: Option<ChecksumAlgorithm>,
}

impl<'a> PutBucketPolicyFluentBuilder<'a> {
    pub(crate) fn new(client: &'a Client) -> Self {
        Self {
            client,
            bucket: None,
            policy: None,
            checksum_algorithm: None,
        }
    }

    pub fn bucket(mut self, bucket: impl Into<String>) -> Self {
        self.bucket = Some(bucket.into());
        self
    }

    /// バケットポリシーを JSON 文字列で設定する
    pub fn policy(mut self, policy: impl Into<String>) -> Self {
        self.policy = Some(policy.into());
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

    pub fn build_request(&self) -> Result<S3Request, Error> {
        let bucket = required(self.bucket.as_deref(), "bucket")?;
        let policy = required(self.policy.as_deref(), "policy")?;

        let content_md5 = base64_md5(policy.as_bytes());
        let mut extra_headers: Vec<(&str, &str)> = vec![
            ("content-type", "application/json"),
            ("content-md5", content_md5.as_str()),
        ];

        let computed_checksum;
        if let Some(ref algorithm) = self.checksum_algorithm {
            extra_headers.push(("x-amz-checksum-algorithm", algorithm.as_str()));
            let header_name = crate::checksum::header_name(algorithm)?;
            computed_checksum = crate::checksum::compute_checksum(algorithm, policy.as_bytes())?;
            extra_headers.push((header_name, &computed_checksum));
        }

        Ok(build_signed_request(
            &self.client.config_ref(),
            "PUT",
            bucket,
            "",
            &extra_headers,
            policy.as_bytes(),
            Some(&[("policy", "")]),
        ))
    }

    pub fn parse_response(response: &super::S3Response) -> Result<PutBucketPolicyOutput, Error> {
        if !response.is_success() {
            return Err(parse_error_response(response));
        }
        Ok(PutBucketPolicyOutput {})
    }
}
