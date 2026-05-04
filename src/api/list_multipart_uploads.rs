//! ListMultipartUploads API
//!
//! バケット内の未完了マルチパートアップロード一覧を取得する。
//! Prefix や Delimiter によるフィルタリング、ページネーションに対応。
//!
//! <https://docs.aws.amazon.com/AmazonS3/latest/API/API_ListMultipartUploads.html>

use crate::client::Client;
use crate::error::Error;
use crate::types::{CommonPrefix, ListMultipartUploadsOutput, MultipartUpload};

use super::{S3Request, build_signed_request, parse_error_response, required};

pub struct ListMultipartUploadsFluentBuilder<'a> {
    client: &'a Client,
    bucket: Option<String>,
    prefix: Option<String>,
    delimiter: Option<String>,
    max_uploads: Option<i32>,
    key_marker: Option<String>,
    upload_id_marker: Option<String>,
    encoding_type: Option<String>,
}

impl<'a> ListMultipartUploadsFluentBuilder<'a> {
    pub(crate) fn new(client: &'a Client) -> Self {
        Self {
            client,
            bucket: None,
            prefix: None,
            delimiter: None,
            max_uploads: None,
            key_marker: None,
            upload_id_marker: None,
            encoding_type: None,
        }
    }

    pub fn bucket(mut self, bucket: impl Into<String>) -> Self {
        self.bucket = Some(bucket.into());
        self
    }

    pub fn prefix(mut self, prefix: impl Into<String>) -> Self {
        self.prefix = Some(prefix.into());
        self
    }

    pub fn delimiter(mut self, delimiter: impl Into<String>) -> Self {
        self.delimiter = Some(delimiter.into());
        self
    }

    pub fn max_uploads(mut self, max_uploads: i32) -> Self {
        self.max_uploads = Some(max_uploads);
        self
    }

    pub fn key_marker(mut self, key_marker: impl Into<String>) -> Self {
        self.key_marker = Some(key_marker.into());
        self
    }

    pub fn upload_id_marker(mut self, upload_id_marker: impl Into<String>) -> Self {
        self.upload_id_marker = Some(upload_id_marker.into());
        self
    }

    /// エンコーディングタイプを指定する ("url")
    pub fn encoding_type(mut self, encoding_type: impl Into<String>) -> Self {
        self.encoding_type = Some(encoding_type.into());
        self
    }

    pub fn build_request(&self) -> Result<S3Request, Error> {
        let bucket = required(self.bucket.as_deref(), "bucket")?;

        let mut query_params: Vec<(String, String)> = vec![("uploads".into(), "".into())];

        if let Some(ref prefix) = self.prefix {
            query_params.push(("prefix".into(), prefix.clone()));
        }
        if let Some(ref delimiter) = self.delimiter {
            query_params.push(("delimiter".into(), delimiter.clone()));
        }
        if let Some(max_uploads) = self.max_uploads {
            query_params.push(("max-uploads".into(), max_uploads.to_string()));
        }
        if let Some(ref marker) = self.key_marker {
            query_params.push(("key-marker".into(), marker.clone()));
        }
        if let Some(ref marker) = self.upload_id_marker {
            query_params.push(("upload-id-marker".into(), marker.clone()));
        }
        if let Some(ref encoding_type) = self.encoding_type {
            query_params.push(("encoding-type".into(), encoding_type.clone()));
        }

        let query_refs: Vec<(&str, &str)> = query_params
            .iter()
            .map(|(k, v)| (k.as_str(), v.as_str()))
            .collect();

        Ok(build_signed_request(
            &self.client.config_ref(),
            "GET",
            bucket,
            "",
            &[],
            b"",
            Some(&query_refs),
        ))
    }

    pub fn parse_response(
        response: &super::S3Response,
    ) -> Result<ListMultipartUploadsOutput, Error> {
        if !response.is_success() {
            return Err(parse_error_response(response));
        }

        let body_text = super::xml_body_text(&response.body)?;

        let uploads = extract_xml_uploads(body_text);
        let common_prefixes = extract_xml_common_prefixes(body_text);

        Ok(ListMultipartUploadsOutput {
            bucket: crate::xml::extract_element(body_text, "Bucket"),
            key_marker: crate::xml::extract_element(body_text, "KeyMarker"),
            upload_id_marker: crate::xml::extract_element(body_text, "UploadIdMarker"),
            next_key_marker: crate::xml::extract_element(body_text, "NextKeyMarker"),
            next_upload_id_marker: crate::xml::extract_element(body_text, "NextUploadIdMarker"),
            prefix: crate::xml::extract_element(body_text, "Prefix"),
            delimiter: crate::xml::extract_element(body_text, "Delimiter"),
            max_uploads: crate::xml::extract_element(body_text, "MaxUploads")
                .and_then(|v| v.parse::<i32>().ok()),
            is_truncated: crate::xml::extract_element(body_text, "IsTruncated")
                .and_then(|v| v.parse::<bool>().ok()),
            uploads: if uploads.is_empty() {
                None
            } else {
                Some(uploads)
            },
            common_prefixes: if common_prefixes.is_empty() {
                None
            } else {
                Some(common_prefixes)
            },
        })
    }
}

fn extract_xml_uploads(text: &str) -> Vec<MultipartUpload> {
    let mut uploads = Vec::new();
    crate::xml::for_each_element(text, "Upload", |elem| {
        uploads.push(MultipartUpload {
            upload_id: elem.get("UploadId").map(String::from),
            key: elem.get("Key").map(String::from),
            initiated: elem.get("Initiated").map(String::from),
            storage_class: elem.get("StorageClass").map(String::from),
        });
    });
    uploads
}

fn extract_xml_common_prefixes(text: &str) -> Vec<CommonPrefix> {
    let mut prefixes = Vec::new();
    crate::xml::for_each_element(text, "CommonPrefixes", |elem| {
        prefixes.push(CommonPrefix {
            prefix: elem.get("Prefix").map(String::from),
        });
    });
    prefixes
}
