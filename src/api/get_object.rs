//! GetObject API
//!
//! バケットからオブジェクトを取得する。
//! Range ヘッダーや PartNumber による部分取得にも対応。
//!
//! <https://docs.aws.amazon.com/AmazonS3/latest/API/API_GetObject.html>

use crate::client::S3Client;
use crate::error::Error;
use crate::types::{GetObjectOutput, HttpDate};

use super::{
    S3Request, build_presigned_url, build_signed_request, parse_error_response, required,
    validate_presign_expires,
};

pub struct GetObjectFluentBuilder<'a> {
    client: &'a S3Client,
    bucket: Option<String>,
    key: Option<String>,
    range: Option<String>,
    part_number: Option<i32>,
    if_match: Option<String>,
    if_none_match: Option<String>,
    if_modified_since: Option<HttpDate>,
    if_unmodified_since: Option<HttpDate>,
    sse_customer_algorithm: Option<String>,
    sse_customer_key: Option<String>,
    version_id: Option<String>,
}

impl<'a> GetObjectFluentBuilder<'a> {
    pub(crate) fn new(client: &'a S3Client) -> Self {
        Self {
            client,
            bucket: None,
            key: None,
            range: None,
            part_number: None,
            if_match: None,
            if_none_match: None,
            if_modified_since: None,
            if_unmodified_since: None,
            sse_customer_algorithm: None,
            sse_customer_key: None,
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

    /// Range ヘッダーを指定する (例: "bytes=0-999")
    pub fn range(mut self, range: impl Into<String>) -> Self {
        self.range = Some(range.into());
        self
    }

    /// マルチパートアップロードされたオブジェクトの特定パートを取得する
    pub fn part_number(mut self, part_number: i32) -> Self {
        self.part_number = Some(part_number);
        self
    }

    /// ETag が一致する場合のみオブジェクトを返す
    ///
    /// 不一致の場合は 412 Precondition Failed が返される。
    pub fn if_match(mut self, e_tag: impl Into<String>) -> Self {
        self.if_match = Some(e_tag.into());
        self
    }

    /// ETag が異なる場合のみオブジェクトを返す
    ///
    /// 一致する場合は 304 Not Modified が返される。
    pub fn if_none_match(mut self, e_tag: impl Into<String>) -> Self {
        self.if_none_match = Some(e_tag.into());
        self
    }

    /// 指定日時以降に変更されている場合のみオブジェクトを返す
    ///
    /// 変更されていない場合は 304 Not Modified が返される。
    pub fn if_modified_since(mut self, date: HttpDate) -> Self {
        self.if_modified_since = Some(date);
        self
    }

    /// 指定日時以降に変更されていない場合のみオブジェクトを返す
    ///
    /// 変更されている場合は 412 Precondition Failed が返される。
    pub fn if_unmodified_since(mut self, date: HttpDate) -> Self {
        self.if_unmodified_since = Some(date);
        self
    }

    /// SSE-C アルゴリズムを指定する (AES256)
    pub fn sse_customer_algorithm(mut self, algorithm: impl Into<String>) -> Self {
        self.sse_customer_algorithm = Some(algorithm.into());
        self
    }

    /// SSE-C キーを指定する (Base64 エンコード)
    ///
    /// MD5 はキーから自動計算される。
    pub fn sse_customer_key(mut self, key: impl Into<String>) -> Self {
        self.sse_customer_key = Some(key.into());
        self
    }

    /// バージョン ID を指定する (バージョニング有効時)
    pub fn version_id(mut self, version_id: impl Into<String>) -> Self {
        self.version_id = Some(version_id.into());
        self
    }

    /// 署名済みリクエストを構築する (Sans I/O)
    pub fn build_request(&self) -> Result<S3Request, Error> {
        let bucket = required(self.bucket.as_deref(), "bucket")?;
        let key = required(self.key.as_deref(), "key")?;

        let mut extra_headers = Vec::new();
        if let Some(ref range) = self.range {
            extra_headers.push(("range", range.as_str()));
        }
        if let Some(ref v) = self.if_match {
            extra_headers.push(("if-match", v.as_str()));
        }
        if let Some(ref v) = self.if_none_match {
            extra_headers.push(("if-none-match", v.as_str()));
        }
        if let Some(ref v) = self.if_modified_since {
            extra_headers.push(("if-modified-since", v.as_str()));
        }
        if let Some(ref v) = self.if_unmodified_since {
            extra_headers.push(("if-unmodified-since", v.as_str()));
        }
        if let Some(ref v) = self.sse_customer_algorithm {
            extra_headers.push((
                "x-amz-server-side-encryption-customer-algorithm",
                v.as_str(),
            ));
        }
        // SSE-C キーが指定されている場合、MD5 を自動計算する
        let computed_key_md5;
        if let Some(ref v) = self.sse_customer_key {
            extra_headers.push(("x-amz-server-side-encryption-customer-key", v.as_str()));
            computed_key_md5 = super::compute_sse_c_key_md5(v)?;
            extra_headers.push((
                "x-amz-server-side-encryption-customer-key-md5",
                &computed_key_md5,
            ));
        }

        let part_number_str;
        let mut query_params = Vec::new();
        if let Some(pn) = self.part_number {
            if !(1..=10000).contains(&pn) {
                return Err(Error::InvalidInput(
                    "part_number must be between 1 and 10000".to_string(),
                ));
            }
            part_number_str = pn.to_string();
            query_params.push(("partNumber", part_number_str.as_str()));
        }
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
            "GET",
            bucket,
            key,
            &extra_headers,
            b"",
            query,
        ))
    }

