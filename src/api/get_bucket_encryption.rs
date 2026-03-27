//! GetBucketEncryption API
//!
//! バケットのデフォルト暗号化設定を取得する。
//!
//! <https://docs.aws.amazon.com/AmazonS3/latest/API/API_GetBucketEncryption.html>

use crate::client::S3Client;
use crate::error::Error;
use crate::types::{
    GetBucketEncryptionOutput, ServerSideEncryptionByDefault, ServerSideEncryptionConfiguration,
    ServerSideEncryptionRule,
};

use super::{S3Request, build_signed_request, parse_error_response, required};

pub struct GetBucketEncryptionFluentBuilder<'a> {
    client: &'a S3Client,
    bucket: Option<String>,
    expected_bucket_owner: Option<String>,
}

impl<'a> GetBucketEncryptionFluentBuilder<'a> {
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

    /// バケット所有者のアカウント ID を指定する (検証用)
    pub fn expected_bucket_owner(mut self, expected_bucket_owner: impl Into<String>) -> Self {
        self.expected_bucket_owner = Some(expected_bucket_owner.into());
        self
    }

    pub fn build_request(&self) -> Result<S3Request, Error> {
        let bucket = required(self.bucket.as_deref(), "bucket")?;

        let mut extra_headers: Vec<(&str, &str)> = Vec::new();
        if let Some(ref v) = self.expected_bucket_owner {
            extra_headers.push(("x-amz-expected-bucket-owner", v.as_str()));
        }

        Ok(build_signed_request(
            &self.client.config_ref(),
            "GET",
            bucket,
            "",
            &extra_headers,
            b"",
            Some(&[("encryption", "")]),
        ))
    }

    pub fn parse_response(
        response: &super::S3Response,
    ) -> Result<GetBucketEncryptionOutput, Error> {
        if !response.is_success() {
            return Err(parse_error_response(response));
        }

        let body_text = std::str::from_utf8(&response.body)
            .map_err(|e| Error::InvalidResponse(format!("invalid UTF-8 in response body: {e}")))?;

        // XML 構造:
        // <ServerSideEncryptionConfiguration>
        //   <Rule>
        //     <ApplyServerSideEncryptionByDefault>
        //       <SSEAlgorithm>AES256</SSEAlgorithm>
        //       <KMSMasterKeyID>...</KMSMasterKeyID>
        //     </ApplyServerSideEncryptionByDefault>
        //     <BucketKeyEnabled>true</BucketKeyEnabled>
        //   </Rule>
        // </ServerSideEncryptionConfiguration>
        //
        // for_each_element は直接の子要素 (depth=2) のみ取得するため、
        // ApplyServerSideEncryptionByDefault 内の SSEAlgorithm には直接アクセスできない。
        // Rule 単位でパースし、ネストした要素は extract_element で取得する。
        let mut rules = Vec::new();
        crate::xml::for_each_element(body_text, "Rule", |elem| {
            let bucket_key_enabled: Option<bool> = elem.get_parsed("BucketKeyEnabled");

            // ApplyServerSideEncryptionByDefault のテキストは空文字列だが、
            // 存在の有無で SSEAlgorithm を含むかを判定する。
            // extract_element はドキュメント全体から検索するため、
            // Rule が複数ある場合には不正確になる可能性があるが、
            // S3 の仕様上 Rule は通常 1 つのみ。
            let sse_by_default = if crate::xml::extract_element(body_text, "SSEAlgorithm").is_some()
            {
                Some(ServerSideEncryptionByDefault {
                    sse_algorithm: crate::xml::extract_element(body_text, "SSEAlgorithm")
                        .unwrap_or_default(),
                    kms_master_key_id: crate::xml::extract_element(body_text, "KMSMasterKeyID"),
                })
            } else {
                None
            };

            rules.push(ServerSideEncryptionRule {
                apply_server_side_encryption_by_default: sse_by_default,
                bucket_key_enabled,
            });
        });

        Ok(GetBucketEncryptionOutput {
            server_side_encryption_configuration: if rules.is_empty() {
                None
            } else {
                Some(ServerSideEncryptionConfiguration { rules })
            },
        })
    }
}
