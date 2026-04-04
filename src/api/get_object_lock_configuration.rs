//! GetObjectLockConfiguration API
//!
//! バケットの Object Lock 既定設定を取得する。
//!
//! <https://docs.aws.amazon.com/AmazonS3/latest/API/API_GetObjectLockConfiguration.html>

use crate::client::S3Client;
use crate::error::Error;
use crate::types::{
    DefaultRetention, GetObjectLockConfigurationOutput, ObjectLockConfiguration, ObjectLockRule,
};

use super::{S3Request, build_signed_request, parse_error_response, required};

pub struct GetObjectLockConfigurationFluentBuilder<'a> {
    client: &'a S3Client,
    bucket: Option<String>,
}

impl<'a> GetObjectLockConfigurationFluentBuilder<'a> {
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
            Some(&[("object-lock", "")]),
        ))
    }

    pub fn parse_response(
        response: &super::S3Response,
    ) -> Result<GetObjectLockConfigurationOutput, Error> {
        if !response.is_success() {
            return Err(parse_error_response(response));
        }

        let body_text = super::xml_body_text(&response.body)?;
        Ok(GetObjectLockConfigurationOutput {
            object_lock_configuration: Some(parse_object_lock_configuration(body_text)),
        })
    }
}

/// ObjectLockConfiguration XML をパースする
///
/// DefaultRetention は Rule > DefaultRetention にネストされるため
/// EventReader で直接パースする。
fn parse_object_lock_configuration(text: &str) -> ObjectLockConfiguration {
    use xml::reader::{EventReader, XmlEvent};

    let reader = EventReader::from_str(text);

    let mut object_lock_enabled: Option<String> = None;
    let mut rule: Option<ObjectLockRule> = None;

    let mut inside_rule = false;
    let mut inside_default_retention = false;
    let mut current_tag: Option<String> = None;
    let mut current_text = String::new();

    let mut retention_mode: Option<String> = None;
    let mut retention_days: Option<i32> = None;
    let mut retention_years: Option<i32> = None;

    for event in reader {
        match event {
            Ok(XmlEvent::StartElement { name, .. }) => {
                let tag = &name.local_name;
                match tag.as_str() {
                    "Rule" => {
                        inside_rule = true;
                        retention_mode = None;
                        retention_days = None;
                        retention_years = None;
                    }
                    "DefaultRetention" if inside_rule => {
                        inside_default_retention = true;
                    }
                    _ => {
                        current_tag = Some(tag.clone());
                        current_text.clear();
                    }
                }
            }
            Ok(XmlEvent::Characters(s)) if current_tag.is_some() => {
                current_text.push_str(&s);
            }
            Ok(XmlEvent::EndElement { name }) => {
                let tag = &name.local_name;
                if tag == "DefaultRetention" && inside_default_retention {
                    inside_default_retention = false;
                } else if tag == "Rule" && inside_rule {
                    let default_retention = if retention_mode.is_some()
                        || retention_days.is_some()
                        || retention_years.is_some()
                    {
                        Some(DefaultRetention {
                            mode: retention_mode.take(),
                            days: retention_days.take(),
                            years: retention_years.take(),
                        })
                    } else {
                        None
                    };
                    rule = Some(ObjectLockRule { default_retention });
                    inside_rule = false;
                } else if let Some(ref ct) = current_tag {
                    if *ct == *tag {
                        match ct.as_str() {
                            "ObjectLockEnabled" => object_lock_enabled = Some(current_text.clone()),
                            "Mode" if inside_default_retention => {
                                retention_mode = Some(current_text.clone())
                            }
                            "Days" if inside_default_retention => {
                                retention_days = current_text.parse().ok()
                            }
                            "Years" if inside_default_retention => {
                                retention_years = current_text.parse().ok()
                            }
                            _ => {}
                        }
                    }
                    current_tag = None;
                }
            }
            Err(_) => break,
            _ => {}
        }
    }

    ObjectLockConfiguration {
        object_lock_enabled,
        rule,
    }
}
