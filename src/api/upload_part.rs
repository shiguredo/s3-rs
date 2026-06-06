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
    /// CRC32 チェックサム (Base64) - 個別指定
    checksum_crc32: Option<String>,
    checksum_crc32_c: Option<String>,
    checksum_crc64_nvme: Option<String>,
    checksum_md5: Option<String>,
    checksum_sha1: Option<String>,
    checksum_sha256: Option<String>,
    checksum_sha512: Option<String>,
    checksum_xxhash128: Option<String>,
    checksum_xxhash3: Option<String>,
    checksum_xxhash64: Option<String>,
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
            checksum_crc32: None,
            checksum_crc32_c: None,
            checksum_crc64_nvme: None,
            checksum_md5: None,
            checksum_sha1: None,
            checksum_sha256: None,
            checksum_sha512: None,
            checksum_xxhash128: None,
            checksum_xxhash3: None,
            checksum_xxhash64: None,
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
    ///
    /// UploadPart では個別 `checksum_*` フィールドが指定された場合、
    /// S3 仕様によりこの値は無視される (本クライアントも `x-amz-sdk-checksum-algorithm`
    /// ヘッダーを送信しない)。
    pub fn checksum_algorithm(mut self, input: ChecksumAlgorithm) -> Self {
        self.checksum_algorithm = Some(input);
        self
    }

    pub fn set_checksum_algorithm(mut self, input: Option<ChecksumAlgorithm>) -> Self {
        self.checksum_algorithm = input;
        self
    }

    /// CRC32 チェックサム (Base64) を直接指定する
    pub fn checksum_crc32(mut self, input: impl Into<String>) -> Self {
        self.checksum_crc32 = Some(input.into());
        self
    }
    pub fn set_checksum_crc32(mut self, input: Option<String>) -> Self {
        self.checksum_crc32 = input;
        self
    }
    /// CRC32C チェックサム (Base64) を直接指定する
    pub fn checksum_crc32_c(mut self, input: impl Into<String>) -> Self {
        self.checksum_crc32_c = Some(input.into());
        self
    }
    pub fn set_checksum_crc32_c(mut self, input: Option<String>) -> Self {
        self.checksum_crc32_c = input;
        self
    }
    /// CRC64NVME チェックサム (Base64) を直接指定する
    pub fn checksum_crc64_nvme(mut self, input: impl Into<String>) -> Self {
        self.checksum_crc64_nvme = Some(input.into());
        self
    }
    pub fn set_checksum_crc64_nvme(mut self, input: Option<String>) -> Self {
        self.checksum_crc64_nvme = input;
        self
    }
    /// MD5 チェックサム (Base64) を直接指定する
    pub fn checksum_md5(mut self, input: impl Into<String>) -> Self {
        self.checksum_md5 = Some(input.into());
        self
    }
    pub fn set_checksum_md5(mut self, input: Option<String>) -> Self {
        self.checksum_md5 = input;
        self
    }
    /// SHA1 チェックサム (Base64) を直接指定する
    pub fn checksum_sha1(mut self, input: impl Into<String>) -> Self {
        self.checksum_sha1 = Some(input.into());
        self
    }
    pub fn set_checksum_sha1(mut self, input: Option<String>) -> Self {
        self.checksum_sha1 = input;
        self
    }
    /// SHA256 チェックサム (Base64) を直接指定する
    pub fn checksum_sha256(mut self, input: impl Into<String>) -> Self {
        self.checksum_sha256 = Some(input.into());
        self
    }
    pub fn set_checksum_sha256(mut self, input: Option<String>) -> Self {
        self.checksum_sha256 = input;
        self
    }
    /// SHA512 チェックサム (Base64) を直接指定する
    pub fn checksum_sha512(mut self, input: impl Into<String>) -> Self {
        self.checksum_sha512 = Some(input.into());
        self
    }
    pub fn set_checksum_sha512(mut self, input: Option<String>) -> Self {
        self.checksum_sha512 = input;
        self
    }
    /// XXHASH128 チェックサム (Base64) を直接指定する
    pub fn checksum_xxhash128(mut self, input: impl Into<String>) -> Self {
        self.checksum_xxhash128 = Some(input.into());
        self
    }
    pub fn set_checksum_xxhash128(mut self, input: Option<String>) -> Self {
        self.checksum_xxhash128 = input;
        self
    }
    /// XXHASH3 チェックサム (Base64) を直接指定する
    pub fn checksum_xxhash3(mut self, input: impl Into<String>) -> Self {
        self.checksum_xxhash3 = Some(input.into());
        self
    }
    pub fn set_checksum_xxhash3(mut self, input: Option<String>) -> Self {
        self.checksum_xxhash3 = input;
        self
    }
    /// XXHASH64 チェックサム (Base64) を直接指定する
    pub fn checksum_xxhash64(mut self, input: impl Into<String>) -> Self {
        self.checksum_xxhash64 = Some(input.into());
        self
    }
    pub fn set_checksum_xxhash64(mut self, input: Option<String>) -> Self {
        self.checksum_xxhash64 = input;
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

    fn any_individual(&self) -> bool {
        self.checksum_crc32.is_some()
            || self.checksum_crc32_c.is_some()
            || self.checksum_crc64_nvme.is_some()
            || self.checksum_md5.is_some()
            || self.checksum_sha1.is_some()
            || self.checksum_sha256.is_some()
            || self.checksum_sha512.is_some()
            || self.checksum_xxhash128.is_some()
            || self.checksum_xxhash3.is_some()
            || self.checksum_xxhash64.is_some()
    }

    fn push_individual_checksum_headers<'b>(&'b self, extra_headers: &mut Vec<(&'b str, &'b str)>) {
        if let Some(ref v) = self.checksum_crc32 {
            extra_headers.push(("x-amz-checksum-crc32", v.as_str()));
        }
        if let Some(ref v) = self.checksum_crc32_c {
            extra_headers.push(("x-amz-checksum-crc32c", v.as_str()));
        }
        if let Some(ref v) = self.checksum_crc64_nvme {
            extra_headers.push(("x-amz-checksum-crc64nvme", v.as_str()));
        }
        if let Some(ref v) = self.checksum_md5 {
            extra_headers.push(("x-amz-checksum-md5", v.as_str()));
        }
        if let Some(ref v) = self.checksum_sha1 {
            extra_headers.push(("x-amz-checksum-sha1", v.as_str()));
        }
        if let Some(ref v) = self.checksum_sha256 {
            extra_headers.push(("x-amz-checksum-sha256", v.as_str()));
        }
        if let Some(ref v) = self.checksum_sha512 {
            extra_headers.push(("x-amz-checksum-sha512", v.as_str()));
        }
        if let Some(ref v) = self.checksum_xxhash128 {
            extra_headers.push(("x-amz-checksum-xxhash128", v.as_str()));
        }
        if let Some(ref v) = self.checksum_xxhash3 {
            extra_headers.push(("x-amz-checksum-xxhash3", v.as_str()));
        }
        if let Some(ref v) = self.checksum_xxhash64 {
            extra_headers.push(("x-amz-checksum-xxhash64", v.as_str()));
        }
    }

    pub fn build_request(&self, now: std::time::SystemTime) -> Result<S3Request, Error> {
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

        // チェックサムの処理 (UploadPart 仕様):
        // - 個別 checksum_* 指定あり: 該当ヘッダーに値を設定。S3 が ChecksumAlgorithm
        //   parameter を無視するため、checksum_algorithm 指定があっても
        //   x-amz-sdk-checksum-algorithm ヘッダーは送信しない。
        // - 個別未指定 + checksum_algorithm 指定: 該当アルゴリズムで自動計算。
        // - 全て未指定: デフォルトの CRC32 で自動計算。
        let mut extra_headers: Vec<(&str, &str)> = Vec::new();
        let content_length_str;
        if let Some(cl) = self.content_length {
            content_length_str = cl.to_string();
            extra_headers.push(("content-length", content_length_str.as_str()));
        }

        let any_individual = self.any_individual();
        let computed_checksum_default;
        if any_individual {
            self.push_individual_checksum_headers(&mut extra_headers);
            // checksum_algorithm 指定は S3 が無視するため、本クライアントも送信しない。
        } else {
            let default_algorithm_owned = ChecksumAlgorithm::Crc32;
            let algorithm = self
                .checksum_algorithm
                .as_ref()
                .unwrap_or(&default_algorithm_owned);
            // checksum_algorithm 未指定でデフォルト CRC32 を使う場合のみ
            // x-amz-sdk-checksum-algorithm ヘッダーを追加 (CRC32 は &'static str)
            if let Some(ref alg) = self.checksum_algorithm {
                extra_headers.push(("x-amz-sdk-checksum-algorithm", alg.as_str()));
            } else {
                extra_headers.push(("x-amz-sdk-checksum-algorithm", "CRC32"));
            }
            let header_name = crate::checksum::header_name(algorithm)?;
            computed_checksum_default = crate::checksum::compute_checksum(algorithm, body)?;
            extra_headers.push((header_name, &computed_checksum_default));
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

        build_signed_request(
            &self.client.config_ref(),
            "PUT",
            bucket,
            key,
            &extra_headers,
            body,
            Some(&query_params),
            now,
        )
    }

    pub fn parse_response(response: &super::S3Response) -> Result<UploadPartOutput, Error> {
        if !response.is_success() {
            return Err(parse_error_response(response));
        }

        Ok(UploadPartOutput {
            e_tag: response.get_header("etag").map(String::from),
            server_side_encryption: response
                .get_header("x-amz-server-side-encryption")
                .map(crate::types::ServerSideEncryption::from),
            sse_customer_algorithm: response
                .get_header("x-amz-server-side-encryption-customer-algorithm")
                .map(String::from),
            sse_customer_key_md5: response
                .get_header("x-amz-server-side-encryption-customer-key-md5")
                .map(String::from),
            ssekms_key_id: response
                .get_header("x-amz-server-side-encryption-aws-kms-key-id")
                .map(String::from),
            bucket_key_enabled: response
                .get_header("x-amz-server-side-encryption-bucket-key-enabled")
                .and_then(|s| s.parse::<bool>().ok()),
            request_charged: response
                .get_header("x-amz-request-charged")
                .map(String::from),
            checksum_crc32: response
                .get_header("x-amz-checksum-crc32")
                .map(String::from),
            checksum_crc32_c: response
                .get_header("x-amz-checksum-crc32c")
                .map(String::from),
            checksum_crc64_nvme: response
                .get_header("x-amz-checksum-crc64nvme")
                .map(String::from),
            checksum_sha1: response.get_header("x-amz-checksum-sha1").map(String::from),
            checksum_sha256: response
                .get_header("x-amz-checksum-sha256")
                .map(String::from),
        })
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

        let mut extra_headers: Vec<(&str, &str)> = Vec::new();
        let content_length_str;
        if let Some(cl) = self.content_length {
            content_length_str = cl.to_string();
            extra_headers.push(("content-length", content_length_str.as_str()));
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
        // チェックサムの処理 (presigned はボディがないため自動計算しない):
        // 個別フィールドが指定されていれば該当ヘッダーをすべて設定する。
        // checksum_algorithm は個別指定が無いときのみ x-amz-sdk-checksum-algorithm として送信する
        // (個別指定がある場合は S3 が無視するため、付与しないのが UploadPart の仕様)。
        let any_individual = self.any_individual();
        if any_individual {
            self.push_individual_checksum_headers(&mut extra_headers);
        } else if let Some(ref v) = self.checksum_algorithm {
            extra_headers.push(("x-amz-sdk-checksum-algorithm", v.as_str()));
        }

        let url = build_presigned_url(
            &self.client.config_ref(),
            "PUT",
            bucket,
            key,
            expires_in_secs,
            &extra_query_params,
            &extra_headers,
            now,
        )?;
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
