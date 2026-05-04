//! ListParts API
//!
//! マルチパートアップロードのアップロード済みパート一覧を取得する。
//! UploadId の指定が必要。
//!
//! <https://docs.aws.amazon.com/AmazonS3/latest/API/API_ListParts.html>

use crate::client::Client;
use crate::error::Error;
use crate::types::{ListPartsOutput, Part};

use super::{S3Request, build_signed_request, parse_error_response, required};

pub struct ListPartsFluentBuilder<'a> {
    client: &'a Client,
    bucket: Option<String>,
    key: Option<String>,
    upload_id: Option<String>,
    max_parts: Option<i32>,
    part_number_marker: Option<i32>,
    /// SSE-C アルゴリズム (AES256)
    sse_customer_algorithm: Option<String>,
    /// SSE-C キー (Base64)
    sse_customer_key: Option<String>,
}

impl<'a> ListPartsFluentBuilder<'a> {
    pub(crate) fn new(client: &'a Client) -> Self {
        Self {
            client,
            bucket: None,
            key: None,
            upload_id: None,
            max_parts: None,
            part_number_marker: None,
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

    pub fn max_parts(mut self, max_parts: i32) -> Self {
        self.max_parts = Some(max_parts);
        self
    }

    pub fn part_number_marker(mut self, part_number_marker: i32) -> Self {
        self.part_number_marker = Some(part_number_marker);
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

    pub fn build_request(&self) -> Result<S3Request, Error> {
        let bucket = required(self.bucket.as_deref(), "bucket")?;
        let key = required(self.key.as_deref(), "key")?;
        let upload_id = required(self.upload_id.as_deref(), "upload_id")?;

        let mut query_params: Vec<(String, String)> =
            vec![("uploadId".into(), upload_id.to_string())];

        if let Some(max_parts) = self.max_parts {
            query_params.push(("max-parts".into(), max_parts.to_string()));
        }
        if let Some(marker) = self.part_number_marker {
            query_params.push(("part-number-marker".into(), marker.to_string()));
        }

        let query_refs: Vec<(&str, &str)> = query_params
            .iter()
            .map(|(k, v)| (k.as_str(), v.as_str()))
            .collect();

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

        Ok(build_signed_request(
            &self.client.config_ref(),
            "GET",
            bucket,
            key,
            &extra_headers,
            b"",
            Some(&query_refs),
        ))
    }

    pub fn parse_response(response: &super::S3Response) -> Result<ListPartsOutput, Error> {
        if !response.is_success() {
            return Err(parse_error_response(response));
        }

        let body_text = super::xml_body_text(&response.body)?;

        let parts = extract_xml_parts(body_text);

        Ok(ListPartsOutput {
            bucket: crate::xml::extract_element(body_text, "Bucket"),
            key: crate::xml::extract_element(body_text, "Key"),
            upload_id: crate::xml::extract_element(body_text, "UploadId"),
            part_number_marker: crate::xml::extract_element(body_text, "PartNumberMarker")
                .and_then(|v| v.parse::<i32>().ok()),
            next_part_number_marker: crate::xml::extract_element(body_text, "NextPartNumberMarker")
                .and_then(|v| v.parse::<i32>().ok()),
            max_parts: crate::xml::extract_element(body_text, "MaxParts")
                .and_then(|v| v.parse::<i32>().ok()),
            is_truncated: crate::xml::extract_element(body_text, "IsTruncated")
                .and_then(|v| v.parse::<bool>().ok()),
            parts: if parts.is_empty() { None } else { Some(parts) },
            storage_class: crate::xml::extract_element(body_text, "StorageClass"),
        })
    }
}

fn extract_xml_parts(text: &str) -> Vec<Part> {
    let mut parts = Vec::new();
    crate::xml::for_each_element(text, "Part", |elem| {
        parts.push(Part {
            part_number: elem.get_parsed::<i32>("PartNumber"),
            last_modified: elem.get("LastModified").map(String::from),
            e_tag: elem.get("ETag").map(String::from),
            size: elem.get_parsed::<i64>("Size"),
        });
    });
    parts
}
