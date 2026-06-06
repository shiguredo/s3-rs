//! GetBucketWebsite の XML パースエラー単体テスト
//!
//! 破損 XML に対して Error::InvalidResponse が返ることを検証する。

use shiguredo_s3::Error;
use shiguredo_s3::api::{GetBucketWebsiteFluentBuilder, S3Response};

/// XML が途中で切断されたケース
#[test]
fn test_malformed_xml_truncated() {
    let response = S3Response {
        status_code: 200,
        headers: vec![],
        body: b"<WebsiteConfiguration><IndexDocument><Suffix>index.html</Suf".to_vec(),
    };
    let result = GetBucketWebsiteFluentBuilder::parse_response(&response);
    assert!(matches!(result, Err(Error::InvalidResponse(_))));
}
