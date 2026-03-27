//! PutBucketLifecycleConfiguration API
//!
//! バケットにライフサイクル設定を適用する。既存の設定は全て上書きされる。
//!
//! <https://docs.aws.amazon.com/AmazonS3/latest/API/API_PutBucketLifecycleConfiguration.html>

use crate::client::S3Client;
use crate::error::Error;
use crate::types::{
    AbortIncompleteMultipartUpload, LifecycleExpiration, LifecycleRule, LifecycleRuleAndOperator,
    LifecycleRuleFilter, NoncurrentVersionExpiration, NoncurrentVersionTransition,
    PutBucketLifecycleConfigurationOutput, Transition,
};

use super::{S3Request, base64_md5, build_signed_request, parse_error_response, required};

pub struct PutBucketLifecycleConfigurationFluentBuilder<'a> {
    client: &'a S3Client,
    bucket: Option<String>,
    rules: Vec<LifecycleRule>,
    checksum_algorithm: Option<String>,
}

impl<'a> PutBucketLifecycleConfigurationFluentBuilder<'a> {
    pub(crate) fn new(client: &'a S3Client) -> Self {
        Self {
            client,
            bucket: None,
            rules: Vec::new(),
            checksum_algorithm: None,
        }
    }

    pub fn bucket(mut self, bucket: impl Into<String>) -> Self {
        self.bucket = Some(bucket.into());
        self
    }

    /// ライフサイクルルールを追加する
    pub fn rule(mut self, rule: LifecycleRule) -> Self {
        self.rules.push(rule);
        self
    }

    /// チェックサムアルゴリズムを指定する (CRC32, CRC32C, SHA1, SHA256, CRC64NVME)
    pub fn checksum_algorithm(mut self, algorithm: impl Into<String>) -> Self {
        self.checksum_algorithm = Some(algorithm.into());
        self
    }

    pub fn build_request(&self) -> Result<S3Request, Error> {
        let bucket = required(self.bucket.as_deref(), "bucket")?;

        let xml_body = build_lifecycle_xml(&self.rules);
        let content_md5 = base64_md5(xml_body.as_bytes());
        let mut extra_headers: Vec<(&str, &str)> = vec![
            ("content-type", "application/xml"),
            ("content-md5", content_md5.as_str()),
        ];

        let computed_checksum;
        if let Some(ref algo_str) = self.checksum_algorithm {
            extra_headers.push(("x-amz-checksum-algorithm", algo_str.as_str()));
            let algorithm: crate::checksum::ChecksumAlgorithm = algo_str.parse()?;
            computed_checksum = crate::checksum::compute_checksum(algorithm, xml_body.as_bytes());
            extra_headers.push((algorithm.header_name(), &computed_checksum));
        }

        Ok(build_signed_request(
            &self.client.config_ref(),
            "PUT",
            bucket,
            "",
            &extra_headers,
            xml_body.as_bytes(),
            Some(&[("lifecycle", "")]),
        ))
    }

    pub fn parse_response(
        response: &super::S3Response,
    ) -> Result<PutBucketLifecycleConfigurationOutput, Error> {
        if !response.is_success() {
            return Err(parse_error_response(response));
        }
        Ok(PutBucketLifecycleConfigurationOutput {})
    }
}

fn build_lifecycle_xml(rules: &[LifecycleRule]) -> String {
    let mut w = crate::xml::XmlWriter::new();
    w.start_ns("LifecycleConfiguration", crate::xml::S3_NS);
    for rule in rules {
        write_rule(&mut w, rule);
    }
    w.end();
    w.finish()
}

/// aws-sdk-rust 互換の要素順序で Rule を書き出す
///
/// aws-sdk-rust の ser_lifecycle_rule が生成する順序:
/// Expiration → ID → Prefix → Filter → Status → Transition →
/// NoncurrentVersionTransition → NoncurrentVersionExpiration →
/// AbortIncompleteMultipartUpload
fn write_rule(w: &mut crate::xml::XmlWriter, rule: &LifecycleRule) {
    w.start("Rule");

    if let Some(ref expiration) = rule.expiration {
        write_expiration(w, expiration);
    }

    if let Some(ref id) = rule.id {
        w.element("ID", id);
    }

    if let Some(ref filter) = rule.filter {
        write_filter(w, filter);
    }

    w.element("Status", rule.status.as_str());

    if let Some(ref transitions) = rule.transitions {
        for transition in transitions {
            write_transition(w, transition);
        }
    }

    if let Some(ref nv_transitions) = rule.noncurrent_version_transitions {
        for nv_transition in nv_transitions {
            write_noncurrent_version_transition(w, nv_transition);
        }
    }

    if let Some(ref nv_expiration) = rule.noncurrent_version_expiration {
        write_noncurrent_version_expiration(w, nv_expiration);
    }

    if let Some(ref abort) = rule.abort_incomplete_multipart_upload {
        write_abort_incomplete_multipart_upload(w, abort);
    }

    w.end();
}

