//! PutObject API
//!
//! バケットにオブジェクトを追加する。
//! Content-Type や Cache-Control などの System Metadata も指定できる。
//!
//! <https://docs.aws.amazon.com/AmazonS3/latest/API/API_PutObject.html>

use crate::client::Client;
use crate::error::Error;
use crate::types::{
    ChecksumAlgorithm, ObjectCannedAcl, PutObjectOutput, ServerSideEncryption, StorageClass,
};

use super::{
    S3Request, build_presigned_url, build_signed_request, parse_error_response, required,
    validate_presign_expires,
};

pub struct PutObjectFluentBuilder<'a> {
    client: &'a Client,
    bucket: Option<String>,
    key: Option<String>,
    body: Option<Vec<u8>>,
    content_type: Option<String>,
    content_encoding: Option<String>,
    content_disposition: Option<String>,
    content_language: Option<String>,
    cache_control: Option<String>,
    expires: Option<String>,
    checksum_algorithm: Option<ChecksumAlgorithm>,
    /// CRC32 チェックサム (Base64) - 個別指定
    checksum_crc32: Option<String>,
    /// CRC32C チェックサム (Base64) - 個別指定
    checksum_crc32_c: Option<String>,
    /// CRC64NVME チェックサム (Base64) - 個別指定
    checksum_crc64_nvme: Option<String>,
    /// MD5 チェックサム (Base64) - 個別指定 (内部計算未対応)
    checksum_md5: Option<String>,
    /// SHA1 チェックサム (Base64) - 個別指定
    checksum_sha1: Option<String>,
    /// SHA256 チェックサム (Base64) - 個別指定
    checksum_sha256: Option<String>,
    /// SHA512 チェックサム (Base64) - 個別指定 (内部計算未対応)
    checksum_sha512: Option<String>,
    /// XXHASH128 チェックサム (Base64) - 個別指定 (内部計算未対応)
    checksum_xxhash128: Option<String>,
    /// XXHASH3 チェックサム (Base64) - 個別指定 (内部計算未対応)
    checksum_xxhash3: Option<String>,
    /// XXHASH64 チェックサム (Base64) - 個別指定 (内部計算未対応)
    checksum_xxhash64: Option<String>,
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
    /// カスタムメタデータ (x-amz-meta-*)
    metadata: Vec<(String, String)>,
    /// ストレージクラス (STANDARD, STANDARD_IA 等)
    storage_class: Option<StorageClass>,
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
    pub(crate) fn new(client: &'a Client) -> Self {
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

    /// サーバサイド暗号化を指定する (AES256 / aws:kms / aws:kms:dsse 等)
    pub fn server_side_encryption(mut self, input: ServerSideEncryption) -> Self {
        self.server_side_encryption = Some(input);
        self
    }

    /// サーバサイド暗号化を Option で設定する (`set_*` バリアント)
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

    /// ACL を Option で設定する (`set_*` バリアント)
    pub fn set_acl(mut self, input: Option<ObjectCannedAcl>) -> Self {
        self.acl = input;
        self
    }

    /// カスタムメタデータを追加する (x-amz-meta-{key}: {value})
    pub fn metadata(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.metadata.push((key.into(), value.into()));
        self
    }

    /// ストレージクラスを指定する (STANDARD, STANDARD_IA, GLACIER 等)
    pub fn storage_class(mut self, input: StorageClass) -> Self {
        self.storage_class = Some(input);
        self
    }

    /// ストレージクラスを Option で設定する (`set_*` バリアント)
    pub fn set_storage_class(mut self, input: Option<StorageClass>) -> Self {
        self.storage_class = input;
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
    pub fn checksum_algorithm(mut self, input: ChecksumAlgorithm) -> Self {
        self.checksum_algorithm = Some(input);
        self
    }

    /// チェックサムアルゴリズムを Option で設定する (`set_*` バリアント)
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

    pub fn build_request(&self, now: std::time::SystemTime) -> Result<S3Request, Error> {
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
        // ChecksumAlgorithm は as_str() が `&str` を返すので enum でも従来通り使える
        // (acl / storage_class / server_side_encryption も同様)
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
        // チェックサムの処理 (PutObject 仕様):
        // - 個別 checksum_* フィールド指定あり: 該当ヘッダー (x-amz-checksum-{algo}) に値を設定
        // - checksum_algorithm 指定あり: x-amz-sdk-checksum-algorithm ヘッダーに設定
        //   (個別フィールドとの整合性チェックは S3 サーバ側に任せる、aws-sdk-rust と同じ挙動)
        // - 個別フィールドも checksum_algorithm も未指定: デフォルトで CRC32 を自動計算する
        let computed_checksum_default;
        let any_individual = self.checksum_crc32.is_some()
            || self.checksum_crc32_c.is_some()
            || self.checksum_crc64_nvme.is_some()
            || self.checksum_md5.is_some()
            || self.checksum_sha1.is_some()
            || self.checksum_sha256.is_some()
            || self.checksum_sha512.is_some()
            || self.checksum_xxhash128.is_some()
            || self.checksum_xxhash3.is_some()
            || self.checksum_xxhash64.is_some();
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
        if let Some(ref algorithm) = self.checksum_algorithm {
            extra_headers.push(("x-amz-sdk-checksum-algorithm", algorithm.as_str()));
        }
        if !any_individual {
            // 個別フィールド未指定。checksum_algorithm 指定があればそれを、なければ
            // デフォルトの CRC32 を自動計算する。
            let default_algorithm_owned = ChecksumAlgorithm::Crc32;
            let algorithm = self
                .checksum_algorithm
                .as_ref()
                .unwrap_or(&default_algorithm_owned);
            // checksum_algorithm 指定があれば既に上で push 済み。未指定なら追加する。
            if self.checksum_algorithm.is_none() {
                extra_headers.push(("x-amz-sdk-checksum-algorithm", "CRC32"));
            }
            let header_name = crate::checksum::header_name(algorithm)?;
            computed_checksum_default = crate::checksum::compute_checksum(algorithm, body)?;
            extra_headers.push((header_name, &computed_checksum_default));
        }

        let meta_headers: Vec<(String, &str)> = self
            .metadata
            .iter()
            .map(|(k, v)| (format!("x-amz-meta-{k}"), v.as_str()))
            .collect();
        for (name, value) in &meta_headers {
            extra_headers.push((name.as_str(), *value));
        }

        build_signed_request(
            &self.client.config_ref(),
            "PUT",
            bucket,
            key,
            &extra_headers,
            body,
            None,
            now,
        )
    }

    pub fn parse_response(response: &super::S3Response) -> Result<PutObjectOutput, Error> {
        if !response.is_success() {
            return Err(parse_error_response(response));
        }

        Ok(PutObjectOutput {
            e_tag: response.get_header("etag").map(String::from),
            version_id: response.get_header("x-amz-version-id").map(String::from),
            expiration: response.get_header("x-amz-expiration").map(String::from),
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
            checksum_type: response.get_header("x-amz-checksum-type").map(String::from),
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
        // チェックサムの処理 (presigned はボディがないため自動計算しない)
        if let Some(ref v) = self.checksum_algorithm {
            extra_headers.push(("x-amz-sdk-checksum-algorithm", v.as_str()));
        }
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

        let url = build_presigned_url(
            &self.client.config_ref(),
            "PUT",
            bucket,
            key,
            expires_in_secs,
            &[],
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
