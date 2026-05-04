//! PutBucketWebsite API
//!
//! バケットのウェブサイト設定を有効化・更新する。
//!
//! <https://docs.aws.amazon.com/AmazonS3/latest/API/API_PutBucketWebsite.html>

use crate::client::Client;
use crate::error::Error;
use crate::types::{
    ErrorDocument, IndexDocument, PutBucketWebsiteOutput, RedirectAllRequestsTo, RoutingRule,
};

use super::{S3Request, base64_md5, build_signed_request, parse_error_response, required};

pub struct PutBucketWebsiteFluentBuilder<'a> {
    client: &'a Client,
    bucket: Option<String>,
    index_document: Option<IndexDocument>,
    error_document: Option<ErrorDocument>,
    redirect_all_requests_to: Option<RedirectAllRequestsTo>,
    routing_rules: Vec<RoutingRule>,
}

impl<'a> PutBucketWebsiteFluentBuilder<'a> {
    pub(crate) fn new(client: &'a Client) -> Self {
        Self {
            client,
            bucket: None,
            index_document: None,
            error_document: None,
            redirect_all_requests_to: None,
            routing_rules: Vec::new(),
        }
    }

    pub fn bucket(mut self, bucket: impl Into<String>) -> Self {
        self.bucket = Some(bucket.into());
        self
    }

    /// インデックスドキュメントを設定する
    pub fn index_document(mut self, doc: IndexDocument) -> Self {
        self.index_document = Some(doc);
        self
    }

    /// エラードキュメントを設定する
    pub fn error_document(mut self, doc: ErrorDocument) -> Self {
        self.error_document = Some(doc);
        self
    }

    /// 全リクエストのリダイレクト先を設定する
    pub fn redirect_all_requests_to(mut self, redirect: RedirectAllRequestsTo) -> Self {
        self.redirect_all_requests_to = Some(redirect);
        self
    }

    /// ルーティングルールを追加する
    pub fn routing_rule(mut self, rule: RoutingRule) -> Self {
        self.routing_rules.push(rule);
        self
    }

    pub fn build_request(&self) -> Result<S3Request, Error> {
        let bucket = required(self.bucket.as_deref(), "bucket")?;

        let xml_body = build_website_xml(
            &self.index_document,
            &self.error_document,
            &self.redirect_all_requests_to,
            &self.routing_rules,
        );
        let content_md5 = base64_md5(xml_body.as_bytes());
        let extra_headers: Vec<(&str, &str)> = vec![
            ("content-type", "application/xml"),
            ("content-md5", content_md5.as_str()),
        ];

        Ok(build_signed_request(
            &self.client.config_ref(),
            "PUT",
            bucket,
            "",
            &extra_headers,
            xml_body.as_bytes(),
            Some(&[("website", "")]),
        ))
    }

    pub fn parse_response(response: &super::S3Response) -> Result<PutBucketWebsiteOutput, Error> {
        if !response.is_success() {
            return Err(parse_error_response(response));
        }
        Ok(PutBucketWebsiteOutput {})
    }
}

fn build_website_xml(
    index: &Option<IndexDocument>,
    error: &Option<ErrorDocument>,
    redirect_all: &Option<RedirectAllRequestsTo>,
    routing_rules: &[RoutingRule],
) -> String {
    let mut w = crate::xml::XmlWriter::new();
    w.start_ns("WebsiteConfiguration", crate::xml::S3_NS);

    if let Some(redirect) = redirect_all {
        w.start("RedirectAllRequestsTo");
        w.element("HostName", &redirect.host_name);
        if let Some(ref protocol) = redirect.protocol {
            w.element("Protocol", protocol);
        }
        w.end();
    }

    if let Some(idx) = index {
        w.start("IndexDocument");
        w.element("Suffix", &idx.suffix);
        w.end();
    }

    if let Some(err) = error {
        w.start("ErrorDocument");
        w.element("Key", &err.key);
        w.end();
    }

    if !routing_rules.is_empty() {
        w.start("RoutingRules");
        for rule in routing_rules {
            w.start("RoutingRule");
            if let Some(ref cond) = rule.condition {
                w.start("Condition");
                if let Some(ref code) = cond.http_error_code_returned_equals {
                    w.element("HttpErrorCodeReturnedEquals", code);
                }
                if let Some(ref prefix) = cond.key_prefix_equals {
                    w.element("KeyPrefixEquals", prefix);
                }
                w.end();
            }
            if let Some(ref redirect) = rule.redirect {
                w.start("Redirect");
                if let Some(ref host) = redirect.host_name {
                    w.element("HostName", host);
                }
                if let Some(ref code) = redirect.http_redirect_code {
                    w.element("HttpRedirectCode", code);
                }
                if let Some(ref protocol) = redirect.protocol {
                    w.element("Protocol", protocol);
                }
                if let Some(ref prefix) = redirect.replace_key_prefix_with {
                    w.element("ReplaceKeyPrefixWith", prefix);
                }
                if let Some(ref key) = redirect.replace_key_with {
                    w.element("ReplaceKeyWith", key);
                }
                w.end();
            }
            w.end();
        }
        w.end();
    }

    w.end();
    w.finish()
}
