//! GetBucketLifecycleConfiguration API
//!
//! バケットのライフサイクル設定を取得する。
//!
//! <https://docs.aws.amazon.com/AmazonS3/latest/API/API_GetBucketLifecycleConfiguration.html>

use crate::client::S3Client;
use crate::error::Error;
use crate::types::{
    AbortIncompleteMultipartUpload, ExpirationStatus, GetBucketLifecycleConfigurationOutput,
    LifecycleExpiration, LifecycleRule, LifecycleRuleAndOperator, LifecycleRuleFilter,
    NoncurrentVersionExpiration, NoncurrentVersionTransition, Tag, Transition,
};

use super::{S3Request, build_signed_request, parse_error_response, required};

pub struct GetBucketLifecycleConfigurationFluentBuilder<'a> {
    client: &'a S3Client,
    bucket: Option<String>,
}

impl<'a> GetBucketLifecycleConfigurationFluentBuilder<'a> {
    pub(crate) fn new(client: &'a S3Client) -> Self {
        Self {
            client,
            bucket: None,
        }
    }

    pub fn bucket(mut self, bucket: impl Into<String>) -> Self {
        self.bucket = Some(bucket.into());
        self
    }

    pub fn build_request(&self) -> Result<S3Request, Error> {
        let bucket = required(self.bucket.as_deref(), "bucket")?;
        Ok(build_signed_request(
            &self.client.config_ref(),
            "GET",
            bucket,
            "",
            &[],
            b"",
            Some(&[("lifecycle", "")]),
        ))
    }

    pub fn parse_response(
        response: &super::S3Response,
    ) -> Result<GetBucketLifecycleConfigurationOutput, Error> {
        if !response.is_success() {
            return Err(parse_error_response(response));
        }

        let body_text = std::str::from_utf8(&response.body)
            .map_err(|_| Error::InvalidResponse("non-UTF-8 response body".to_string()))?;

        Ok(GetBucketLifecycleConfigurationOutput {
            rules: extract_lifecycle_rules(body_text),
        })
    }
}

/// LifecycleConfiguration XML からルールを抽出する
///
/// S3 のレスポンスは以下の構造:
/// ```xml
/// <LifecycleConfiguration>
///   <Rule>
///     <ID>rule-id</ID>
///     <Status>Enabled</Status>
///     <Filter><Prefix>documents/</Prefix></Filter>
///     <Expiration><Days>365</Days></Expiration>
///     ...
///   </Rule>
/// </LifecycleConfiguration>
/// ```
///
/// `for_each_element` は直接の子要素のみを取得するため、
/// ネストされた要素 (Filter, Expiration 等) は個別にパースする必要がある。
/// Rule 要素全体の XML を切り出してサブパースを行う。
fn extract_lifecycle_rules(xml_str: &str) -> Vec<LifecycleRule> {
    let mut rules = Vec::new();

    // Rule 要素の位置を特定して個別にパースする
    let mut search_from = 0;
    while let Some(start) = xml_str[search_from..].find("<Rule>") {
        let abs_start = search_from + start;
        if let Some(end_offset) = xml_str[abs_start..].find("</Rule>") {
            let rule_xml = &xml_str[abs_start..abs_start + end_offset + "</Rule>".len()];
            if let Some(rule) = parse_rule(rule_xml) {
                rules.push(rule);
            }
            search_from = abs_start + end_offset + "</Rule>".len();
        } else {
            break;
        }
    }

    rules
}

fn parse_rule(rule_xml: &str) -> Option<LifecycleRule> {
    let id = crate::xml::extract_element(rule_xml, "ID");
    let status_str = crate::xml::extract_element(rule_xml, "Status")?;
    let status: ExpirationStatus = status_str.parse().ok()?;

    let filter = parse_filter(rule_xml);
    let expiration = parse_expiration(rule_xml);
    let transitions = parse_transitions(rule_xml);
    let nv_transitions = parse_noncurrent_version_transitions(rule_xml);
    let nv_expiration = parse_noncurrent_version_expiration(rule_xml);
    let abort = parse_abort_incomplete_multipart_upload(rule_xml);

    Some(LifecycleRule {
        id,
        status,
        filter,
        expiration,
        transitions: if transitions.is_empty() {
            None
        } else {
            Some(transitions)
        },
        noncurrent_version_transitions: if nv_transitions.is_empty() {
            None
        } else {
            Some(nv_transitions)
        },
        noncurrent_version_expiration: nv_expiration,
        abort_incomplete_multipart_upload: abort,
    })
}

fn parse_filter(rule_xml: &str) -> Option<LifecycleRuleFilter> {
    let filter_start = rule_xml.find("<Filter>")?;
    let filter_end = rule_xml.find("</Filter>")?;
    let filter_xml = &rule_xml[filter_start..filter_end + "</Filter>".len()];

    let mut filter = LifecycleRuleFilter::default();

    // And 要素があるかチェックする
    if let Some(and_start) = filter_xml.find("<And>")
        && let Some(and_end) = filter_xml.find("</And>")
    {
        let and_xml = &filter_xml[and_start..and_end + "</And>".len()];
        filter.and = Some(parse_and_operator(and_xml));
        return Some(filter);
    }

    filter.prefix = crate::xml::extract_element(filter_xml, "Prefix");

    // Tag 要素のパース
    if filter_xml.contains("<Tag>") {
        let tag_key = crate::xml::extract_element(filter_xml, "Key");
        let tag_value = crate::xml::extract_element(filter_xml, "Value");
        if let (Some(key), Some(value)) = (tag_key, tag_value) {
            filter.tag = Some(Tag { key, value });
        }
    }

    filter.object_size_greater_than =
        crate::xml::extract_element(filter_xml, "ObjectSizeGreaterThan")
            .and_then(|v| v.parse().ok());
    filter.object_size_less_than =
        crate::xml::extract_element(filter_xml, "ObjectSizeLessThan").and_then(|v| v.parse().ok());

    Some(filter)
}

