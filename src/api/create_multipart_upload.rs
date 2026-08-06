//! CreateMultipartUpload API
//!
//! マルチパートアップロードを開始し、UploadId を取得する。
//!
//! <https://docs.aws.amazon.com/AmazonS3/latest/API/API_CreateMultipartUpload.html>

use crate::client::Client;
use crate::error::Error;
use crate::types::{
    ChecksumAlgorithm, CreateMultipartUploadOutput, ObjectCannedAcl, ServerSideEncryption,
    StorageClass,
};

use super::{
    S3Request, build_presigned_url, build_signed_request, parse_error_response, required,
    validate_presign_expires,
};

pub struct CreateMultipartUploadFluentBuilder<'a> {
    client: &'a Client,
    bucket: Option<String>,
    key: Option<String>,
    content_type: Option<String>,
    content_encoding: Option<String>,
    content_disposition: Option<String>,
    content_language: Option<String>,
    cache_control: Option<String>,
    expires: Option<String>,
    /// カスタムメタデータ (x-amz-meta-*)
    metadata: Vec<(String, String)>,
    /// サーバサイド暗号化 (AES256 または aws:kms)
    server_side_encryption: Option<ServerSideEncryption>,
    /// SSE-KMS キー ID
    ssekms_key_id: Option<String>,
    /// SSE-C アルゴリズム (AES256)
    sse_customer_algorithm: Option<String>,
    /// SSE-C キー (Base64)
    sse_customer_key: Option<String>,
    /// ACL (private, public-read 等)
    acl: Option<ObjectCannedAcl>,
    /// ストレージクラス (STANDARD, STANDARD_IA 等)
    storage_class: Option<StorageClass>,
    /// チェックサムアルゴリズム
    checksum_algorithm: Option<ChecksumAlgorithm>,
    /// オブジェクトタグ (URL エンコードされたキーバリューペア)
    tagging: Option<String>,
}

