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
    response_cache_control: Option<String>,
    response_content_disposition: Option<String>,
    response_content_encoding: Option<String>,
    response_content_language: Option<String>,
    response_content_type: Option<String>,
    response_expires: Option<String>,
    checksum_mode: Option<String>,
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
            response_cache_control: None,
            response_content_disposition: None,
            response_content_encoding: None,
            response_content_language: None,
            response_content_type: None,
            response_expires: None,
            checksum_mode: None,
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

    /// レスポンスの Cache-Control ヘッダーを上書きする
    pub fn response_cache_control(mut self, value: impl Into<String>) -> Self {
        self.response_cache_control = Some(value.into());
        self
    }

    /// レスポンスの Content-Disposition ヘッダーを上書きする
    pub fn response_content_disposition(mut self, value: impl Into<String>) -> Self {
        self.response_content_disposition = Some(value.into());
        self
    }

    /// レスポンスの Content-Encoding ヘッダーを上書きする
    pub fn response_content_encoding(mut self, value: impl Into<String>) -> Self {
        self.response_content_encoding = Some(value.into());
        self
    }

    /// レスポンスの Content-Language ヘッダーを上書きする
    pub fn response_content_language(mut self, value: impl Into<String>) -> Self {
        self.response_content_language = Some(value.into());
        self
    }

    /// レスポンスの Content-Type ヘッダーを上書きする
    pub fn response_content_type(mut self, value: impl Into<String>) -> Self {
        self.response_content_type = Some(value.into());
        self
    }

    /// レスポンスの Expires ヘッダーを上書きする
    pub fn response_expires(mut self, value: impl Into<String>) -> Self {
        self.response_expires = Some(value.into());
        self
    }

    /// チェックサムモードを指定する ("ENABLED")
    ///
    /// ENABLED を指定するとレスポンスにチェックサム値が含まれる。
    pub fn checksum_mode(mut self, mode: impl Into<String>) -> Self {
        self.checksum_mode = Some(mode.into());
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
        if let Some(ref v) = self.checksum_mode {
            extra_headers.push(("x-amz-checksum-mode", v.as_str()));
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
        if let Some(ref v) = self.response_cache_control {
            query_params.push(("response-cache-control", v.as_str()));
        }
        if let Some(ref v) = self.response_content_disposition {
            query_params.push(("response-content-disposition", v.as_str()));
        }
        if let Some(ref v) = self.response_content_encoding {
            query_params.push(("response-content-encoding", v.as_str()));
        }
        if let Some(ref v) = self.response_content_language {
            query_params.push(("response-content-language", v.as_str()));
        }
        if let Some(ref v) = self.response_content_type {
            query_params.push(("response-content-type", v.as_str()));
        }
        if let Some(ref v) = self.response_expires {
            query_params.push(("response-expires", v.as_str()));
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
            checksum_crc32: response
                .get_header("x-amz-checksum-crc32")
                .map(String::from),
            checksum_crc32c: response
                .get_header("x-amz-checksum-crc32c")
                .map(String::from),
            checksum_crc64nvme: response
                .get_header("x-amz-checksum-crc64nvme")
                .map(String::from),
            checksum_sha1: response.get_header("x-amz-checksum-sha1").map(String::from),
            checksum_sha256: response
                .get_header("x-amz-checksum-sha256")
                .map(String::from),
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
        if let Some(ref v) = self.response_cache_control {
            extra_query_params.push(("response-cache-control", v.as_str()));
        }
        if let Some(ref v) = self.response_content_disposition {
            extra_query_params.push(("response-content-disposition", v.as_str()));
        }
        if let Some(ref v) = self.response_content_encoding {
            extra_query_params.push(("response-content-encoding", v.as_str()));
        }
        if let Some(ref v) = self.response_content_language {
            extra_query_params.push(("response-content-language", v.as_str()));
        }
        if let Some(ref v) = self.response_content_type {
            extra_query_params.push(("response-content-type", v.as_str()));
        }
        if let Some(ref v) = self.response_expires {
            extra_query_params.push(("response-expires", v.as_str()));
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
