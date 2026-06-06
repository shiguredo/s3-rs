//! GetBucketEncryption API
//!
//! バケットのデフォルト暗号化設定を取得する。
//!
//! <https://docs.aws.amazon.com/AmazonS3/latest/API/API_GetBucketEncryption.html>

use crate::client::Client;
use crate::error::Error;
use crate::types::{
    GetBucketEncryptionOutput, ServerSideEncryptionByDefault, ServerSideEncryptionConfiguration,
    ServerSideEncryptionRule,
};

use super::{S3Request, build_signed_request, parse_error_response, required};

pub struct GetBucketEncryptionFluentBuilder<'a> {
    client: &'a Client,
    bucket: Option<String>,
}

impl<'a> GetBucketEncryptionFluentBuilder<'a> {
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

    pub fn build_request(&self, now: std::time::SystemTime) -> Result<S3Request, Error> {
        let bucket = required(self.bucket.as_deref(), "bucket")?;
        build_signed_request(
            &self.client.config_ref(),
            "GET",
            bucket,
            "",
            &[],
            b"",
            Some(&[("encryption", "")]),
            now,
        )
    }

    pub fn parse_response(
        response: &super::S3Response,
    ) -> Result<GetBucketEncryptionOutput, Error> {
        if !response.is_success() {
            return Err(parse_error_response(response));
        }

        let body_text = super::xml_body_text(&response.body)?;

        let rules = extract_encryption_rules(body_text)?;
        Ok(GetBucketEncryptionOutput {
            server_side_encryption_configuration: if rules.is_empty() {
                None
            } else {
                Some(ServerSideEncryptionConfiguration { rules })
            },
        })
    }
}

/// ServerSideEncryptionConfiguration から Rule を抽出する
///
/// Rule 内に ApplyServerSideEncryptionByDefault がネストされているため、
/// for_each_element (直接の子要素のみ) では対応できない。EventReader で直接パースする。
fn extract_encryption_rules(text: &str) -> Result<Vec<ServerSideEncryptionRule>, Error> {
    use xml::reader::{EventReader, XmlEvent};

    let reader = EventReader::from_str(text);
    let mut rules = Vec::new();
    let mut inside_rule = false;
    let mut inside_default = false;
    let mut current_tag: Option<String> = None;
    let mut current_text = String::new();

    let mut sse_algorithm: Option<String> = None;
    let mut kms_master_key_id: Option<String> = None;
    let mut bucket_key_enabled: Option<bool> = None;

    for event in reader {
        match event {
            Ok(XmlEvent::StartElement { name, .. }) if name.local_name == "Rule" => {
                inside_rule = true;
                sse_algorithm = None;
                kms_master_key_id = None;
                bucket_key_enabled = None;
            }
            Ok(XmlEvent::StartElement { name, .. })
                if inside_rule && name.local_name == "ApplyServerSideEncryptionByDefault" =>
            {
                inside_default = true;
            }
            Ok(XmlEvent::StartElement { name, .. }) if inside_rule => {
                current_tag = Some(name.local_name.clone());
                current_text.clear();
            }
            Ok(XmlEvent::Characters(s)) if inside_rule && current_tag.is_some() => {
                current_text.push_str(&s);
            }
            Ok(XmlEvent::EndElement { name }) if inside_rule => {
                if name.local_name == "Rule" {
                    let default = sse_algorithm
                        .take()
                        .map(|algo| ServerSideEncryptionByDefault {
                            sse_algorithm: algo,
                            kms_master_key_id: kms_master_key_id.take(),
                        });
                    rules.push(ServerSideEncryptionRule {
                        apply_server_side_encryption_by_default: default,
                        bucket_key_enabled: bucket_key_enabled.take(),
                    });
                    inside_rule = false;
                    inside_default = false;
                } else if name.local_name == "ApplyServerSideEncryptionByDefault" {
                    inside_default = false;
                } else if let Some(ref tag) = current_tag {
                    if name.local_name == *tag {
                        match tag.as_str() {
                            "SSEAlgorithm" if inside_default => {
                                sse_algorithm = Some(current_text.clone());
                            }
                            "KMSMasterKeyID" if inside_default => {
                                kms_master_key_id = Some(current_text.clone());
                            }
                            "BucketKeyEnabled" => {
                                bucket_key_enabled = Some(current_text == "true");
                            }
                            _ => {}
                        }
                    }
                    current_tag = None;
                }
            }
            Err(_) => {
                let element = current_tag.as_deref().unwrap_or("unknown");
                let parent = if inside_default {
                    "ApplyServerSideEncryptionByDefault"
                } else if inside_rule {
                    "Rule"
                } else {
                    "ServerSideEncryptionConfiguration"
                };
                return Err(Error::InvalidResponse(format!(
                    "failed to parse {element} in {parent}"
                )));
            }
            _ => {}
        }
    }

    Ok(rules)
}
