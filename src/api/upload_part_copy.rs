//! UploadPartCopy API
//!
//! コピー元���ブジェクトの範囲をパートデータ��してアップロードする。
//!
//! <https://docs.aws.amazon.com/AmazonS3/latest/API/API_UploadPartCopy.html>

use crate::client::Client;
use crate::error::Error;
use crate::types::UploadPartCopyOutput;

use super::{S3Request, build_signed_request, parse_error_response, required};

pub struct UploadPartCopyFluentBuilder<'a> {
    client: &'a Client,
    bucket: Option<String>,
    key: Option<String>,
    upload_id: Option<String>,
    part_number: Option<i32>,
    copy_source: Option<String>,
    copy_source_range: Option<String>,
    // 条件付きコピー
    copy_source_if_match: Option<String>,
    copy_source_if_none_match: Option<String>,
    copy_source_if_modified_since: Option<String>,
    copy_source_if_unmodified_since: Option<String>,
    // コピー元 SSE-C
    copy_source_sse_customer_algorithm: Option<String>,
    copy_source_sse_customer_key: Option<String>,
    // コピー先 SSE-C
    sse_customer_algorithm: Option<String>,
    sse_customer_key: Option<String>,
}

impl<'a> UploadPartCopyFluentBuilder<'a> {
    pub(crate) fn new(client: &'a Client) -> Self {
        Self {
            client,
            bucket: None,
            key: None,
            upload_id: None,
            part_number: None,
            copy_source: None,
            copy_source_range: None,
            copy_source_if_match: None,
            copy_source_if_none_match: None,
            copy_source_if_modified_since: None,
            copy_source_if_unmodified_since: None,
            copy_source_sse_customer_algorithm: None,
            copy_source_sse_customer_key: None,
            sse_customer_algorithm: None,
            sse_customer_key: None,
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

    /// コピー元を "bucket/key" 形式で指定する
    pub fn copy_source(mut self, copy_source: impl Into<String>) -> Self {
        self.copy_source = Some(copy_source.into());
        self
    }

    /// コピー元のバイト範囲を指定する (例: "bytes=0-999")
    pub fn copy_source_range(mut self, range: impl Into<String>) -> Self {
        self.copy_source_range = Some(range.into());
        self
    }

    /// コピー元の ETag が一致する場合のみコピーする
    pub fn copy_source_if_match(mut self, e_tag: impl Into<String>) -> Self {
        self.copy_source_if_match = Some(e_tag.into());
        self
    }

    /// コピー元の ETag が異なる場合のみコピー���る
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

    /// コピー元の SSE-C アル���リズムを指定する (AES256)
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

    /// コピー先の SSE-C アルゴリズムを指定する (AES256)
    pub fn sse_customer_algorithm(mut self, algorithm: impl Into<String>) -> Self {
        self.sse_customer_algorithm = Some(algorithm.into());
        self
    }

    /// コピー先の SSE-C キーを指定���る (Base64 エンコード)
    ///
    /// MD5 はキーから自���計算される。
    pub fn sse_customer_key(mut self, key: impl Into<String>) -> Self {
        self.sse_customer_key = Some(key.into());
        self
    }

    pub fn build_request(&self, now: std::time::SystemTime) -> Result<S3Request, Error> {
        let bucket = required(self.bucket.as_deref(), "bucket")?;
        let key = required(self.key.as_deref(), "key")?;
        let upload_id = required(self.upload_id.as_deref(), "upload_id")?;
        let part_number = self
            .part_number
            .ok_or_else(|| Error::InvalidInput("part_number is required".into()))?;
        let copy_source = required(self.copy_source.as_deref(), "copy_source")?;

        if !(1..=10000).contains(&part_number) {
            return Err(Error::InvalidInput(
                "part_number must be between 1 and 10000".to_string(),
            ));
        }

        // 先頭の / を正規化して二重スラッシュを防ぐ
        let copy_source_normalized = copy_source.strip_prefix('/').unwrap_or(copy_source);
        let copy_source_header = format!("/{copy_source_normalized}");
        let mut extra_headers = vec![("x-amz-copy-source", copy_source_header.as_str())];

        if let Some(ref v) = self.copy_source_range {
            extra_headers.push(("x-amz-copy-source-range", v.as_str()));
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

        // コピー元の SSE-C
        if let Some(ref v) = self.copy_source_sse_customer_algorithm {
            extra_headers.push((
                "x-amz-copy-source-server-side-encryption-customer-algorithm",
                v.as_str(),
            ));
        }
        let computed_src_key_md5;
        if let Some(ref v) = self.copy_source_sse_customer_key {
            extra_headers.push((
                "x-amz-copy-source-server-side-encryption-customer-key",
                v.as_str(),
            ));
            computed_src_key_md5 = super::compute_sse_c_key_md5(v)?;
            extra_headers.push((
                "x-amz-copy-source-server-side-encryption-customer-key-md5",
                &computed_src_key_md5,
            ));
        }

        // コピー先の SSE-C
        if let Some(ref v) = self.sse_customer_algorithm {
            extra_headers.push((
                "x-amz-server-side-encryption-customer-algorithm",
                v.as_str(),
            ));
        }
        let computed_dst_key_md5;
        if let Some(ref v) = self.sse_customer_key {
            extra_headers.push(("x-amz-server-side-encryption-customer-key", v.as_str()));
            computed_dst_key_md5 = super::compute_sse_c_key_md5(v)?;
            extra_headers.push((
                "x-amz-server-side-encryption-customer-key-md5",
                &computed_dst_key_md5,
            ));
        }

        let part_number_str = part_number.to_string();
        let query_params = [
            ("partNumber", part_number_str.as_str()),
            ("uploadId", upload_id),
        ];

        build_signed_request(
            &self.client.config_ref(),
            "PUT",
            bucket,
            key,
            &extra_headers,
            b"",
            Some(&query_params),
            now,
        )
    }

    pub fn parse_response(response: &super::S3Response) -> Result<UploadPartCopyOutput, Error> {
        if !response.is_success() {
            return Err(parse_error_response(response));
        }

        // UploadPartCopy も 200 OK でボディにエラーを返すことがある
        super::check_body_error(response)?;

        let body_text = super::xml_body_text(&response.body)?;

        Ok(UploadPartCopyOutput {
            e_tag: crate::xml::extract_element(body_text, "ETag")?,
            last_modified: crate::xml::extract_element(body_text, "LastModified")?
                .map(|s| crate::datetime::parse_iso8601(s.as_str()))
                .transpose()?,
            copy_source_version_id: response
                .get_header("x-amz-copy-source-version-id")
                .map(String::from),
        })
    }
}
