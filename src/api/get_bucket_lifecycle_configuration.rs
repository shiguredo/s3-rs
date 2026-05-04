//! GetBucketLifecycleConfiguration API
//!
//! バケットのライフサイクル設定を取得する。
//!
//! <https://docs.aws.amazon.com/AmazonS3/latest/API/API_GetBucketLifecycleConfiguration.html>

use crate::client::Client;
use crate::error::Error;
use crate::types::{
    AbortIncompleteMultipartUpload, ExpirationStatus, GetBucketLifecycleConfigurationOutput,
    LifecycleExpiration, LifecycleRule, LifecycleRuleAndOperator, LifecycleRuleFilter,
    NoncurrentVersionExpiration, NoncurrentVersionTransition, Tag, Transition,
};

use super::{S3Request, build_signed_request, parse_error_response, required};

pub struct GetBucketLifecycleConfigurationFluentBuilder<'a> {
    client: &'a Client,
    bucket: Option<String>,
}

impl<'a> GetBucketLifecycleConfigurationFluentBuilder<'a> {
    pub(crate) fn new(client: &'a Client) -> Self {
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

        let body_text = super::xml_body_text(&response.body)?;

        Ok(GetBucketLifecycleConfigurationOutput {
            rules: extract_lifecycle_rules(body_text),
        })
    }
}

