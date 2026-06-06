//! GetBucketCors API
//!
//! バケットに設定された CORS ルールを取得する。
//!
//! <https://docs.aws.amazon.com/AmazonS3/latest/API/API_GetBucketCors.html>

use crate::client::Client;
use crate::error::Error;
use crate::types::{CorsRule, GetBucketCorsOutput};

use super::{S3Request, build_signed_request, parse_error_response, required};

pub struct GetBucketCorsFluentBuilder<'a> {
    client: &'a Client,
    bucket: Option<String>,
}

impl<'a> GetBucketCorsFluentBuilder<'a> {
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
            Some(&[("cors", "")]),
            now,
        )
    }

    pub fn parse_response(response: &super::S3Response) -> Result<GetBucketCorsOutput, Error> {
        if !response.is_success() {
            return Err(parse_error_response(response));
        }

        let body_text = super::xml_body_text(&response.body)?;

        let cors_rules = extract_cors_rules(body_text)?;
        Ok(GetBucketCorsOutput {
            cors_rules: Some(cors_rules),
        })
    }
}

/// CORSRule を XML からパースする
///
/// CORSRule 内の AllowedOrigin, AllowedMethod, AllowedHeader, ExposeHeader は
/// 同名タグが複数出現するため、for_each_element (最後の値のみ保持) は使えない。
/// EventReader で直接パースする。
fn extract_cors_rules(text: &str) -> Result<Vec<CorsRule>, Error> {
    use xml::reader::{EventReader, XmlEvent};

    let reader = EventReader::from_str(text);
    let mut rules = Vec::new();
    let mut inside_rule = false;
    let mut current_tag: Option<String> = None;
    let mut current_text = String::new();

    let mut allowed_origins = Vec::new();
    let mut allowed_methods = Vec::new();
    let mut allowed_headers = Vec::new();
    let mut expose_headers = Vec::new();
    let mut max_age_seconds: Option<i32> = None;

    for event in reader {
        match event {
            Ok(XmlEvent::StartElement { name, .. }) if name.local_name == "CORSRule" => {
                inside_rule = true;
                allowed_origins.clear();
                allowed_methods.clear();
                allowed_headers.clear();
                expose_headers.clear();
                max_age_seconds = None;
            }
            Ok(XmlEvent::StartElement { name, .. }) if inside_rule => {
                current_tag = Some(name.local_name.clone());
                current_text.clear();
            }
            Ok(XmlEvent::Characters(s)) if inside_rule && current_tag.is_some() => {
                current_text.push_str(&s);
            }
            Ok(XmlEvent::EndElement { name }) if inside_rule => {
                if name.local_name == "CORSRule" {
                    rules.push(CorsRule {
                        id: None,
                        allowed_origins: allowed_origins.clone(),
                        allowed_methods: allowed_methods.clone(),
                        allowed_headers: if allowed_headers.is_empty() {
                            None
                        } else {
                            Some(allowed_headers.clone())
                        },
                        max_age_seconds,
                        expose_headers: if expose_headers.is_empty() {
                            None
                        } else {
                            Some(expose_headers.clone())
                        },
                    });
                    inside_rule = false;
                } else if let Some(ref tag) = current_tag {
                    if name.local_name == *tag {
                        match tag.as_str() {
                            "AllowedOrigin" => allowed_origins.push(current_text.clone()),
                            "AllowedMethod" => allowed_methods.push(current_text.clone()),
                            "AllowedHeader" => allowed_headers.push(current_text.clone()),
                            "ExposeHeader" => expose_headers.push(current_text.clone()),
                            "MaxAgeSeconds" => {
                                max_age_seconds = Some(current_text.parse().map_err(|_| {
                                    Error::InvalidResponse(
                                        "failed to parse MaxAgeSeconds in CORSRule".to_string(),
                                    )
                                })?);
                            }
                            _ => {}
                        }
                    }
                    current_tag = None;
                }
            }
            Err(_) => {
                let element = current_tag.as_deref().unwrap_or("unknown");
                let parent = if inside_rule {
                    "CORSRule"
                } else {
                    "CORSConfiguration"
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
