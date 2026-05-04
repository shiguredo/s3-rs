//! GetBucketNotificationConfiguration API
//!
//! バケットのイベント通知設定を取得する。
//!
//! <https://docs.aws.amazon.com/AmazonS3/latest/API/API_GetBucketNotificationConfiguration.html>

use crate::client::Client;
use crate::error::Error;
use crate::types::{
    EventBridgeConfiguration, FilterRule, GetBucketNotificationConfigurationOutput,
    LambdaFunctionConfiguration, NotificationConfigurationFilter, QueueConfiguration, S3KeyFilter,
    TopicConfiguration,
};

use super::{S3Request, build_signed_request, parse_error_response, required};

pub struct GetBucketNotificationConfigurationFluentBuilder<'a> {
    client: &'a Client,
    bucket: Option<String>,
}

impl<'a> GetBucketNotificationConfigurationFluentBuilder<'a> {
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
            Some(&[("notification", "")]),
        ))
    }

    pub fn parse_response(
        response: &super::S3Response,
    ) -> Result<GetBucketNotificationConfigurationOutput, Error> {
        if !response.is_success() {
            return Err(parse_error_response(response));
        }

        let body_text = super::xml_body_text(&response.body)?;
        Ok(parse_notification_configuration(body_text))
    }
}

/// NotificationConfiguration XML をパースする
///
/// TopicConfiguration, QueueConfiguration, CloudFunctionConfiguration,
/// EventBridgeConfiguration の各要素内にネストされた Event, Filter > S3Key > FilterRule が
/// 複数出現するため EventReader で直接パースする。
fn parse_notification_configuration(text: &str) -> GetBucketNotificationConfigurationOutput {
    use xml::reader::{EventReader, XmlEvent};

    let reader = EventReader::from_str(text);

    let mut topic_configs = Vec::new();
    let mut queue_configs = Vec::new();
    let mut lambda_configs = Vec::new();
    let mut event_bridge: Option<EventBridgeConfiguration> = None;

    // パーサー状態
    #[derive(PartialEq)]
    enum Context {
        Root,
        Topic,
        Queue,
        Lambda,
        Filter,
        S3Key,
        FilterRule,
    }

    let mut ctx = Context::Root;
    let mut parent_type = 0u8; // 1=Topic, 2=Queue, 3=Lambda
    let mut current_tag: Option<String> = None;
    let mut current_text = String::new();

    // 共通フィールド
    let mut id: Option<String> = None;
    let mut arn = String::new();
    let mut events: Vec<String> = Vec::new();
    let mut filter_rules: Vec<FilterRule> = Vec::new();
    let mut has_filter = false;
    let mut rule_name = String::new();
    let mut rule_value = String::new();

    for event in reader {
        match event {
            Ok(XmlEvent::StartElement { name, .. }) => {
                let tag = &name.local_name;
                match ctx {
                    Context::Root => match tag.as_str() {
                        "TopicConfiguration" => {
                            ctx = Context::Topic;
                            parent_type = 1;
                            id = None;
                            arn.clear();
                            events.clear();
                            filter_rules.clear();
                            has_filter = false;
                        }
                        "QueueConfiguration" => {
                            ctx = Context::Queue;
                            parent_type = 2;
                            id = None;
                            arn.clear();
                            events.clear();
                            filter_rules.clear();
                            has_filter = false;
                        }
                        "CloudFunctionConfiguration" => {
                            ctx = Context::Lambda;
                            parent_type = 3;
                            id = None;
                            arn.clear();
                            events.clear();
                            filter_rules.clear();
                            has_filter = false;
                        }
                        "EventBridgeConfiguration" => {
                            event_bridge = Some(EventBridgeConfiguration {});
                        }
                        _ => {}
                    },
                    Context::Topic | Context::Queue | Context::Lambda if tag == "Filter" => {
                        ctx = Context::Filter;
                        has_filter = true;
                    }
                    Context::Filter if tag == "S3Key" => {
                        ctx = Context::S3Key;
                    }
                    Context::S3Key if tag == "FilterRule" => {
                        ctx = Context::FilterRule;
                        rule_name.clear();
                        rule_value.clear();
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
                match ctx {
                    Context::FilterRule if tag == "FilterRule" => {
                        filter_rules.push(FilterRule {
                            name: rule_name.clone(),
                            value: rule_value.clone(),
                        });
                        ctx = Context::S3Key;
                    }
                    Context::FilterRule => {
                        if let Some(ref ct) = current_tag
                            && *ct == *tag
                        {
                            match ct.as_str() {
                                "Name" => rule_name = current_text.clone(),
                                "Value" => rule_value = current_text.clone(),
                                _ => {}
                            }
                        }
                        current_tag = None;
                    }
                    Context::S3Key if tag == "S3Key" => {
                        ctx = Context::Filter;
                    }
                    Context::Filter if tag == "Filter" => {
                        ctx = match parent_type {
                            1 => Context::Topic,
                            2 => Context::Queue,
                            _ => Context::Lambda,
                        };
                    }
                    Context::Topic if tag == "TopicConfiguration" => {
                        let filter = if has_filter {
                            Some(NotificationConfigurationFilter {
                                key: if filter_rules.is_empty() {
                                    None
                                } else {
                                    Some(S3KeyFilter {
                                        filter_rules: filter_rules.clone(),
                                    })
                                },
                            })
                        } else {
                            None
                        };
                        topic_configs.push(TopicConfiguration {
                            id: id.take(),
                            topic_arn: arn.clone(),
                            events: events.clone(),
                            filter,
                        });
                        ctx = Context::Root;
                    }
                    Context::Queue if tag == "QueueConfiguration" => {
                        let filter = if has_filter {
                            Some(NotificationConfigurationFilter {
                                key: if filter_rules.is_empty() {
                                    None
                                } else {
                                    Some(S3KeyFilter {
                                        filter_rules: filter_rules.clone(),
                                    })
                                },
                            })
                        } else {
                            None
                        };
                        queue_configs.push(QueueConfiguration {
                            id: id.take(),
                            queue_arn: arn.clone(),
                            events: events.clone(),
                            filter,
                        });
                        ctx = Context::Root;
                    }
                    Context::Lambda if tag == "CloudFunctionConfiguration" => {
                        let filter = if has_filter {
                            Some(NotificationConfigurationFilter {
                                key: if filter_rules.is_empty() {
                                    None
                                } else {
                                    Some(S3KeyFilter {
                                        filter_rules: filter_rules.clone(),
                                    })
                                },
                            })
                        } else {
                            None
                        };
                        lambda_configs.push(LambdaFunctionConfiguration {
                            id: id.take(),
                            lambda_function_arn: arn.clone(),
                            events: events.clone(),
                            filter,
                        });
                        ctx = Context::Root;
                    }
                    Context::Topic | Context::Queue | Context::Lambda => {
                        if let Some(ref ct) = current_tag
                            && *ct == *tag
                        {
                            match ct.as_str() {
                                "Id" => id = Some(current_text.clone()),
                                "Topic" | "Queue" | "CloudFunction" => arn = current_text.clone(),
                                "Event" => events.push(current_text.clone()),
                                _ => {}
                            }
                        }
                        current_tag = None;
                    }
                    _ => {}
                }
            }
            Err(_) => break,
            _ => {}
        }
    }

    GetBucketNotificationConfigurationOutput {
        topic_configurations: topic_configs,
        queue_configurations: queue_configs,
        lambda_function_configurations: lambda_configs,
        event_bridge_configuration: event_bridge,
    }
}
