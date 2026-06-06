//! CopyObject API
//!
//! 既存のオブジェクトをコピーする。
//! MetadataDirective を REPLACE に設定するとメタデータを置換できる。
//!
//! <https://docs.aws.amazon.com/AmazonS3/latest/API/API_CopyObject.html>

use crate::client::Client;
use crate::error::Error;
use crate::types::{
    ChecksumAlgorithm, CopyObjectOutput, MetadataDirective, ObjectCannedAcl, ServerSideEncryption,
    StorageClass, TaggingDirective,
};

use super::{S3Request, build_signed_request, check_body_error, parse_error_response, required};

pub struct CopyObjectFluentBuilder<'a> {
    client: &'a Client,
    bucket: Option<String>,
    key: Option<String>,
    copy_source: Option<String>,
    metadata_directive: Option<MetadataDirective>,
    content_type: Option<String>,
    content_encoding: Option<String>,
    content_disposition: Option<String>,
    content_language: Option<String>,
    cache_control: Option<String>,
    expires: Option<String>,
    /// コピー先のサーバサイド暗号化 (AES256 または aws:kms)
    server_side_encryption: Option<ServerSideEncryption>,
    /// コピー先の SSE-KMS キー ID
    ssekms_key_id: Option<String>,
    /// コピー先の SSE-C アルゴリズム (AES256)
    sse_customer_algorithm: Option<String>,
    /// コピー先の SSE-C キー (Base64)
    sse_customer_key: Option<String>,
    /// コピー元の SSE-C アルゴリズム (AES256)
    copy_source_sse_customer_algorithm: Option<String>,
    /// コピー元の SSE-C キー (Base64)
    copy_source_sse_customer_key: Option<String>,
    /// ACL (private, public-read 等)
    acl: Option<ObjectCannedAcl>,
    /// カスタムメタデータ (x-amz-meta-*)
    metadata: Vec<(String, String)>,
    /// ストレージクラス (STANDARD, STANDARD_IA 等)
    storage_class: Option<StorageClass>,
    /// チェックサムアルゴリズム
    checksum_algorithm: Option<ChecksumAlgorithm>,
    /// オブジェクトタグ (URL エンコードされたキーバリューペア)
    tagging: Option<String>,
    /// タグディレクティブ (COPY または REPLACE)
    tagging_directive: Option<TaggingDirective>,
    /// 条件付きコピー: コピー元の ETag が一致する場合のみコピーする
    copy_source_if_match: Option<String>,
    /// 条件付きコピー: コピー元の ETag が異なる場合のみコピーする
    copy_source_if_none_match: Option<String>,
    /// 条件付きコピー: コピー元が指定日時以降に変更されている場合のみコピーする
    copy_source_if_modified_since: Option<String>,
    /// 条件付きコピー: コピー元が指定日時以降に変更されていない場合のみコピーする
    copy_source_if_unmodified_since: Option<String>,
}

