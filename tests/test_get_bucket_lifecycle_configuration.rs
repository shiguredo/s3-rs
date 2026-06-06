//! GetBucketLifecycleConfiguration の XML パースエラー単体テスト
//!
//! 破損 XML や無効な値に対して Error::InvalidResponse が返ることを検証する。

use shiguredo_s3::Error;
use shiguredo_s3::api::{GetBucketLifecycleConfigurationFluentBuilder, S3Response};

/// XML が途中で切断されたケース
#[test]
fn test_malformed_xml_truncated() {
    let response = S3Response {
        status_code: 200,
        headers: vec![],
        body: b"<LifecycleConfiguration><Rule><ID>test</ID><Status>Enabled</Status></Ru".to_vec(),
    };
    let result = GetBucketLifecycleConfigurationFluentBuilder::parse_response(&response);
    assert!(matches!(result, Err(Error::InvalidResponse(_))));
}

/// Status に無効値が入ったケース
#[test]
fn test_invalid_status() {
    let response = S3Response {
        status_code: 200,
        headers: vec![],
        body: br#"<LifecycleConfiguration><Rule><Status>Unknown</Status></Rule></LifecycleConfiguration>"#.to_vec(),
    };
    let result = GetBucketLifecycleConfigurationFluentBuilder::parse_response(&response);
    assert!(matches!(result, Err(Error::InvalidResponse(_))));
}

/// Days に非数値が入ったケース
#[test]
fn test_non_numeric_days_in_expiration() {
    let response = S3Response {
        status_code: 200,
        headers: vec![],
        body: br#"<LifecycleConfiguration><Rule><Status>Enabled</Status><Expiration><Days>abc</Days></Expiration></Rule></LifecycleConfiguration>"#.to_vec(),
    };
    let result = GetBucketLifecycleConfigurationFluentBuilder::parse_response(&response);
    assert!(matches!(result, Err(Error::InvalidResponse(_))));
}

/// ObjectSizeGreaterThan に非数値が入ったケース
#[test]
fn test_non_numeric_object_size() {
    let response = S3Response {
        status_code: 200,
        headers: vec![],
        body: br#"<LifecycleConfiguration><Rule><Status>Enabled</Status><Filter><ObjectSizeGreaterThan>xyz</ObjectSizeGreaterThan></Filter></Rule></LifecycleConfiguration>"#.to_vec(),
    };
    let result = GetBucketLifecycleConfigurationFluentBuilder::parse_response(&response);
    assert!(matches!(result, Err(Error::InvalidResponse(_))));
}
