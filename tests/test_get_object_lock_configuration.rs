//! GetObjectLockConfiguration の XML パースエラー単体テスト
//!
//! 破損 XML や無効な値に対して Error::InvalidResponse が返ることを検証する。

use shiguredo_s3::Error;
use shiguredo_s3::api::{GetObjectLockConfigurationFluentBuilder, S3Response};

/// XML が途中で切断されたケース
#[test]
fn test_malformed_xml_truncated() {
    let response = S3Response {
        status_code: 200,
        headers: vec![],
        body: b"<ObjectLockConfiguration><ObjectLockEnabled>Enabled</ObjectLockEnabled><Rule><DefaultRetention><Mode>GOVERNANCE</Mode><Days>30</Da".to_vec(),
    };
    let result = GetObjectLockConfigurationFluentBuilder::parse_response(&response);
    assert!(matches!(result, Err(Error::InvalidResponse(_))));
}

/// Days に非数値が入ったケース
#[test]
fn test_non_numeric_days() {
    let response = S3Response {
        status_code: 200,
        headers: vec![],
        body: br#"<ObjectLockConfiguration><ObjectLockEnabled>Enabled</ObjectLockEnabled><Rule><DefaultRetention><Mode>GOVERNANCE</Mode><Days>abc</Days></DefaultRetention></Rule></ObjectLockConfiguration>"#.to_vec(),
    };
    let result = GetObjectLockConfigurationFluentBuilder::parse_response(&response);
    assert!(matches!(result, Err(Error::InvalidResponse(_))));
}

/// Years に非数値が入ったケース
#[test]
fn test_non_numeric_years() {
    let response = S3Response {
        status_code: 200,
        headers: vec![],
        body: br#"<ObjectLockConfiguration><ObjectLockEnabled>Enabled</ObjectLockEnabled><Rule><DefaultRetention><Mode>GOVERNANCE</Mode><Years>xyz</Years></DefaultRetention></Rule></ObjectLockConfiguration>"#.to_vec(),
    };
    let result = GetObjectLockConfigurationFluentBuilder::parse_response(&response);
    assert!(matches!(result, Err(Error::InvalidResponse(_))));
}
