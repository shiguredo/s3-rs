//! PutObject API
//!
//! バケットにオブジェクトを追加する。
//! Content-Type や Cache-Control などの System Metadata も指定できる。
//!
//! <https://docs.aws.amazon.com/AmazonS3/latest/API/API_PutObject.html>

use crate::client::S3Client;
use crate::error::Error;
use crate::types::PutObjectOutput;

use super::{
    S3Request, build_presigned_url, build_signed_request, parse_error_response, required,
    validate_presign_expires,
};

pub struct PutObjectFluentBuilder<'a> {
    client: &'a S3Client,
    bucket: Option<String>,
    key: Option<String>,
    body: Option<Vec<u8>>,
    content_type: Option<String>,
    content_encoding: Option<String>,
    content_disposition: Option<String>,
    content_language: Option<String>,
    cache_control: Option<String>,
    expires: Option<String>,
    checksum_algorithm: Option<String>,
    checksum_value: Option<String>,
    /// サーバサイド暗号化 (AES256 または aws:kms)
    server_side_encryption: Option<String>,
    /// SSE-KMS キー ID
    ssekms_key_id: Option<String>,
    /// SSE-C アルゴリズム (AES256)
    sse_customer_algorithm: Option<String>,
    /// SSE-C キー (Base64)
    sse_customer_key: Option<String>,
    /// ACL (private, public-read 等)
    acl: Option<String>,
    /// カスタムメタデータ (x-amz-meta-*)
    metadata: Vec<(String, String)>,
    /// ストレージクラス (STANDARD, STANDARD_IA 等)
    storage_class: Option<String>,
    /// 明示的な Content-Length
    content_length: Option<i64>,
    /// オブジェクトタグ (URL エンコードされたキーバリューペア)
    tagging: Option<String>,
    /// 条件付き書き込み: ETag が一致する場合のみ上書きする
    if_match: Option<String>,
    /// 条件付き書き込み: オブジェクトが存在しない場合のみ作成する ("*")
    if_none_match: Option<String>,
}