fn write_filter(w: &mut crate::xml::XmlWriter, filter: &LifecycleRuleFilter) {
    w.start("Filter");

    if let Some(ref and) = filter.and {
        write_and_operator(w, and);
    } else {
        if let Some(ref prefix) = filter.prefix {
            w.element("Prefix", prefix);
        }
        if let Some(ref tag) = filter.tag {
            w.start("Tag");
            w.element("Key", &tag.key);
            w.element("Value", &tag.value);
            w.end();
        }
        if let Some(size) = filter.object_size_greater_than {
            w.element("ObjectSizeGreaterThan", &size.to_string());
        }
        if let Some(size) = filter.object_size_less_than {
            w.element("ObjectSizeLessThan", &size.to_string());
        }
    }

    w.end();
}

fn write_and_operator(w: &mut crate::xml::XmlWriter, and: &LifecycleRuleAndOperator) {
    w.start("And");

    if let Some(ref prefix) = and.prefix {
        w.element("Prefix", prefix);
    }
    if let Some(ref tags) = and.tags {
        for tag in tags {
            w.start("Tag");
            w.element("Key", &tag.key);
            w.element("Value", &tag.value);
            w.end();
        }
    }
    if let Some(size) = and.object_size_greater_than {
        w.element("ObjectSizeGreaterThan", &size.to_string());
    }
    if let Some(size) = and.object_size_less_than {
        w.element("ObjectSizeLessThan", &size.to_string());
    }

    w.end();
}

fn write_expiration(w: &mut crate::xml::XmlWriter, expiration: &LifecycleExpiration) {
    w.start("Expiration");

    if let Some(ref date) = expiration.date {
        w.element("Date", date);
    }
    if let Some(days) = expiration.days {
        w.element("Days", &days.to_string());
    }
    if let Some(marker) = expiration.expired_object_delete_marker {
        w.element("ExpiredObjectDeleteMarker", &marker.to_string());
    }

    w.end();
}

fn write_transition(w: &mut crate::xml::XmlWriter, transition: &Transition) {
    w.start("Transition");

    if let Some(ref date) = transition.date {
        w.element("Date", date);
    }
    if let Some(days) = transition.days {
        w.element("Days", &days.to_string());
    }
    if let Some(ref storage_class) = transition.storage_class {
        w.element("StorageClass", storage_class);
    }

    w.end();
}

fn write_noncurrent_version_transition(
    w: &mut crate::xml::XmlWriter,
    nv_transition: &NoncurrentVersionTransition,
) {
    w.start("NoncurrentVersionTransition");

    if let Some(days) = nv_transition.noncurrent_days {
        w.element("NoncurrentDays", &days.to_string());
    }
    if let Some(ref storage_class) = nv_transition.storage_class {
        w.element("StorageClass", storage_class);
    }
    if let Some(versions) = nv_transition.newer_noncurrent_versions {
        w.element("NewerNoncurrentVersions", &versions.to_string());
    }

    w.end();
}

fn write_noncurrent_version_expiration(
    w: &mut crate::xml::XmlWriter,
    nv_expiration: &NoncurrentVersionExpiration,
) {
    w.start("NoncurrentVersionExpiration");

    if let Some(days) = nv_expiration.noncurrent_days {
        w.element("NoncurrentDays", &days.to_string());
    }
    if let Some(versions) = nv_expiration.newer_noncurrent_versions {
        w.element("NewerNoncurrentVersions", &versions.to_string());
    }

    w.end();
}

fn write_abort_incomplete_multipart_upload(
    w: &mut crate::xml::XmlWriter,
    abort: &AbortIncompleteMultipartUpload,
) {
    w.start("AbortIncompleteMultipartUpload");

    if let Some(days) = abort.days_after_initiation {
        w.element("DaysAfterInitiation", &days.to_string());
    }

    w.end();
}
