//! GetBucketCors API
//!
//! バケットの CORS 設定を取得する。
//!
//! <https://docs.aws.amazon.com/AmazonS3/latest/API/API_GetBucketCors.html>

use crate::client::S3Client;
use crate::error::Error;
use crate::types::{CorsRule, GetBucketCorsOutput};

use super::{S3Request, build_signed_request, parse_error_response, required};

pub struct GetBucketCorsFluentBuilder<'a> {
    client: &'a S3Client,
    bucket: Option<String>,
    expected_bucket_owner: Option<String>,
}

impl<'a> GetBucketCorsFluentBuilder<'a> {
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
            Some(&[("cors", "")]),
        ))
    }

    pub fn parse_response(response: &super::S3Response) -> Result<GetBucketCorsOutput, Error> {
        if !response.is_success() {
            return Err(parse_error_response(response));
        }

        let body_text = std::str::from_utf8(&response.body)
            .map_err(|e| Error::InvalidResponse(format!("invalid UTF-8 in response body: {e}")))?;

        let mut rules = Vec::new();
        crate::xml::for_each_element(body_text, "CORSRule", |elem| {
            rules.push(CorsRule {
                id: elem.get("ID").map(String::from),
                allowed_headers: {
                    let v: Vec<String> = elem
                        .get_all("AllowedHeader")
                        .into_iter()
                        .map(String::from)
                        .collect();
                    if v.is_empty() { None } else { Some(v) }
                },
                allowed_methods: elem
                    .get_all("AllowedMethod")
                    .into_iter()
                    .map(String::from)
                    .collect(),
                allowed_origins: elem
                    .get_all("AllowedOrigin")
                    .into_iter()
                    .map(String::from)
                    .collect(),
                expose_headers: {
                    let v: Vec<String> = elem
                        .get_all("ExposeHeader")
                        .into_iter()
                        .map(String::from)
                        .collect();
                    if v.is_empty() { None } else { Some(v) }
                },
                max_age_seconds: elem.get_parsed("MaxAgeSeconds"),
            });
        });

        Ok(GetBucketCorsOutput {
            cors_rules: if rules.is_empty() { None } else { Some(rules) },
        })
    }
}
