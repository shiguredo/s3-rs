//! ListObjectVersions API
//!
//! バージョニング有効バケット内のオブジェクトバージョンおよび削除マーカーを一覧する。
//!
//! <https://docs.aws.amazon.com/AmazonS3/latest/API/API_ListObjectVersions.html>

use crate::client::Client;
use crate::error::Error;
use crate::types::{CommonPrefix, DeleteMarkerEntry, ListObjectVersionsOutput, ObjectVersion};

use super::{S3Request, build_signed_request, parse_error_response, required};

pub struct ListObjectVersionsFluentBuilder<'a> {
    client: &'a Client,
    bucket: Option<String>,
    prefix: Option<String>,
    delimiter: Option<String>,
    key_marker: Option<String>,
    version_id_marker: Option<String>,
    max_keys: Option<i32>,
    encoding_type: Option<String>,
}

impl<'a> ListObjectVersionsFluentBuilder<'a> {
    pub(crate) fn new(client: &'a Client) -> Self {
        Self {
            client,
            bucket: None,
            prefix: None,
            delimiter: None,
            key_marker: None,
            version_id_marker: None,
            max_keys: None,
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

    pub fn key_marker(mut self, key_marker: impl Into<String>) -> Self {
        self.key_marker = Some(key_marker.into());
        self
    }

    pub fn version_id_marker(mut self, version_id_marker: impl Into<String>) -> Self {
        self.version_id_marker = Some(version_id_marker.into());
        self
    }

    pub fn max_keys(mut self, max_keys: i32) -> Self {
        self.max_keys = Some(max_keys);
        self
    }

    /// エンコーディングタイプを指定する ("url")
    pub fn encoding_type(mut self, encoding_type: impl Into<String>) -> Self {
        self.encoding_type = Some(encoding_type.into());
        self
    }

    pub fn build_request(&self) -> Result<S3Request, Error> {
        let bucket = required(self.bucket.as_deref(), "bucket")?;

        let mut query_params: Vec<(String, String)> = vec![("versions".into(), "".into())];
        if let Some(ref prefix) = self.prefix {
            query_params.push(("prefix".into(), prefix.clone()));
        }
        if let Some(ref delimiter) = self.delimiter {
            query_params.push(("delimiter".into(), delimiter.clone()));
        }
        if let Some(ref key_marker) = self.key_marker {
            query_params.push(("key-marker".into(), key_marker.clone()));
        }
        if let Some(ref version_id_marker) = self.version_id_marker {
            query_params.push(("version-id-marker".into(), version_id_marker.clone()));
        }
        if let Some(max_keys) = self.max_keys {
            query_params.push(("max-keys".into(), max_keys.to_string()));
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

    pub fn parse_response(response: &super::S3Response) -> Result<ListObjectVersionsOutput, Error> {
        if !response.is_success() {
            return Err(parse_error_response(response));
        }

        let body_text = super::xml_body_text(&response.body)?;

        let versions = extract_xml_versions(body_text);
        let delete_markers = extract_xml_delete_markers(body_text);
        let common_prefixes = extract_xml_common_prefixes(body_text);

        Ok(ListObjectVersionsOutput {
            is_truncated: crate::xml::extract_element(body_text, "IsTruncated")
                .and_then(|v| v.parse::<bool>().ok()),
            next_key_marker: crate::xml::extract_element(body_text, "NextKeyMarker"),
            next_version_id_marker: crate::xml::extract_element(body_text, "NextVersionIdMarker"),
            versions: if versions.is_empty() {
                None
            } else {
                Some(versions)
            },
            delete_markers: if delete_markers.is_empty() {
                None
            } else {
                Some(delete_markers)
            },
            common_prefixes: if common_prefixes.is_empty() {
                None
            } else {
                Some(common_prefixes)
            },
            name: crate::xml::extract_element(body_text, "Name"),
            prefix: crate::xml::extract_element(body_text, "Prefix"),
            delimiter: crate::xml::extract_element(body_text, "Delimiter"),
            max_keys: crate::xml::extract_element(body_text, "MaxKeys")
                .and_then(|v| v.parse::<i32>().ok()),
            key_marker: crate::xml::extract_element(body_text, "KeyMarker"),
            version_id_marker: crate::xml::extract_element(body_text, "VersionIdMarker"),
            encoding_type: crate::xml::extract_element(body_text, "EncodingType"),
        })
    }
}

fn extract_xml_versions(text: &str) -> Vec<ObjectVersion> {
    let mut versions = Vec::new();
    crate::xml::for_each_element(text, "Version", |elem| {
        versions.push(ObjectVersion {
            key: elem.get("Key").map(String::from),
            version_id: elem.get("VersionId").map(String::from),
            is_latest: elem.get_parsed::<bool>("IsLatest"),
            last_modified: elem.get("LastModified").map(String::from),
            e_tag: elem.get("ETag").map(String::from),
            size: elem.get_parsed::<i64>("Size"),
            storage_class: elem.get("StorageClass").map(String::from),
        });
    });
    versions
}

fn extract_xml_delete_markers(text: &str) -> Vec<DeleteMarkerEntry> {
    let mut markers = Vec::new();
    crate::xml::for_each_element(text, "DeleteMarker", |elem| {
        markers.push(DeleteMarkerEntry {
            key: elem.get("Key").map(String::from),
            version_id: elem.get("VersionId").map(String::from),
            is_latest: elem.get_parsed::<bool>("IsLatest"),
            last_modified: elem.get("LastModified").map(String::from),
        });
    });
    markers
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