/// LifecycleConfiguration から Rule をパースする
///
/// Rule 内にネストされた要素 (Filter, Expiration, Transition 等) があるため
/// EventReader で直接パースする。
fn extract_lifecycle_rules(text: &str) -> Vec<LifecycleRule> {
    use xml::reader::{EventReader, XmlEvent};

    let reader = EventReader::from_str(text);
    let mut rules = Vec::new();

    // パーサー状態
    #[derive(PartialEq)]
    enum Context {
        None,
        Rule,
        Filter,
        FilterAnd,
        FilterTag,
        FilterAndTag,
        Expiration,
        Transition,
        NoncurrentVersionExpiration,
        NoncurrentVersionTransition,
        AbortIncompleteMultipartUpload,
    }

    let mut ctx = Context::None;
    let mut current_tag: Option<String> = None;
    let mut current_text = String::new();

    // Rule フィールド
    let mut id: Option<String> = None;
    let mut status = String::new();
    let mut filter: Option<LifecycleRuleFilter> = None;
    let mut expiration: Option<LifecycleExpiration> = None;
    let mut transitions: Vec<Transition> = Vec::new();
    let mut nv_expiration: Option<NoncurrentVersionExpiration> = None;
    let mut nv_transitions: Vec<NoncurrentVersionTransition> = Vec::new();
    let mut abort_incomplete: Option<AbortIncompleteMultipartUpload> = None;

    // Filter フィールド
    let mut filter_prefix: Option<String> = None;
    let mut filter_tag: Option<Tag> = None;
    let mut filter_size_gt: Option<i64> = None;
    let mut filter_size_lt: Option<i64> = None;
    let mut filter_and: Option<LifecycleRuleAndOperator> = None;

    // FilterAnd フィールド
    let mut and_prefix: Option<String> = None;
    let mut and_tags: Vec<Tag> = Vec::new();
    let mut and_size_gt: Option<i64> = None;
    let mut and_size_lt: Option<i64> = None;

    // Tag フィールド
    let mut tag_key = String::new();
    let mut tag_value = String::new();

    // Expiration フィールド
    let mut exp_days: Option<i32> = None;
    let mut exp_date: Option<String> = None;
    let mut exp_delete_marker: Option<bool> = None;

    // Transition フィールド
    let mut trans_days: Option<i32> = None;
    let mut trans_date: Option<String> = None;
    let mut trans_storage_class: Option<String> = None;

    // NoncurrentVersionExpiration フィールド
    let mut nv_exp_days: Option<i32> = None;
    let mut nv_exp_newer: Option<i32> = None;

    // NoncurrentVersionTransition フィールド
    let mut nv_trans_days: Option<i32> = None;
    let mut nv_trans_storage_class: Option<String> = None;
    let mut nv_trans_newer: Option<i32> = None;

    // AbortIncompleteMultipartUpload フィールド
    let mut abort_days: Option<i32> = None;

    for event in reader {
        match event {
            Ok(XmlEvent::StartElement { name, .. }) => {
                let tag_name = &name.local_name;
                match ctx {
                    Context::None if tag_name == "Rule" => {
                        ctx = Context::Rule;
                        id = None;
                        status.clear();
                        filter = None;
                        expiration = None;
                        transitions.clear();
                        nv_expiration = None;
                        nv_transitions.clear();
                        abort_incomplete = None;
                    }
                    Context::Rule => match tag_name.as_str() {
                        "Filter" => {
                            ctx = Context::Filter;
                            filter_prefix = None;
                            filter_tag = None;
                            filter_size_gt = None;
                            filter_size_lt = None;
                            filter_and = None;
                        }
                        "Expiration" => {
                            ctx = Context::Expiration;
                            exp_days = None;
                            exp_date = None;
                            exp_delete_marker = None;
                        }
                        "Transition" => {
                            ctx = Context::Transition;
                            trans_days = None;
                            trans_date = None;
                            trans_storage_class = None;
                        }
                        "NoncurrentVersionExpiration" => {
                            ctx = Context::NoncurrentVersionExpiration;
                            nv_exp_days = None;
                            nv_exp_newer = None;
                        }
                        "NoncurrentVersionTransition" => {
                            ctx = Context::NoncurrentVersionTransition;
                            nv_trans_days = None;
                            nv_trans_storage_class = None;
                            nv_trans_newer = None;
                        }
                        "AbortIncompleteMultipartUpload" => {
                            ctx = Context::AbortIncompleteMultipartUpload;
                            abort_days = None;
                        }
                        _ => {
                            current_tag = Some(tag_name.clone());
                            current_text.clear();
                        }
                    },
                    Context::Filter if tag_name == "And" => {
                        ctx = Context::FilterAnd;
                        and_prefix = None;
                        and_tags.clear();
                        and_size_gt = None;
                        and_size_lt = None;
                    }
                    Context::Filter if tag_name == "Tag" => {
                        ctx = Context::FilterTag;
                        tag_key.clear();
                        tag_value.clear();
                    }
                    Context::FilterAnd if tag_name == "Tag" => {
                        ctx = Context::FilterAndTag;
                        tag_key.clear();
                        tag_value.clear();
                    }
                    _ => {
                        current_tag = Some(tag_name.clone());
                        current_text.clear();
                    }
                }
            }
            Ok(XmlEvent::Characters(s)) if current_tag.is_some() => {
                current_text.push_str(&s);
            }
            Ok(XmlEvent::EndElement { name }) => {
                let tag_name = &name.local_name;
                match ctx {
                    Context::Rule if tag_name == "Rule" => {
                        rules.push(LifecycleRule {
                            id: id.take(),
                            filter: filter.take(),
                            status: status
                                .parse::<ExpirationStatus>()
                                .unwrap_or(ExpirationStatus::Enabled),
                            expiration: expiration.take(),
                            transitions: if transitions.is_empty() {
                                None
                            } else {
                                Some(transitions.clone())
                            },
                            noncurrent_version_expiration: nv_expiration.take(),
                            noncurrent_version_transitions: if nv_transitions.is_empty() {
                                None
                            } else {
                                Some(nv_transitions.clone())
                            },
                            abort_incomplete_multipart_upload: abort_incomplete.take(),
                        });
                        ctx = Context::None;
                    }
                    Context::Rule => {
                        if let Some(ref tag) = current_tag
                            && *tag == *tag_name
                        {
                            match tag.as_str() {
                                "ID" => id = Some(current_text.clone()),
                                "Status" => status = current_text.clone(),
                                _ => {}
                            }
                        }
                        current_tag = None;
                    }
                    Context::Filter if tag_name == "Filter" => {
                        filter = Some(LifecycleRuleFilter {
                            prefix: filter_prefix.take(),
                            tag: filter_tag.take(),
                            object_size_greater_than: filter_size_gt.take(),
                            object_size_less_than: filter_size_lt.take(),
                            and: filter_and.take(),
                        });
                        ctx = Context::Rule;
                    }
                    Context::Filter => {
                        if let Some(ref tag) = current_tag
                            && *tag == *tag_name
                        {
                            match tag.as_str() {
                                "Prefix" => filter_prefix = Some(current_text.clone()),
                                "ObjectSizeGreaterThan" => {
                                    filter_size_gt = current_text.parse().ok()
                                }
                                "ObjectSizeLessThan" => filter_size_lt = current_text.parse().ok(),
                                _ => {}
                            }
                        }
                        current_tag = None;
                    }
                    Context::FilterTag if tag_name == "Tag" => {
                        filter_tag = Some(Tag {
                            key: tag_key.clone(),
                            value: tag_value.clone(),
                        });
                        ctx = Context::Filter;
                    }
                    Context::FilterTag => {
                        if let Some(ref tag) = current_tag
                            && *tag == *tag_name
                        {
                            match tag.as_str() {
                                "Key" => tag_key = current_text.clone(),
                                "Value" => tag_value = current_text.clone(),
                                _ => {}
                            }
                        }
                        current_tag = None;
                    }
                    Context::FilterAnd if tag_name == "And" => {
                        filter_and = Some(LifecycleRuleAndOperator {
                            prefix: and_prefix.take(),
                            tags: if and_tags.is_empty() {
                                None
                            } else {
                                Some(and_tags.clone())
                            },
                            object_size_greater_than: and_size_gt.take(),
                            object_size_less_than: and_size_lt.take(),
                        });
                        ctx = Context::Filter;
                    }
                    Context::FilterAnd => {
                        if let Some(ref tag) = current_tag
                            && *tag == *tag_name
                        {
                            match tag.as_str() {
                                "Prefix" => and_prefix = Some(current_text.clone()),
                                "ObjectSizeGreaterThan" => and_size_gt = current_text.parse().ok(),
                                "ObjectSizeLessThan" => and_size_lt = current_text.parse().ok(),
                                _ => {}
                            }
                        }
                        current_tag = None;
                    }
                    Context::FilterAndTag if tag_name == "Tag" => {
                        and_tags.push(Tag {
                            key: tag_key.clone(),
                            value: tag_value.clone(),
                        });
                        ctx = Context::FilterAnd;
                    }
                    Context::FilterAndTag => {
                        if let Some(ref tag) = current_tag
                            && *tag == *tag_name
                        {
                            match tag.as_str() {
                                "Key" => tag_key = current_text.clone(),
                                "Value" => tag_value = current_text.clone(),
                                _ => {}
                            }
                        }
                        current_tag = None;
                    }
                    Context::Expiration if tag_name == "Expiration" => {
                        expiration = Some(LifecycleExpiration {
                            days: exp_days.take(),
                            date: exp_date.take(),
                            expired_object_delete_marker: exp_delete_marker.take(),
                        });
                        ctx = Context::Rule;
                    }
                    Context::Expiration => {
                        if let Some(ref tag) = current_tag
                            && *tag == *tag_name
                        {
                            match tag.as_str() {
                                "Days" => exp_days = current_text.parse().ok(),
                                "Date" => exp_date = Some(current_text.clone()),
                                "ExpiredObjectDeleteMarker" => {
                                    exp_delete_marker = Some(current_text == "true")
                                }
                                _ => {}
                            }
                        }
                        current_tag = None;
                    }
                    Context::Transition if tag_name == "Transition" => {
                        transitions.push(Transition {
                            days: trans_days.take(),
                            date: trans_date.take(),
                            storage_class: trans_storage_class
                                .take()
                                .map(|s| crate::types::StorageClass::from(s.as_str())),
                        });
                        ctx = Context::Rule;
                    }
                    Context::Transition => {
                        if let Some(ref tag) = current_tag
                            && *tag == *tag_name
                        {
                            match tag.as_str() {
                                "Days" => trans_days = current_text.parse().ok(),
                                "Date" => trans_date = Some(current_text.clone()),
                                "StorageClass" => trans_storage_class = Some(current_text.clone()),
                                _ => {}
                            }
                        }
                        current_tag = None;
                    }
                    Context::NoncurrentVersionExpiration
                        if tag_name == "NoncurrentVersionExpiration" =>
                    {
                        nv_expiration = Some(NoncurrentVersionExpiration {
                            noncurrent_days: nv_exp_days.take(),
                            newer_noncurrent_versions: nv_exp_newer.take(),
                        });
                        ctx = Context::Rule;
                    }
                    Context::NoncurrentVersionExpiration => {
                        if let Some(ref tag) = current_tag
                            && *tag == *tag_name
                        {
                            match tag.as_str() {
                                "NoncurrentDays" => nv_exp_days = current_text.parse().ok(),
                                "NewerNoncurrentVersions" => {
                                    nv_exp_newer = current_text.parse().ok()
                                }
                                _ => {}
                            }
                        }
                        current_tag = None;
                    }
                    Context::NoncurrentVersionTransition
                        if tag_name == "NoncurrentVersionTransition" =>
                    {
                        nv_transitions.push(NoncurrentVersionTransition {
                            noncurrent_days: nv_trans_days.take(),
                            storage_class: nv_trans_storage_class
                                .take()
                                .map(|s| crate::types::StorageClass::from(s.as_str())),
                            newer_noncurrent_versions: nv_trans_newer.take(),
                        });
                        ctx = Context::Rule;
                    }
                    Context::NoncurrentVersionTransition => {
                        if let Some(ref tag) = current_tag
                            && *tag == *tag_name
                        {
                            match tag.as_str() {
                                "NoncurrentDays" => nv_trans_days = current_text.parse().ok(),
                                "StorageClass" => {
                                    nv_trans_storage_class = Some(current_text.clone())
                                }
                                "NewerNoncurrentVersions" => {
                                    nv_trans_newer = current_text.parse().ok()
                                }
                                _ => {}
                            }
                        }
                        current_tag = None;
                    }
                    Context::AbortIncompleteMultipartUpload
                        if tag_name == "AbortIncompleteMultipartUpload" =>
                    {
                        abort_incomplete = Some(AbortIncompleteMultipartUpload {
                            days_after_initiation: abort_days.take(),
                        });
                        ctx = Context::Rule;
                    }
                    Context::AbortIncompleteMultipartUpload => {
                        if let Some(ref tag) = current_tag
                            && *tag == *tag_name
                            && tag == "DaysAfterInitiation"
                        {
                            abort_days = current_text.parse().ok();
                        }
                        current_tag = None;
                    }
                    _ => {}
                }
            }
            Err(_) => return rules,
            _ => {}
        }
    }

    rules
}