impl<'a> CopyObjectFluentBuilder<'a> {
    pub(crate) fn new(client: &'a Client) -> Self {
        Self {
            client,
            bucket: None,
            key: None,
            copy_source: None,
            metadata_directive: None,
            content_type: None,
            content_encoding: None,
            content_disposition: None,
            content_language: None,
            cache_control: None,
            expires: None,
            server_side_encryption: None,
            ssekms_key_id: None,
            sse_customer_algorithm: None,
            sse_customer_key: None,
            acl: None,
            metadata: Vec::new(),
            storage_class: None,
            copy_source_sse_customer_algorithm: None,
            copy_source_sse_customer_key: None,
            checksum_algorithm: None,
            tagging: None,
            tagging_directive: None,
            copy_source_if_match: None,
            copy_source_if_none_match: None,
            copy_source_if_modified_since: None,
            copy_source_if_unmodified_since: None,
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

    /// コピー元を "bucket/key" 形式で指定する
    ///
    /// エンコードせずそのまま x-amz-copy-source ヘッダーに設定する。
    /// キーに特殊文字が含まれる場合は、利用者側で URL エンコード済みの値を渡すこと。
    pub fn copy_source(mut self, copy_source: impl Into<String>) -> Self {
        self.copy_source = Some(copy_source.into());
        self
    }

    /// メタデータディレクティブ (COPY または REPLACE)
    ///
    /// `REPLACE` を指定するとコピー先のメタデータをこのリクエストで指定した値に置き換える。
    /// デフォルトは `COPY` (コピー元のメタデータをそのまま引き継ぐ)。
    pub fn metadata_directive(mut self, input: MetadataDirective) -> Self {
        self.metadata_directive = Some(input);
        self
    }

    pub fn set_metadata_directive(mut self, input: Option<MetadataDirective>) -> Self {
        self.metadata_directive = input;
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

    pub fn expires(mut self, expires: impl Into<String>) -> Self {
        self.expires = Some(expires.into());
        self
    }

    /// コピー先のサーバサイド暗号化を指定する (AES256 / aws:kms / aws:kms:dsse 等)
    pub fn server_side_encryption(mut self, input: ServerSideEncryption) -> Self {
        self.server_side_encryption = Some(input);
        self
    }

    pub fn set_server_side_encryption(mut self, input: Option<ServerSideEncryption>) -> Self {
        self.server_side_encryption = input;
        self
    }

    /// コピー先の SSE-KMS キー ID を指定する
    pub fn ssekms_key_id(mut self, key_id: impl Into<String>) -> Self {
        self.ssekms_key_id = Some(key_id.into());
        self
    }

    /// コピー先の SSE-C アルゴリズムを指定する (AES256)
    pub fn sse_customer_algorithm(mut self, algorithm: impl Into<String>) -> Self {
        self.sse_customer_algorithm = Some(algorithm.into());
        self
    }

    /// コピー先の SSE-C キーを指定する (Base64 エンコード)
    ///
    /// MD5 はキーから自動計算される。
    pub fn sse_customer_key(mut self, key: impl Into<String>) -> Self {
        self.sse_customer_key = Some(key.into());
        self
    }

    /// コピー元の SSE-C アルゴリズムを指定する (AES256)
    pub fn copy_source_sse_customer_algorithm(mut self, algorithm: impl Into<String>) -> Self {
        self.copy_source_sse_customer_algorithm = Some(algorithm.into());
        self
    }

    /// コピー元の SSE-C キーを指定する (Base64 エンコード)
    ///
    /// MD5 はキーから自動計算される。
    pub fn copy_source_sse_customer_key(mut self, key: impl Into<String>) -> Self {
        self.copy_source_sse_customer_key = Some(key.into());
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

    pub fn set_storage_class(mut self, input: Option<StorageClass>) -> Self {
        self.storage_class = input;
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

    /// オブジェクトタグを指定する (URL エンコード形式: "key1=value1&key2=value2")
    pub fn tagging(mut self, tagging: impl Into<String>) -> Self {
        self.tagging = Some(tagging.into());
        self
    }

    /// タグディレクティブを指定する (COPY または REPLACE)
    pub fn tagging_directive(mut self, input: TaggingDirective) -> Self {
        self.tagging_directive = Some(input);
        self
    }

    pub fn set_tagging_directive(mut self, input: Option<TaggingDirective>) -> Self {
        self.tagging_directive = input;
        self
    }

    /// コピー元の ETag が一致する場合のみコピーする
    pub fn copy_source_if_match(mut self, e_tag: impl Into<String>) -> Self {
        self.copy_source_if_match = Some(e_tag.into());
        self
    }

    /// コピー元の ETag が異なる場合のみコピーする
    pub fn copy_source_if_none_match(mut self, e_tag: impl Into<String>) -> Self {
        self.copy_source_if_none_match = Some(e_tag.into());
        self
    }

    /// コピー元が指定日時以降に変更されている場合のみコピーする
    pub fn copy_source_if_modified_since(mut self, date: impl Into<String>) -> Self {
        self.copy_source_if_modified_since = Some(date.into());
        self
    }

    /// コピー元が指定日時以降に変更されていない場合のみコピーする
    pub fn copy_source_if_unmodified_since(mut self, date: impl Into<String>) -> Self {
        self.copy_source_if_unmodified_since = Some(date.into());
        self
    }

    pub fn build_request(&self, now: std::time::SystemTime) -> Result<S3Request, Error> {
        let bucket = required(self.bucket.as_deref(), "bucket")?;
        let key = required(self.key.as_deref(), "key")?;
        let copy_source = required(self.copy_source.as_deref(), "copy_source")?;

        // 先頭の / を正規化して二重スラッシュを防ぐ
        let copy_source_normalized = copy_source.strip_prefix('/').unwrap_or(copy_source);
        let copy_source_header = format!("/{copy_source_normalized}");
        let mut extra_headers = vec![("x-amz-copy-source", copy_source_header.as_str())];

        if let Some(ref v) = self.tagging {
            extra_headers.push(("x-amz-tagging", v.as_str()));
        }
        if let Some(ref v) = self.tagging_directive {
            extra_headers.push(("x-amz-tagging-directive", v.as_str()));
        }
        if let Some(ref v) = self.copy_source_if_match {
            extra_headers.push(("x-amz-copy-source-if-match", v.as_str()));
        }
        if let Some(ref v) = self.copy_source_if_none_match {
            extra_headers.push(("x-amz-copy-source-if-none-match", v.as_str()));
        }
        if let Some(ref v) = self.copy_source_if_modified_since {
            extra_headers.push(("x-amz-copy-source-if-modified-since", v.as_str()));
        }
        if let Some(ref v) = self.copy_source_if_unmodified_since {
            extra_headers.push(("x-amz-copy-source-if-unmodified-since", v.as_str()));
        }
        if let Some(ref v) = self.acl {
            extra_headers.push(("x-amz-acl", v.as_str()));
        }
        if let Some(ref v) = self.storage_class {
            extra_headers.push(("x-amz-storage-class", v.as_str()));
        }
        if let Some(ref v) = self.metadata_directive {
            extra_headers.push(("x-amz-metadata-directive", v.as_str()));
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
        // コピー先の SSE-C キーが指定されている場合、MD5 を自動計算する
        let computed_key_md5;
        if let Some(ref v) = self.sse_customer_key {
            extra_headers.push(("x-amz-server-side-encryption-customer-key", v.as_str()));
            computed_key_md5 = super::compute_sse_c_key_md5(v)?;
            extra_headers.push((
                "x-amz-server-side-encryption-customer-key-md5",
                &computed_key_md5,
            ));
        }
        if let Some(ref v) = self.copy_source_sse_customer_algorithm {
            extra_headers.push((
                "x-amz-copy-source-server-side-encryption-customer-algorithm",
                v.as_str(),
            ));
        }
        // コピー元の SSE-C キーが指定されている場合、MD5 を自動計算する
        let computed_copy_source_key_md5;
        if let Some(ref v) = self.copy_source_sse_customer_key {
            extra_headers.push((
                "x-amz-copy-source-server-side-encryption-customer-key",
                v.as_str(),
            ));
            computed_copy_source_key_md5 = super::compute_sse_c_key_md5(v)?;
            extra_headers.push((
                "x-amz-copy-source-server-side-encryption-customer-key-md5",
                &computed_copy_source_key_md5,
            ));
        }

        if let Some(ref v) = self.checksum_algorithm {
            // CopyObject ではボディがないためヘッダーのみ指定する
            extra_headers.push(("x-amz-checksum-algorithm", v.as_str()));
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
            b"",
            None,
            now,
        )
    }

    pub fn parse_response(response: &super::S3Response) -> Result<CopyObjectOutput, Error> {
        if !response.is_success() {
            return Err(parse_error_response(response));
        }

        // S3 は 200 OK でもボディに <Error> を返すことがある
        check_body_error(response)?;

        let body_text = super::xml_body_text(&response.body)?;

        // <CopyObjectResult> 要素配下のフィールドを抽出する
        let last_modified = crate::xml::extract_element(body_text, "LastModified")?
            .map(|s| crate::datetime::parse_iso8601(s.as_str()))
            .transpose()?;
        let copy_object_result = Some(crate::types::CopyObjectResult {
            e_tag: crate::xml::extract_element(body_text, "ETag")?,
            last_modified,
            checksum_crc32: crate::xml::extract_element(body_text, "ChecksumCRC32")?,
            checksum_crc32_c: crate::xml::extract_element(body_text, "ChecksumCRC32C")?,
            checksum_crc64_nvme: crate::xml::extract_element(body_text, "ChecksumCRC64NVME")?,
            checksum_sha1: crate::xml::extract_element(body_text, "ChecksumSHA1")?,
            checksum_sha256: crate::xml::extract_element(body_text, "ChecksumSHA256")?,
            checksum_type: crate::xml::extract_element(body_text, "ChecksumType")?,
        });

        Ok(CopyObjectOutput {
            copy_object_result,
            version_id: response.get_header("x-amz-version-id").map(String::from),
            copy_source_version_id: response
                .get_header("x-amz-copy-source-version-id")
                .map(String::from),
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
            ssekms_encryption_context: response
                .get_header("x-amz-server-side-encryption-context")
                .map(String::from),
            bucket_key_enabled: response
                .get_header("x-amz-server-side-encryption-bucket-key-enabled")
                .and_then(|s| s.parse::<bool>().ok()),
            request_charged: response
                .get_header("x-amz-request-charged")
                .map(String::from),
        })
    }
}
