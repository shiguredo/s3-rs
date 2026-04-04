//! GetBucketWebsite API
//!
//! バケットのウェブサイト設定を取得する。
//!
//! <https://docs.aws.amazon.com/AmazonS3/latest/API/API_GetBucketWebsite.html>

use crate::client::S3Client;
use crate::error::Error;
use crate::types::{
    ErrorDocument, GetBucketWebsiteOutput, IndexDocument, RedirectAllRequestsTo, RoutingRule,
    RoutingRuleCondition, RoutingRuleRedirect,
};

use super::{S3Request, build_signed_request, parse_error_response, required};

pub struct GetBucketWebsiteFluentBuilder<'a> {
    client: &'a S3Client,
    bucket: Option<String>,
}

impl<'a> GetBucketWebsiteFluentBuilder<'a> {
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
            Some(&[("website", "")]),
        ))
    }

    pub fn parse_response(response: &super::S3Response) -> Result<GetBucketWebsiteOutput, Error> {
        if !response.is_success() {
            return Err(parse_error_response(response));
        }

        let body_text = super::xml_body_text(&response.body)?;
        Ok(parse_website_configuration(body_text))
    }
}

/// WebsiteConfiguration XML をパースする
fn parse_website_configuration(text: &str) -> GetBucketWebsiteOutput {
    use xml::reader::{EventReader, XmlEvent};

    let reader = EventReader::from_str(text);

    let mut index_document: Option<IndexDocument> = None;
    let mut error_document: Option<ErrorDocument> = None;
    let mut redirect_all: Option<RedirectAllRequestsTo> = None;
    let mut routing_rules: Vec<RoutingRule> = Vec::new();

    #[derive(PartialEq)]
    enum Ctx {
        Root,
        IndexDocument,
        ErrorDocument,
        RedirectAll,
        RoutingRules,
        RoutingRule,
        Condition,
        Redirect,
    }

    let mut ctx = Ctx::Root;
    let mut current_tag: Option<String> = None;
    let mut current_text = String::new();

    // IndexDocument
    let mut idx_suffix: Option<String> = None;
    // ErrorDocument
    let mut err_key: Option<String> = None;
    // RedirectAllRequestsTo
    let mut redir_host = String::new();
    let mut redir_protocol: Option<String> = None;
    // Condition
    let mut cond_error_code: Option<String> = None;
    let mut cond_prefix: Option<String> = None;
    // Redirect
    let mut rd_host: Option<String> = None;
    let mut rd_code: Option<String> = None;
    let mut rd_protocol: Option<String> = None;
    let mut rd_replace_prefix: Option<String> = None;
    let mut rd_replace_key: Option<String> = None;

    for event in reader {
        match event {
            Ok(XmlEvent::StartElement { name, .. }) => {
                let tag = &name.local_name;
                match ctx {
                    Ctx::Root => match tag.as_str() {
                        "IndexDocument" => {
                            ctx = Ctx::IndexDocument;
                            idx_suffix = None;
                        }
                        "ErrorDocument" => {
                            ctx = Ctx::ErrorDocument;
                            err_key = None;
                        }
                        "RedirectAllRequestsTo" => {
                            ctx = Ctx::RedirectAll;
                            redir_host.clear();
                            redir_protocol = None;
                        }
                        "RoutingRules" => ctx = Ctx::RoutingRules,
                        _ => {}
                    },
                    Ctx::RoutingRules if tag == "RoutingRule" => {
                        ctx = Ctx::RoutingRule;
                        cond_error_code = None;
                        cond_prefix = None;
                        rd_host = None;
                        rd_code = None;
                        rd_protocol = None;
                        rd_replace_prefix = None;
                        rd_replace_key = None;
                    }
                    Ctx::RoutingRule if tag == "Condition" => ctx = Ctx::Condition,
                    Ctx::RoutingRule if tag == "Redirect" => ctx = Ctx::Redirect,
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
                match ctx {
                    Ctx::IndexDocument if tag == "IndexDocument" => {
                        index_document = idx_suffix.take().map(|s| IndexDocument { suffix: s });
                        ctx = Ctx::Root;
                    }
                    Ctx::IndexDocument => {
                        if current_tag.as_deref() == Some(tag) && tag == "Suffix" {
                            idx_suffix = Some(current_text.clone());
                        }
                        current_tag = None;
                    }
                    Ctx::ErrorDocument if tag == "ErrorDocument" => {
                        error_document = err_key.take().map(|k| ErrorDocument { key: k });
                        ctx = Ctx::Root;
                    }
                    Ctx::ErrorDocument => {
                        if current_tag.as_deref() == Some(tag) && tag == "Key" {
                            err_key = Some(current_text.clone());
                        }
                        current_tag = None;
                    }
                    Ctx::RedirectAll if tag == "RedirectAllRequestsTo" => {
                        redirect_all = Some(RedirectAllRequestsTo {
                            host_name: redir_host.clone(),
                            protocol: redir_protocol.take(),
                        });
                        ctx = Ctx::Root;
                    }
                    Ctx::RedirectAll => {
                        if let Some(ref ct) = current_tag
                            && *ct == *tag
                        {
                            match ct.as_str() {
                                "HostName" => redir_host = current_text.clone(),
                                "Protocol" => redir_protocol = Some(current_text.clone()),
                                _ => {}
                            }
                        }
                        current_tag = None;
                    }
                    Ctx::Condition if tag == "Condition" => ctx = Ctx::RoutingRule,
                    Ctx::Condition => {
                        if let Some(ref ct) = current_tag
                            && *ct == *tag
                        {
                            match ct.as_str() {
                                "HttpErrorCodeReturnedEquals" => {
                                    cond_error_code = Some(current_text.clone())
                                }
                                "KeyPrefixEquals" => cond_prefix = Some(current_text.clone()),
                                _ => {}
                            }
                        }
                        current_tag = None;
                    }
                    Ctx::Redirect if tag == "Redirect" => ctx = Ctx::RoutingRule,
                    Ctx::Redirect => {
                        if let Some(ref ct) = current_tag
                            && *ct == *tag
                        {
                            match ct.as_str() {
                                "HostName" => rd_host = Some(current_text.clone()),
                                "HttpRedirectCode" => rd_code = Some(current_text.clone()),
                                "Protocol" => rd_protocol = Some(current_text.clone()),
                                "ReplaceKeyPrefixWith" => {
                                    rd_replace_prefix = Some(current_text.clone())
                                }
                                "ReplaceKeyWith" => rd_replace_key = Some(current_text.clone()),
                                _ => {}
                            }
                        }
                        current_tag = None;
                    }
                    Ctx::RoutingRule if tag == "RoutingRule" => {
                        let condition = if cond_error_code.is_some() || cond_prefix.is_some() {
                            Some(RoutingRuleCondition {
                                http_error_code_returned_equals: cond_error_code.take(),
                                key_prefix_equals: cond_prefix.take(),
                            })
                        } else {
                            None
                        };
                        let redirect = Some(RoutingRuleRedirect {
                            host_name: rd_host.take(),
                            http_redirect_code: rd_code.take(),
                            protocol: rd_protocol.take(),
                            replace_key_prefix_with: rd_replace_prefix.take(),
                            replace_key_with: rd_replace_key.take(),
                        });
                        routing_rules.push(RoutingRule {
                            condition,
                            redirect,
                        });
                        ctx = Ctx::RoutingRules;
                    }
                    Ctx::RoutingRules if tag == "RoutingRules" => ctx = Ctx::Root,
                    _ => {}
                }
            }
            Err(_) => break,
            _ => {}
        }
    }

    GetBucketWebsiteOutput {
        index_document,
        error_document,
        redirect_all_requests_to: redirect_all,
        routing_rules,
    }
}