impl<'a> CreateMultipartUploadFluentBuilder<'a> {
    pub(crate) fn new(client: &'a Client) -> Self {
        Self {
            client,
            bucket: None,
            key: None,
            content_type: None,
            content_encoding: None,
            content_disposition: None,
            content_language: None,
            cache_control: None,
            expires: None,
            metadata: Vec::new(),
            server_side_encryption: None,
            ssekms_key_id: None,
            sse_customer_algorithm: None,
            sse_customer_key: None,
            acl: None,
            storage_class: None,
            checksum_algorithm: None,
            tagging: None,
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

    /// カスタムメタデータを追加する (x-amz-meta-{key}: {value})
    pub fn metadata(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.metadata.push((key.into(), value.into()));
        self
    }

    /// サーバサイド暗号化を指定する (AES256 / aws:kms / aws:kms:dsse 等)
    pub fn server_side_encryption(mut self, input: ServerSideEncryption) -> Self {
        self.server_side_encryption = Some(input);
        self
    }

    pub fn set_server_side_encryption(mut self, input: Option<ServerSideEncryption>) -> Self {
        self.server_side_encryption = input;
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
    pub fn acl(mut self, input: ObjectCannedAcl) -> Self {
        self.acl = Some(input);
        self
    }

    pub fn set_acl(mut self, input: Option<ObjectCannedAcl>) -> Self {
        self.acl = input;
        self
    }

    /// ストレージクラスを指定する (STANDARD, STANDARD_IA, GLACIER 等)
    pub fn storage_class(mut self, input: StorageClass) -> Self {
        self.storage_class = Some(input);
        self
    }

    pub fn set_storage_class(mut self, input: Option<StorageClass>) -> Self {
        self.storage_class = input;
        self
    }

    /// チェックサムアルゴリズムを指定する (CRC32, CRC32C, SHA1, SHA256, CRC64NVME)
    ///
    /// マルチパートアップロード全体で使用するチェックサムアルゴリズムを指定する。
    /// ここで指定したアルゴリズムは後続の UploadPart でも使用する。
    pub fn checksum_algorithm(mut self, input: ChecksumAlgorithm) -> Self {
        self.checksum_algorithm = Some(input);
        self
    }

    pub fn set_checksum_algorithm(mut self, input: Option<ChecksumAlgorithm>) -> Self {
        self.checksum_algorithm = input;
        self
    }

    /// オブジェクトタグを指定する (URL エンコード形式: "key1=value1&key2=value2")
    pub fn tagging(mut self, tagging: impl Into<String>) -> Self {
        self.tagging = Some(tagging.into());
        self
    }

    pub fn build_request(&self, now: std::time::SystemTime) -> Result<S3Request, Error> {
        let bucket = required(self.bucket.as_deref(), "bucket")?;
        let key = required(self.key.as_deref(), "key")?;

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
        if let Some(ref ct) = self.content_type {
            extra_headers.push(("content-type", ct.as_str()));
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
        if let Some(ref v) = self.server_side_encryption {
            extra_headers.push(("x-amz-server-side-encryption", v.as_str()));
        }
        if let Some(ref v) = self.ssekms_key_id {
            extra_headers.push(("x-amz-server-side-encryption-aws-kms-key-id", v.as_str()));
        }
        let mut computed_key_md5 = None;
        super::add_sse_c_headers(
            &mut extra_headers,
            self.sse_customer_algorithm.as_deref(),
            self.sse_customer_key.as_deref(),
            &mut computed_key_md5,
            false,
        )?;

        if let Some(ref v) = self.checksum_algorithm {
            // CreateMultipartUpload ではボディがないためヘッダーのみ指定する
            extra_headers.push(("x-amz-checksum-algorithm", v.as_str()));
        }

        // カスタムメタデータ用のヘッダー名を保持する
        let meta_headers: Vec<(String, &str)> = self
            .metadata
            .iter()
            .map(|(k, v)| (format!("x-amz-meta-{k}"), v.as_str()))
            .collect();
        for (name, value) in &meta_headers {
            extra_headers.push((name.as_str(), *value));
        }

        let query_params = [("uploads", "")];
        build_signed_request(
            &self.client.config_ref(),
            "POST",
            bucket,
            key,
            &extra_headers,
            b"",
            Some(&query_params),
            now,
        )
    }

    pub fn parse_response(
        response: &super::S3Response,
    ) -> Result<CreateMultipartUploadOutput, Error> {
        if !response.is_success() {
            return Err(parse_error_response(response));
        }

        let body_text = super::xml_body_text(&response.body)?;

        Ok(CreateMultipartUploadOutput {
            bucket: crate::xml::extract_element(body_text, "Bucket")?,
            key: crate::xml::extract_element(body_text, "Key")?,
            upload_id: crate::xml::extract_element(body_text, "UploadId")?,
            request_charged: response
                .get_header("x-amz-request-charged")
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
        if let Some(ref ct) = self.content_type {
            extra_headers.push(("content-type", ct.as_str()));
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
        if let Some(ref v) = self.server_side_encryption {
            extra_headers.push(("x-amz-server-side-encryption", v.as_str()));
        }
        if let Some(ref v) = self.ssekms_key_id {
            extra_headers.push(("x-amz-server-side-encryption-aws-kms-key-id", v.as_str()));
        }
        let mut computed_key_md5 = None;
        super::add_sse_c_headers(
            &mut extra_headers,
            self.sse_customer_algorithm.as_deref(),
            self.sse_customer_key.as_deref(),
            &mut computed_key_md5,
            false,
        )?;
        if let Some(ref v) = self.checksum_algorithm {
            extra_headers.push(("x-amz-checksum-algorithm", v.as_str()));
        }
        // カスタムメタデータ用のヘッダー名を保持する
        let meta_headers: Vec<(String, &str)> = self
            .metadata
            .iter()
            .map(|(k, v)| (format!("x-amz-meta-{k}"), v.as_str()))
            .collect();
        for (name, value) in &meta_headers {
            extra_headers.push((name.as_str(), *value));
        }

        let url = build_presigned_url(
            &self.client.config_ref(),
            "POST",
            bucket,
            key,
            expires_in_secs,
            &[("uploads", "")],
            &extra_headers,
            now,
        )?;
        let headers = extra_headers
            .iter()
            .map(|&(k, v)| (k.to_string(), v.to_string()))
            .collect();
        Ok(super::PresignedRequest {
            url,
            method: "POST".to_string(),
            headers,
            body: Vec::new(),
        })
    }
}
