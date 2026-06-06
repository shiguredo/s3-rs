//! GetBucketNotificationConfiguration の XML パースエラー単体テスト
//!
//! 破損 XML に対して Error::InvalidResponse が返ることを検証する。

use shiguredo_s3::Error;
use shiguredo_s3::api::{GetBucketNotificationConfigurationFluentBuilder, S3Response};

/// XML が途中で切断されたケース
#[test]
fn test_malformed_xml_truncated() {
    let response = S3Response {
        status_code: 200,
        headers: vec![],
        body: b"<NotificationConfiguration><TopicConfiguration><Id>test</Id><Topic>arn:aws:sns:us-east-1:123456789012:MyTopic</Topic><Event>s3:ObjectCreated:*</Eve".to_vec(),
    };
    let result = GetBucketNotificationConfigurationFluentBuilder::parse_response(&response);
    assert!(matches!(result, Err(Error::InvalidResponse(_))));
}