    /// レスポンスをパースする (Sans I/O)
    pub fn parse_response(response: &super::S3Response) -> Result<GetObjectOutput, Error> {
        if response.status_code == 304 {
            return Err(Error::NotModified);
        }
        if response.status_code == 412 {
            return Err(Error::PreconditionFailed);
        }
        if !response.is_success() {
            return Err(parse_error_response(response));
        }

        Ok(GetObjectOutput {
            body: response.body.clone(),
            content_type: response.get_header("content-type").map(String::from),
            content_length: response.content_length().map(|v| v as i64),
            e_tag: response.get_header("etag").map(String::from),
            last_modified: response.get_header("last-modified").map(String::from),
            version_id: response.get_header("x-amz-version-id").map(String::from),
            metadata: response.extract_metadata(),
        })
    }

    /// Presigned リクエストを生成する (Sans I/O)
    pub fn presigned(self, expires_in_secs: u64) -> Result<super::PresignedRequest, Error> {
        validate_presign_expires(expires_in_secs)?;
        let bucket = required(self.bucket.as_deref(), "bucket")?;
        let key = required(self.key.as_deref(), "key")?;

        let part_number_str;
        let mut extra_query_params = Vec::new();
        if let Some(pn) = self.part_number {
            part_number_str = pn.to_string();
            extra_query_params.push(("partNumber", part_number_str.as_str()));
        }
        if let Some(ref v) = self.version_id {
            extra_query_params.push(("versionId", v.as_str()));
        }

        let mut extra_headers = Vec::new();
        if let Some(ref v) = self.sse_customer_algorithm {
            extra_headers.push((
                "x-amz-server-side-encryption-customer-algorithm",
                v.as_str(),
            ));
        }
        // SSE-C キーが指定されている場合、MD5 を自動計算する
        let computed_key_md5;
        if let Some(ref v) = self.sse_customer_key {
            extra_headers.push(("x-amz-server-side-encryption-customer-key", v.as_str()));
            computed_key_md5 = super::compute_sse_c_key_md5(v)?;
            extra_headers.push((
                "x-amz-server-side-encryption-customer-key-md5",
                &computed_key_md5,
            ));
        }

        let url = build_presigned_url(
            &self.client.config_ref(),
            "GET",
            bucket,
            key,
            expires_in_secs,
            &extra_query_params,
            &extra_headers,
        );
        let headers = extra_headers
            .iter()
            .map(|&(k, v)| (k.to_string(), v.to_string()))
            .collect();
        Ok(super::PresignedRequest {
            url,
            method: "GET".to_string(),
            headers,
            body: Vec::new(),
        })
    }
}
