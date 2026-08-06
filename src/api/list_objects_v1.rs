//! ListObjects API (v1)
//!
//! バケット内のオブジェクト一覧を取得する。
//! Prefix や Delimiter によるフィルタリング、Marker によるページネーションに対応。
//! v2 (ListObjectsV2) と異なり `list-type` クエリパラメータを送らない。
//!
//! <https://docs.aws.amazon.com/AmazonS3/latest/API/API_ListObjects.html>

use crate::client::Client;
use crate::error::Error;
use crate::types::{EncodingType, ListObjectsOutput};

use super::{
    S3Request, build_signed_request, extract_xml_common_prefixes, extract_xml_objects,
    parse_error_response, required,
};

pub struct ListObjectsFluentBuilder<'a> {
    client: &'a Client,
    bucket: Option<String>,
    prefix: Option<String>,
    delimiter: Option<String>,
    max_keys: Option<i32>,
    marker: Option<String>,
    encoding_type: Option<EncodingType>,
}

impl<'a> ListObjectsFluentBuilder<'a> {
    pub(crate) fn new(client: &'a Client) -> Self {
        Self {
            client,
            bucket: None,
            prefix: None,
            delimiter: None,
            max_keys: None,
            marker: None,
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

    /// ページネーションの開始位置を指定する
    ///
    /// delimiter 指定時は前回のレスポンスの `next_marker` を渡す。
    /// delimiter 未指定で切り詰められた場合は、前回のレスポンスの最後の
    /// `Key` 要素を渡す (NextMarker は delimiter 指定時のみ返る)。
    ///
    /// `encoding_type` を指定した場合は、レスポンスの `Key` / `Marker` /
    /// `NextMarker` が URL エンコードされた値で返る。ページネーションでは
    /// エンコードされた値をそのまま marker に渡すか、デコードしてから渡すかで
    /// サーバー実装により挙動が異なるため、対象サーバーの仕様を確認すること。
    ///
    /// 仕様: https://docs.aws.amazon.com/AmazonS3/latest/API/API_ListObjects.html
    /// > This element is returned only if you have the delimiter request parameter specified.
    /// > If the response does not include the NextMarker element and it is truncated,
    /// > you can use the value of the last Key element in the response as the marker parameter.
    pub fn marker(mut self, marker: impl Into<String>) -> Self {
        self.marker = Some(marker.into());
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

        let mut query_params: Vec<(String, String)> = Vec::new();
        if let Some(ref prefix) = self.prefix {
            query_params.push(("prefix".into(), prefix.clone()));
        }
        if let Some(ref delimiter) = self.delimiter {
            query_params.push(("delimiter".into(), delimiter.clone()));
        }
        if let Some(max_keys) = self.max_keys {
            query_params.push(("max-keys".into(), max_keys.to_string()));
        }
        if let Some(ref marker) = self.marker {
            query_params.push(("marker".into(), marker.clone()));
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

    pub fn parse_response(response: &super::S3Response) -> Result<ListObjectsOutput, Error> {
        if !response.is_success() {
            return Err(parse_error_response(response));
        }

        let body_text = super::xml_body_text(&response.body)?;

        let contents = extract_xml_objects(body_text)?;
        let common_prefixes = extract_xml_common_prefixes(body_text)?;

        Ok(ListObjectsOutput {
            is_truncated: crate::xml::extract_element(body_text, "IsTruncated")?
                .map(|v| crate::xml::parse_xml_bool(&v))
                .transpose()?,
            // Marker / NextMarker は空要素で返ると Some("") になるため None に正規化する。
            // 空の marker をページネーションに渡すと無限ループの原因になる。
            // (prefix / name 等の他の文字列フィールドは空要素が意味を持つため正規化しない)
            marker: crate::xml::extract_element(body_text, "Marker")?.filter(|v| !v.is_empty()),
            next_marker: crate::xml::extract_element(body_text, "NextMarker")?
                .filter(|v| !v.is_empty()),
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
            encoding_type: crate::xml::extract_element(body_text, "EncodingType")?
                .map(|v| EncodingType::from(v.as_str())),
            request_charged: response
                .get_header("x-amz-request-charged")
                .map(String::from),
        })
    }
}
