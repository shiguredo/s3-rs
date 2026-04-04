//! PutBucketLifecycleConfiguration API
//!
//! バケットのライフサイクル設定を設定する。既存の設定は上書きされる。
//!
//! <https://docs.aws.amazon.com/AmazonS3/latest/API/API_PutBucketLifecycleConfiguration.html>

use crate::client::S3Client;
use crate::error::Error;
use crate::types::{LifecycleRule, PutBucketLifecycleConfigurationOutput};

use super::{S3Request, base64_md5, build_signed_request, parse_error_response, required};

pub struct PutBucketLifecycleConfigurationFluentBuilder<'a> {
    client: &'a S3Client,
    bucket: Option<String>,
    rules: Vec<LifecycleRule>,
    checksum_algorithm: Option<String>,
    /// 遷移対象のデフォルト最小オブジェクトサイズ
    transition_default_minimum_object_size: Option<String>,
}

impl<'a> PutBucketLifecycleConfigurationFluentBuilder<'a> {
    pub(crate) fn new(client: &'a S3Client) -> Self {
        Self {
            client,
            bucket: None,
            rules: Vec::new(),
            checksum_algorithm: None,
            transition_default_minimum_object_size: None,
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

    /// 遷移対象のデフォルト最小オブジェクトサイズを指定する
    ///
    /// "varies_by_storage_class" または "all_storage_classes_128K"
    pub fn transition_default_minimum_object_size(mut self, value: impl Into<String>) -> Self {
        self.transition_default_minimum_object_size = Some(value.into());
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

        if let Some(ref size) = self.transition_default_minimum_object_size {
            extra_headers.push((
                "x-amz-transition-default-minimum-object-size",
                size.as_str(),
            ));
        }

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
        w.start("Rule");

        if let Some(ref id) = rule.id {
            w.element("ID", id);
        }

        // Filter
        if let Some(ref f) = rule.filter {
            w.start("Filter");
            if let Some(ref and) = f.and {
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
            } else {
                if let Some(ref prefix) = f.prefix {
                    w.element("Prefix", prefix);
                }
                if let Some(ref tag) = f.tag {
                    w.start("Tag");
                    w.element("Key", &tag.key);
                    w.element("Value", &tag.value);
                    w.end();
                }
                if let Some(size) = f.object_size_greater_than {
                    w.element("ObjectSizeGreaterThan", &size.to_string());
                }
                if let Some(size) = f.object_size_less_than {
                    w.element("ObjectSizeLessThan", &size.to_string());
                }
            }
            w.end();
        }

        w.element("Status", rule.status.as_str());

        // Expiration
        if let Some(ref exp) = rule.expiration {
            w.start("Expiration");
            if let Some(days) = exp.days {
                w.element("Days", &days.to_string());
            }
            if let Some(ref date) = exp.date {
                w.element("Date", date);
            }
            if let Some(marker) = exp.expired_object_delete_marker {
                w.element(
                    "ExpiredObjectDeleteMarker",
                    if marker { "true" } else { "false" },
                );
            }
            w.end();
        }

        // Transitions
        for trans in rule.transitions.iter().flatten() {
            w.start("Transition");
            if let Some(days) = trans.days {
                w.element("Days", &days.to_string());
            }
            if let Some(ref date) = trans.date {
                w.element("Date", date);
            }
            if let Some(ref sc) = trans.storage_class {
                w.element("StorageClass", sc);
            }
            w.end();
        }

        // NoncurrentVersionExpiration
        if let Some(ref nve) = rule.noncurrent_version_expiration {
            w.start("NoncurrentVersionExpiration");
            if let Some(days) = nve.noncurrent_days {
                w.element("NoncurrentDays", &days.to_string());
            }
            if let Some(newer) = nve.newer_noncurrent_versions {
                w.element("NewerNoncurrentVersions", &newer.to_string());
            }
            w.end();
        }

        // NoncurrentVersionTransitions
        for nvt in rule.noncurrent_version_transitions.iter().flatten() {
            w.start("NoncurrentVersionTransition");
            if let Some(days) = nvt.noncurrent_days {
                w.element("NoncurrentDays", &days.to_string());
            }
            if let Some(ref sc) = nvt.storage_class {
                w.element("StorageClass", sc);
            }
            if let Some(newer) = nvt.newer_noncurrent_versions {
                w.element("NewerNoncurrentVersions", &newer.to_string());
            }
            w.end();
        }

        // AbortIncompleteMultipartUpload
        if let Some(ref abort) = rule.abort_incomplete_multipart_upload {
            w.start("AbortIncompleteMultipartUpload");
            if let Some(days) = abort.days_after_initiation {
                w.element("DaysAfterInitiation", &days.to_string());
            }
            w.end();
        }

        w.end(); // Rule
    }
    w.end(); // LifecycleConfiguration
    w.finish()
}
