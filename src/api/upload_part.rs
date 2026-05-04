//! UploadPart API
//!
//! マルチパートアップロードのパートをアップロードする。
//! UploadId と PartNumber の指定が必要。
//!
//! <https://docs.aws.amazon.com/AmazonS3/latest/API/API_UploadPart.html>

use crate::client::Client;
use crate::error::Error;
use crate::types::{ChecksumAlgorithm, UploadPartOutput};

use super::{
    S3Request, build_presigned_url, build_signed_request, parse_error_response, required,
    validate_presign_expires,
};

pub struct UploadPartFluentBuilder<'a> {
    client: &'a Client,
    bucket: Option<String>,
    key: Option<String>,
    upload_id: Option<String>,
    part_number: Option<i32>,
    body: Option<Vec<u8>>,
    checksum_algorithm: Option<ChecksumAlgorithm>,
    checksum_value: Option<String>,
    /// SSE-C アルゴリズム (AES256)
    sse_customer_algorithm: Option<String>,
    /// SSE-C キー (Base64)
    sse_customer_key: Option<String>,
    /// 明示的な Content-Length
    content_length: Option<i64>,
}

impl<'a> UploadPartFluentBuilder<'a> {
    pub(crate) fn new(client: &'a Client) -> Self {
        Self {
            client,
            bucket: None,
            key: None,
            upload_id: None,
            part_number: None,
            body: None,
            checksum_algorithm: None,
            checksum_value: None,
            sse_customer_algorithm: None,
            sse_customer_key: None,
            content_length: None,
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

    pub fn part_number(mut self, part_number: i32) -> Self {
        self.part_number = Some(part_number);
        self
    }

    pub fn body(mut self, body: Vec<u8>) -> Self {
        self.body = Some(body);
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

    /// 計算済みチェックサム値を指定する (Base64 エンコード)
    pub fn checksum_value(mut self, value: impl Into<String>) -> Self {
        self.checksum_value = Some(value.into());
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

    /// Content-Length を明示的に指定する
    pub fn content_length(mut self, content_length: i64) -> Self {
        self.content_length = Some(content_length);
        self
    }

    pub fn build_request(&self) -> Result<S3Request, Error> {
        let bucket = required(self.bucket.as_deref(), "bucket")?;
        let key = required(self.key.as_deref(), "key")?;
        let upload_id = required(self.upload_id.as_deref(), "upload_id")?;
        let part_number = self
            .part_number
            .ok_or_else(|| Error::InvalidInput("part_number is required".into()))?;
        if !(1..=10000).contains(&part_number) {
            return Err(Error::InvalidInput(
                "part_number must be between 1 and 10000".to_string(),
            ));
        }
        let body = self.body.as_deref().unwrap_or_default();

        let part_number_str = part_number.to_string();
        let query_params = [
            ("partNumber", part_number_str.as_str()),
            ("uploadId", upload_id),
        ];

        // チェックサムの処理:
        // - checksum_algorithm 指定あり + checksum_value 指定あり → 利用者提供の値を使用する
        // - checksum_algorithm 指定あり + checksum_value 未指定 → ボディから自動計算する
        // - checksum_algorithm 未指定 → デフォルトで CRC32 を自動計算する
        let computed_checksum;
        let mut extra_headers = Vec::new();
        let content_length_str;
        if let Some(cl) = self.content_length {
            content_length_str = cl.to_string();
            extra_headers.push(("content-length", content_length_str.as_str()));
        }
        let default_algorithm = ChecksumAlgorithm::Crc32;
        let algorithm = self
            .checksum_algorithm
            .as_ref()
            .unwrap_or(&default_algorithm);
        extra_headers.push(("x-amz-checksum-algorithm", algorithm.as_str()));
        let header_name = crate::checksum::header_name(algorithm)?;
        if let Some(ref v) = self.checksum_value {
            extra_headers.push((header_name, v.as_str()));
        } else {
            computed_checksum = crate::checksum::compute_checksum(algorithm, body)?;
            extra_headers.push((header_name, &computed_checksum));
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

        Ok(build_signed_request(
            &self.client.config_ref(),
            "PUT",
            bucket,
            key,
            &extra_headers,
            body,
            Some(&query_params),
        ))
    }

    pub fn parse_response(response: &super::S3Response) -> Result<UploadPartOutput, Error> {
        if !response.is_success() {
            return Err(parse_error_response(response));
        }

        Ok(UploadPartOutput {
            e_tag: response.get_header("etag").map(String::from),
        })
    }

    /// Presigned リクエストを生成する (Sans I/O)
    pub fn presigned(self, expires_in_secs: u64) -> Result<super::PresignedRequest, Error> {
        validate_presign_expires(expires_in_secs)?;
        let bucket = required(self.bucket.as_deref(), "bucket")?;
        let key = required(self.key.as_deref(), "key")?;
        let upload_id = required(self.upload_id.as_deref(), "upload_id")?;
        let part_number = self
            .part_number
            .ok_or_else(|| Error::InvalidInput("part_number is required".into()))?;
        if !(1..=10000).contains(&part_number) {
            return Err(Error::InvalidInput(
                "part_number must be between 1 and 10000".to_string(),
            ));
        }

        let part_number_str = part_number.to_string();
        let extra_query_params = [
            ("partNumber", part_number_str.as_str()),
            ("uploadId", upload_id),
        ];

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
        // チェックサムアルゴリズムが指定されている場合はヘッダーに含める
        // (presigned ではボディがないため自動計算は行わない)
        if let Some(ref v) = self.checksum_algorithm {
            extra_headers.push(("x-amz-checksum-algorithm", v.as_str()));
        }
        if let Some(ref v) = self.checksum_value
            && let Some(ref algorithm) = self.checksum_algorithm
        {
            let header_name = crate::checksum::header_name(algorithm)?;
            extra_headers.push((header_name, v.as_str()));
        }

        let url = build_presigned_url(
            &self.client.config_ref(),
            "PUT",
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
            method: "PUT".to_string(),
            headers,
            body: Vec::new(),
        })
    }
}
