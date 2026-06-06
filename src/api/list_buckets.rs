//! ListBuckets API
//!
//! 所有する全バケットの一覧を取得する。
//!
//! <https://docs.aws.amazon.com/AmazonS3/latest/API/API_ListBuckets.html>

use crate::client::Client;
use crate::error::Error;
use crate::types::{Bucket, ListBucketsOutput};

use super::{S3Request, build_signed_service_request, parse_error_response};

pub struct ListBucketsFluentBuilder<'a> {
    client: &'a Client,
    max_buckets: Option<u32>,
    continuation_token: Option<String>,
    prefix: Option<String>,
    bucket_region: Option<String>,
}

impl<'a> ListBucketsFluentBuilder<'a> {
    pub(crate) fn new(client: &'a Client) -> Self {
        Self {
            client,
            max_buckets: None,
            continuation_token: None,
            prefix: None,
            bucket_region: None,
        }
    }

    /// 1 ページあたりの最大バケット数を指定する
    pub fn max_buckets(mut self, max_buckets: u32) -> Self {
        self.max_buckets = Some(max_buckets);
        self
    }

    /// ページングの継続トークンを指定する
    pub fn continuation_token(mut self, token: impl Into<String>) -> Self {
        self.continuation_token = Some(token.into());
        self
    }

    /// バケット名のプレフィックスフィルタを指定する
    pub fn prefix(mut self, prefix: impl Into<String>) -> Self {
        self.prefix = Some(prefix.into());
        self
    }

    /// 指定リージョンのバケットのみに絞り込む
    pub fn bucket_region(mut self, region: impl Into<String>) -> Self {
        self.bucket_region = Some(region.into());
        self
    }

    pub fn build_request(&self, now: std::time::SystemTime) -> Result<S3Request, Error> {
        let max_buckets_str = self.max_buckets.map(|v| v.to_string());

        let mut params: Vec<(&str, &str)> = Vec::new();
        if let Some(ref v) = max_buckets_str {
            params.push(("max-buckets", v.as_str()));
        }
        if let Some(ref v) = self.continuation_token {
            params.push(("continuation-token", v.as_str()));
        }
        if let Some(ref v) = self.prefix {
            params.push(("prefix", v.as_str()));
        }
        if let Some(ref v) = self.bucket_region {
            params.push(("bucket-region", v.as_str()));
        }

        build_signed_service_request(
            &self.client.config_ref(),
            "GET",
            "/",
            &[],
            b"",
            Some(params.as_slice()),
            now,
        )
    }

    pub fn parse_response(response: &super::S3Response) -> Result<ListBucketsOutput, Error> {
        if !response.is_success() {
            return Err(parse_error_response(response));
        }

        let body_text = super::xml_body_text(&response.body)?;

        let buckets = extract_xml_buckets(body_text)?;
        let continuation_token = crate::xml::extract_element(body_text, "ContinuationToken")?;
        let prefix = crate::xml::extract_element(body_text, "Prefix")?;
        // <Owner> はトップレベル <ListAllMyBucketsResult> 配下に出現する
        let display_name = crate::xml::extract_element(body_text, "DisplayName")?;
        let id = crate::xml::extract_element(body_text, "ID")?;
        let owner = if display_name.is_some() || id.is_some() {
            Some(crate::types::Owner { display_name, id })
        } else {
            None
        };

        Ok(ListBucketsOutput {
            buckets,
            continuation_token,
            prefix,
            owner,
        })
    }
}

fn extract_xml_buckets(text: &str) -> Result<Vec<Bucket>, Error> {
    let mut buckets = Vec::new();
    crate::xml::for_each_element(text, "Bucket", |elem| {
        buckets.push(Bucket {
            name: elem.get("Name").map(String::from),
            creation_date: elem
                .get("CreationDate")
                .and_then(|s| crate::datetime::parse_iso8601(s).ok()),
            bucket_region: elem.get("BucketRegion").map(String::from),
            bucket_arn: elem.get("BucketArn").map(String::from),
        });
    })?;
    Ok(buckets)
}