fn parse_and_operator(and_xml: &str) -> LifecycleRuleAndOperator {
    let mut and = LifecycleRuleAndOperator {
        prefix: crate::xml::extract_element(and_xml, "Prefix"),
        ..Default::default()
    };

    // タグのパース
    let mut tags = Vec::new();
    crate::xml::for_each_element(and_xml, "Tag", |elem| {
        if let (Some(key), Some(value)) = (elem.get("Key"), elem.get("Value")) {
            tags.push(Tag {
                key: key.to_string(),
                value: value.to_string(),
            });
        }
    });
    if !tags.is_empty() {
        and.tags = Some(tags);
    }

    and.object_size_greater_than =
        crate::xml::extract_element(and_xml, "ObjectSizeGreaterThan").and_then(|v| v.parse().ok());
    and.object_size_less_than =
        crate::xml::extract_element(and_xml, "ObjectSizeLessThan").and_then(|v| v.parse().ok());

    and
}

fn parse_expiration(rule_xml: &str) -> Option<LifecycleExpiration> {
    let exp_start = rule_xml.find("<Expiration>")?;
    let exp_end = rule_xml.find("</Expiration>")?;
    let exp_xml = &rule_xml[exp_start..exp_end + "</Expiration>".len()];

    Some(LifecycleExpiration {
        date: crate::xml::extract_element(exp_xml, "Date"),
        days: crate::xml::extract_element(exp_xml, "Days").and_then(|v| v.parse().ok()),
        expired_object_delete_marker: crate::xml::extract_element(
            exp_xml,
            "ExpiredObjectDeleteMarker",
        )
        .and_then(|v| v.parse().ok()),
    })
}

fn parse_transitions(rule_xml: &str) -> Vec<Transition> {
    let mut transitions = Vec::new();
    let mut search_from = 0;

    while let Some(start) = rule_xml[search_from..].find("<Transition>") {
        let abs_start = search_from + start;
        if let Some(end_offset) = rule_xml[abs_start..].find("</Transition>") {
            let t_xml = &rule_xml[abs_start..abs_start + end_offset + "</Transition>".len()];
            transitions.push(Transition {
                date: crate::xml::extract_element(t_xml, "Date"),
                days: crate::xml::extract_element(t_xml, "Days").and_then(|v| v.parse().ok()),
                storage_class: crate::xml::extract_element(t_xml, "StorageClass"),
            });
            search_from = abs_start + end_offset + "</Transition>".len();
        } else {
            break;
        }
    }

    transitions
}

fn parse_noncurrent_version_transitions(rule_xml: &str) -> Vec<NoncurrentVersionTransition> {
    let mut transitions = Vec::new();
    let mut search_from = 0;

    while let Some(start) = rule_xml[search_from..].find("<NoncurrentVersionTransition>") {
        let abs_start = search_from + start;
        let tag_end = "</NoncurrentVersionTransition>";
        if let Some(end_offset) = rule_xml[abs_start..].find(tag_end) {
            let t_xml = &rule_xml[abs_start..abs_start + end_offset + tag_end.len()];
            transitions.push(NoncurrentVersionTransition {
                noncurrent_days: crate::xml::extract_element(t_xml, "NoncurrentDays")
                    .and_then(|v| v.parse().ok()),
                storage_class: crate::xml::extract_element(t_xml, "StorageClass"),
                newer_noncurrent_versions: crate::xml::extract_element(
                    t_xml,
                    "NewerNoncurrentVersions",
                )
                .and_then(|v| v.parse().ok()),
            });
            search_from = abs_start + end_offset + tag_end.len();
        } else {
            break;
        }
    }

    transitions
}

fn parse_noncurrent_version_expiration(rule_xml: &str) -> Option<NoncurrentVersionExpiration> {
    let tag_start = rule_xml.find("<NoncurrentVersionExpiration>")?;
    let tag_end = "</NoncurrentVersionExpiration>";
    let end = rule_xml.find(tag_end)?;
    let nve_xml = &rule_xml[tag_start..end + tag_end.len()];

    Some(NoncurrentVersionExpiration {
        noncurrent_days: crate::xml::extract_element(nve_xml, "NoncurrentDays")
            .and_then(|v| v.parse().ok()),
        newer_noncurrent_versions: crate::xml::extract_element(nve_xml, "NewerNoncurrentVersions")
            .and_then(|v| v.parse().ok()),
    })
}

fn parse_abort_incomplete_multipart_upload(
    rule_xml: &str,
) -> Option<AbortIncompleteMultipartUpload> {
    let tag_start = rule_xml.find("<AbortIncompleteMultipartUpload>")?;
    let tag_end = "</AbortIncompleteMultipartUpload>";
    let end = rule_xml.find(tag_end)?;
    let abort_xml = &rule_xml[tag_start..end + tag_end.len()];

    Some(AbortIncompleteMultipartUpload {
        days_after_initiation: crate::xml::extract_element(abort_xml, "DaysAfterInitiation")
            .and_then(|v| v.parse().ok()),
    })
}
