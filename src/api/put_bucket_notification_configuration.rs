//! PutBucketNotificationConfiguration API
//!
//! バケットのイベント通知設定を登録・更新する。
//! 通知を無効化するには空の NotificationConfiguration を PUT する。
//!
//! <https://docs.aws.amazon.com/AmazonS3/latest/API/API_PutBucketNotificationConfiguration.html>

use crate::client::Client;
use crate::error::Error;
use crate::types::{
    LambdaFunctionConfiguration, PutBucketNotificationConfigurationOutput, QueueConfiguration,
    TopicConfiguration,
};

use super::{S3Request, base64_md5, build_signed_request, parse_error_response, required};

pub struct PutBucketNotificationConfigurationFluentBuilder<'a> {
    client: &'a Client,
    bucket: Option<String>,
    topic_configurations: Vec<TopicConfiguration>,
    queue_configurations: Vec<QueueConfiguration>,
    lambda_function_configurations: Vec<LambdaFunctionConfiguration>,
    event_bridge_enabled: bool,
    skip_destination_validation: Option<bool>,
}

impl<'a> PutBucketNotificationConfigurationFluentBuilder<'a> {
    pub(crate) fn new(client: &'a Client) -> Self {
        Self {
            client,
            bucket: None,
            topic_configurations: Vec::new(),
            queue_configurations: Vec::new(),
            lambda_function_configurations: Vec::new(),
            event_bridge_enabled: false,
            skip_destination_validation: None,
        }
    }

    pub fn bucket(mut self, bucket: impl Into<String>) -> Self {
        self.bucket = Some(bucket.into());
        self
    }

    /// SNS トピック通知設定を追加する
    pub fn topic_configuration(mut self, config: TopicConfiguration) -> Self {
        self.topic_configurations.push(config);
        self
    }

    /// SQS キュー通知設定を追加する
    pub fn queue_configuration(mut self, config: QueueConfiguration) -> Self {
        self.queue_configurations.push(config);
        self
    }

    /// Lambda 関数通知設定を追加する
    pub fn lambda_function_configuration(mut self, config: LambdaFunctionConfiguration) -> Self {
        self.lambda_function_configurations.push(config);
        self
    }

    /// EventBridge 通知を有効化する
    pub fn event_bridge_enabled(mut self, enabled: bool) -> Self {
        self.event_bridge_enabled = enabled;
        self
    }

    /// 宛先の到達可能性検証をスキップする
    pub fn skip_destination_validation(mut self, skip: bool) -> Self {
        self.skip_destination_validation = Some(skip);
        self
    }

    pub fn build_request(&self, now: std::time::SystemTime) -> Result<S3Request, Error> {
        let bucket = required(self.bucket.as_deref(), "bucket")?;

        let xml_body = build_notification_xml(
            &self.topic_configurations,
            &self.queue_configurations,
            &self.lambda_function_configurations,
            self.event_bridge_enabled,
        )?;
        let content_md5 = base64_md5(xml_body.as_bytes());
        let mut extra_headers: Vec<(&str, &str)> = vec![
            ("content-type", "application/xml"),
            ("content-md5", content_md5.as_str()),
        ];

        if self.skip_destination_validation == Some(true) {
            extra_headers.push(("x-amz-skip-destination-validation", "true"));
        }

        build_signed_request(
            &self.client.config_ref(),
            "PUT",
            bucket,
            "",
            &extra_headers,
            xml_body.as_bytes(),
            Some(&[("notification", "")]),
            now,
        )
    }

    pub fn parse_response(
        response: &super::S3Response,
    ) -> Result<PutBucketNotificationConfigurationOutput, Error> {
        if !response.is_success() {
            return Err(parse_error_response(response));
        }
        Ok(PutBucketNotificationConfigurationOutput {})
    }
}

fn write_filter(
    w: &mut crate::xml::XmlWriter,
    filter: &crate::types::NotificationConfigurationFilter,
) -> Result<(), Error> {
    w.start("Filter");
    if let Some(ref key) = filter.key {
        w.start("S3Key");
        for rule in &key.filter_rules {
            w.start("FilterRule");
            w.element("Name", &rule.name)?;
            w.element("Value", &rule.value)?;
            w.end();
        }
        w.end();
    }
    w.end();
    Ok(())
}

fn build_notification_xml(
    topics: &[TopicConfiguration],
    queues: &[QueueConfiguration],
    lambdas: &[LambdaFunctionConfiguration],
    event_bridge: bool,
) -> Result<String, Error> {
    let mut w = crate::xml::XmlWriter::new();
    w.start_ns("NotificationConfiguration", crate::xml::S3_NS);

    for topic in topics {
        w.start("TopicConfiguration");
        if let Some(ref id) = topic.id {
            w.element("Id", id)?;
        }
        w.element("Topic", &topic.topic_arn)?;
        for event in &topic.events {
            w.element("Event", event)?;
        }
        if let Some(ref filter) = topic.filter {
            write_filter(&mut w, filter)?;
        }
        w.end();
    }

    for queue in queues {
        w.start("QueueConfiguration");
        if let Some(ref id) = queue.id {
            w.element("Id", id)?;
        }
        w.element("Queue", &queue.queue_arn)?;
        for event in &queue.events {
            w.element("Event", event)?;
        }
        if let Some(ref filter) = queue.filter {
            write_filter(&mut w, filter)?;
        }
        w.end();
    }

    for lambda in lambdas {
        w.start("CloudFunctionConfiguration");
        if let Some(ref id) = lambda.id {
            w.element("Id", id)?;
        }
        w.element("CloudFunction", &lambda.lambda_function_arn)?;
        for event in &lambda.events {
            w.element("Event", event)?;
        }
        if let Some(ref filter) = lambda.filter {
            write_filter(&mut w, filter)?;
        }
        w.end();
    }

    if event_bridge {
        w.start("EventBridgeConfiguration");
        w.end();
    }

    w.end();
    Ok(w.finish())
}