impl<'a> PutObjectFluentBuilder<'a> {
    pub(crate) fn new(client: &'a S3Client) -> Self {
        Self {
            client,
            bucket: None,
            key: None,
            body: None,
            content_type: None,
            content_encoding: None,
            content_disposition: None,
            content_language: None,
            cache_control: None,
            expires: None,
            checksum_algorithm: None,
            checksum_value: None,
            server_side_encryption: None,
            ssekms_key_id: None,
            sse_customer_algorithm: None,
            sse_customer_key: None,
            acl: None,
            metadata: Vec::new(),
            storage_class: None,
            content_length: None,
            tagging: None,
            if_match: None,
            if_none_match: None,
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

    pub fn body(mut self, body: Vec<u8>) -> Self {
        self.body = Some(body);
        self
    }

    pub fn content_type(mut self, content_type: impl Into<String>) -> Self {
        self.content_type = Some(content_type.into());
        self
    }

    pub fn content_encoding(mut self, content_encoding: impl Into<String>) -> Self {
        self.content_encoding = Some(content_encoding.into());
        self
    }

    pub fn content_disposition(mut self, content_disposition: impl Into<String>) -> Self {
        self.content_disposition = Some(content_disposition.into());
        self
    }

    pub fn content_language(mut self, content_language: impl Into<String>) -> Self {
        self.content_language = Some(content_language.into());
        self
    }

    pub fn cache_control(mut self, cache_control: impl Into<String>) -> Self {
        self.cache_control = Some(cache_control.into());
        self
    }

    /// HTTP の Expires ヘッダー (RFC 7234 形式)
    pub fn expires(mut self, expires: impl Into<String>) -> Self {
        self.expires = Some(expires.into());
        self
    }

    /// サーバサイド暗号化を指定する (AES256 または aws:kms)
    pub fn server_side_encryption(mut self, sse: impl Into<String>) -> Self {
        self.server_side_encryption = Some(sse.into());
        self
    }

    /// SSE-KMS キー ID を指定する
    pub fn ssekms_key_id(mut self, key_id: impl Into<String>) -> Self {
        self.ssekms_key_id = Some(key_id.into());
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

    /// ACL を指定する (private, public-read, public-read-write 等)
    pub fn acl(mut self, acl: impl Into<String>) -> Self {
        self.acl = Some(acl.into());
        self
    }

    /// カスタムメタデータを追加する (x-amz-meta-{key}: {value})
    pub fn metadata(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.metadata.push((key.into(), value.into()));
        self
    }

    /// ストレージクラスを指定する (STANDARD, STANDARD_IA, GLACIER 等)
    pub fn storage_class(mut self, storage_class: impl Into<String>) -> Self {
        self.storage_class = Some(storage_class.into());
        self
    }

    /// Content-Length を明示的に指定する
    pub fn content_length(mut self, content_length: i64) -> Self {
        self.content_length = Some(content_length);
        self
    }

    /// オブジェクトタグを指定する (URL エンコード形式: "key1=value1&key2=value2")
    pub fn tagging(mut self, tagging: impl Into<String>) -> Self {
        self.tagging = Some(tagging.into());
        self
    }

    /// ETag が一致する場合のみ上書きする (楽観的ロック)
    pub fn if_match(mut self, e_tag: impl Into<String>) -> Self {
        self.if_match = Some(e_tag.into());
        self
    }

    /// オブジェクトが存在しない場合のみ作成する ("*" を指定)
    pub fn if_none_match(mut self, value: impl Into<String>) -> Self {
        self.if_none_match = Some(value.into());
        self
    }

    /// チェックサムアルゴリズムを指定する (CRC32, CRC32C, SHA1, SHA256, CRC64NVME)
    ///
    /// 未指定の場合はデフォルトで CRC32 が使用される。
    /// チェックサムは常に自動計算されてヘッダーに付与される。
    pub fn checksum_algorithm(mut self, algorithm: impl Into<String>) -> Self {
        self.checksum_algorithm = Some(algorithm.into());
        self
    }

    /// 計算済みチェックサム値を指定する (Base64 エンコード)
    pub fn checksum_value(mut self, value: impl Into<String>) -> Self {
        self.checksum_value = Some(value.into());
        self
    }

    pub fn build_request(&self) -> Result<S3Request, Error> {
        let bucket = required(self.bucket.as_deref(), "bucket")?;
        let key = required(self.key.as_deref(), "key")?;
        let body = self.body.as_deref().unwrap_or_default();

        let mut extra_headers = Vec::new();
        if let Some(ref v) = self.acl {
            extra_headers.push(("x-amz-acl", v.as_str()));
        }
        if let Some(ref v) = self.storage_class {
            extra_headers.push(("x-amz-storage-class", v.as_str()));
        }
        if let Some(ref v) = self.tagging {
            extra_headers.push(("x-amz-tagging", v.as_str()));
        }
        if let Some(ref v) = self.if_match {
            extra_headers.push(("if-match", v.as_str()));
        }
        if let Some(ref v) = self.if_none_match {
            extra_headers.push(("if-none-match", v.as_str()));
        }
        if let Some(ref v) = self.content_type {
            extra_headers.push(("content-type", v.as_str()));
        }
        if let Some(ref v) = self.content_encoding {
            extra_headers.push(("content-encoding", v.as_str()));
        }
        if let Some(ref v) = self.content_disposition {
            extra_headers.push(("content-disposition", v.as_str()));
        }
        if let Some(ref v) = self.content_language {
            extra_headers.push(("content-language", v.as_str()));
        }
        if let Some(ref v) = self.cache_control {
            extra_headers.push(("cache-control", v.as_str()));
        }
        if let Some(ref v) = self.expires {
            extra_headers.push(("expires", v.as_str()));
        }
        let content_length_str;
        if let Some(cl) = self.content_length {
            content_length_str = cl.to_string();
            extra_headers.push(("content-length", &content_length_str));
        }
        if let Some(ref v) = self.server_side_encryption {
            extra_headers.push(("x-amz-server-side-encryption", v.as_str()));
        }
        if let Some(ref v) = self.ssekms_key_id {
            extra_headers.push(("x-amz-server-side-encryption-aws-kms-key-id", v.as_str()));
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
        // チェックサムの処理:
        // - checksum_algorithm 指定あり + checksum_value 指定あり → 利用者提供の値を使用する
        // - checksum_algorithm 指定あり + checksum_value 未指定 → ボディから自動計算する
        // - checksum_algorithm 未指定 → デフォルトで CRC32 を自動計算する
        let computed_checksum;
        let algo_str = self.checksum_algorithm.as_deref().unwrap_or("CRC32");
        extra_headers.push(("x-amz-checksum-algorithm", algo_str));
        let algorithm: crate::checksum::ChecksumAlgorithm = algo_str.parse()?;
        if let Some(ref v) = self.checksum_value {
            extra_headers.push((algorithm.header_name(), v.as_str()));
        } else {
            computed_checksum = crate::checksum::compute_checksum(algorithm, body);
            extra_headers.push((algorithm.header_name(), &computed_checksum));
        }

        let meta_headers: Vec<(String, &str)> = self
            .metadata
            .iter()
            .map(|(k, v)| (format!("x-amz-meta-{k}"), v.as_str()))
            .collect();
        for (name, value) in &meta_headers {
            extra_headers.push((name.as_str(), *value));
        }

        Ok(build_signed_request(
            &self.client.config_ref(),
            "PUT",
            bucket,
            key,
            &extra_headers,
            body,
            None,
        ))
    }

    pub fn parse_response(response: &super::S3Response) -> Result<PutObjectOutput, Error> {
        if !response.is_success() {
            return Err(parse_error_response(response));
        }

        Ok(PutObjectOutput {
            e_tag: response.get_header("etag").map(String::from),
            version_id: response.get_header("x-amz-version-id").map(String::from),
        })
    }

    /// Presigned リクエストを生成する (Sans I/O)
    pub fn presigned(self, expires_in_secs: u64) -> Result<super::PresignedRequest, Error> {
        validate_presign_expires(expires_in_secs)?;
        let bucket = required(self.bucket.as_deref(), "bucket")?;
        let key = required(self.key.as_deref(), "key")?;

        let mut extra_headers = Vec::new();
        if let Some(ref v) = self.content_type {
            extra_headers.push(("content-type", v.as_str()));
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
        // チェックサムアルゴリズムが指定されている場合はヘッダーに含める
        // (presigned ではボディがないため自動計算は行わない)
        if let Some(ref v) = self.checksum_algorithm {
            extra_headers.push(("x-amz-checksum-algorithm", v.as_str()));
        }
        if let Some(ref v) = self.checksum_value
            && let Some(ref algo) = self.checksum_algorithm
        {
            let algorithm: crate::checksum::ChecksumAlgorithm = algo.parse()?;
            extra_headers.push((algorithm.header_name(), v.as_str()));
        }

        let url = build_presigned_url(
            &self.client.config_ref(),
            "PUT",
            bucket,
            key,
            expires_in_secs,
            &[],
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
