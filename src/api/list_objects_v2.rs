//! ListObjectsV2 API
//!
//! バケット内のオブジェクト一覧を取得する。
//! Prefix や Delimiter によるフィルタリング、ContinuationToken によるページネーションに対応。
//!
//! <https://docs.aws.amazon.com/AmazonS3/latest/API/API_ListObjectsV2.html>

use crate::client::Client;
use crate::error::Error;
use crate::types::{CommonPrefix, EncodingType, ListObjectsV2Output, Object};

use super::{S3Request, build_signed_request, parse_error_response, required};

pub struct ListObjectsV2FluentBuilder<'a> {
    client: &'a Client,
    bucket: Option<String>,
    prefix: Option<String>,
    delimiter: Option<String>,
    max_keys: Option<i32>,
    continuation_token: Option<String>,
    start_after: Option<String>,
    encoding_type: Option<EncodingType>,
}

impl<'a> ListObjectsV2FluentBuilder<'a> {
    pub(crate) fn new(client: &'a Client) -> Self {
        Self {
            client,
            bucket: None,
            prefix: None,
            delimiter: None,
            max_keys: None,
            continuation_token: None,
            start_after: None,
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

    pub fn max_keys(mut self, max_keys: i32) -> Self {
        self.max_keys = Some(max_keys);
        self
    }

    pub fn continuation_token(mut self, continuation_token: impl Into<String>) -> Self {
        self.continuation_token = Some(continuation_token.into());
        self
    }

    pub fn start_after(mut self, start_after: impl Into<String>) -> Self {
        self.start_after = Some(start_after.into());
        self
    }

    /// エンコーディングタイプを指定する ("url")
    pub fn encoding_type(mut self, input: EncodingType) -> Self {
        self.encoding_type = Some(input);
        self
    }

    pub fn set_encoding_type(mut self, input: Option<EncodingType>) -> Self {
        self.encoding_type = input;
        self
    }

    pub fn build_request(&self, now: std::time::SystemTime) -> Result<S3Request, Error> {
        let bucket = required(self.bucket.as_deref(), "bucket")?;

        let mut query_params: Vec<(String, String)> = vec![("list-type".into(), "2".into())];
        if let Some(ref prefix) = self.prefix {
            query_params.push(("prefix".into(), prefix.clone()));
        }
        if let Some(ref delimiter) = self.delimiter {
            query_params.push(("delimiter".into(), delimiter.clone()));
        }
        if let Some(max_keys) = self.max_keys {
            query_params.push(("max-keys".into(), max_keys.to_string()));
        }
        if let Some(ref token) = self.continuation_token {
            query_params.push(("continuation-token".into(), token.clone()));
        }
        if let Some(ref start_after) = self.start_after {
            query_params.push(("start-after".into(), start_after.clone()));
        }
        if let Some(ref encoding_type) = self.encoding_type {
            query_params.push(("encoding-type".into(), encoding_type.as_str().to_string()));
        }

        let query_refs: Vec<(&str, &str)> = query_params
            .iter()
            .map(|(k, v)| (k.as_str(), v.as_str()))
            .collect();

        build_signed_request(
            &self.client.config_ref(),
            "GET",
            bucket,
            "",
            &[],
            b"",
            Some(&query_refs),
            now,
        )
    }

    pub fn parse_response(response: &super::S3Response) -> Result<ListObjectsV2Output, Error> {
        if !response.is_success() {
            return Err(parse_error_response(response));
        }

        let body_text = super::xml_body_text(&response.body)?;

        let contents = extract_xml_objects(body_text)?;
        let common_prefixes = extract_xml_common_prefixes(body_text)?;

        Ok(ListObjectsV2Output {
            is_truncated: crate::xml::extract_element(body_text, "IsTruncated")?
                .and_then(|v| v.parse::<bool>().ok()),
            contents: if contents.is_empty() {
                None
            } else {
                Some(contents)
            },
            name: crate::xml::extract_element(body_text, "Name")?,
            prefix: crate::xml::extract_element(body_text, "Prefix")?,
            delimiter: crate::xml::extract_element(body_text, "Delimiter")?,
            max_keys: crate::xml::extract_element(body_text, "MaxKeys")?
                .and_then(|v| v.parse::<i32>().ok()),
            common_prefixes: if common_prefixes.is_empty() {
                None
            } else {
                Some(common_prefixes)
            },
            key_count: crate::xml::extract_element(body_text, "KeyCount")?
                .and_then(|v| v.parse::<i32>().ok()),
            continuation_token: crate::xml::extract_element(body_text, "ContinuationToken")?,
            next_continuation_token: crate::xml::extract_element(
                body_text,
                "NextContinuationToken",
            )?,
            start_after: crate::xml::extract_element(body_text, "StartAfter")?,
        })
    }
}

fn extract_xml_objects(text: &str) -> Result<Vec<Object>, Error> {
    let mut objects = Vec::new();
    crate::xml::for_each_element(text, "Contents", |elem| {
        let owner = if elem.has("Owner") {
            Some(crate::types::Owner {
                display_name: elem.get_nested(&["Owner", "DisplayName"]).map(String::from),
                id: elem.get_nested(&["Owner", "ID"]).map(String::from),
            })
        } else {
            None
        };
        let restore_status = if elem.has("RestoreStatus") {
            Some(crate::types::RestoreStatus {
                is_restore_in_progress: elem
                    .get_nested(&["RestoreStatus", "IsRestoreInProgress"])
                    .and_then(|s| s.parse::<bool>().ok()),
                restore_expiry_date: elem
                    .get_nested(&["RestoreStatus", "RestoreExpiryDate"])
                    .and_then(|s| crate::datetime::parse_iso8601(s).ok()),
            })
        } else {
            None
        };
        let checksum_algorithm = {
            let all = elem.get_all("ChecksumAlgorithm");
            if all.is_empty() {
                None
            } else {
                Some(
                    all.into_iter()
                        .map(crate::types::ChecksumAlgorithm::from)
                        .collect(),
                )
            }
        };
        objects.push(Object {
            key: elem.get("Key").map(String::from),
            last_modified: elem
                .get("LastModified")
                .and_then(|s| crate::datetime::parse_iso8601(s).ok()),
            e_tag: elem.get("ETag").map(String::from),
            size: elem.get_parsed::<i64>("Size"),
            storage_class: elem
                .get("StorageClass")
                .map(crate::types::StorageClass::from),
            owner,
            restore_status,
            checksum_algorithm,
            checksum_type: elem.get("ChecksumType").map(String::from),
        });
    })?;
    Ok(objects)
}

fn extract_xml_common_prefixes(text: &str) -> Result<Vec<CommonPrefix>, Error> {
    let mut prefixes = Vec::new();
    crate::xml::for_each_element(text, "CommonPrefixes", |elem| {
        prefixes.push(CommonPrefix {
            prefix: elem.get("Prefix").map(String::from),
        });
    })?;
    Ok(prefixes)
}
