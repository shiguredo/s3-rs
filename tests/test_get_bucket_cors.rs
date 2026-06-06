//! GetBucketCors の XML パースエラー単体テスト
//!
//! 破損 XML や無効な値に対して Error::InvalidResponse が返ることを検証する。

use shiguredo_s3::Error;
use shiguredo_s3::api::{GetBucketCorsFluentBuilder, S3Response};

/// XML が途中で切断されたケース
#[test]
fn test_malformed_xml_truncated() {
    let response = S3Response {
        status_code: 200,
        headers: vec![],
        body: b"<CORSConfiguration><CORSRule><AllowedOrigin>*</AllowedOrigin><AllowedMethod>GET</AllowedMethod></CORSRu".to_vec(),
    };
    let result = GetBucketCorsFluentBuilder::parse_response(&response);
    assert!(matches!(result, Err(Error::InvalidResponse(_))));
}

/// MaxAgeSeconds に非数値文字列が入ったケース
#[test]
fn test_non_numeric_max_age() {
    let response = S3Response {
        status_code: 200,
        headers: vec![],
        body: br#"<CORSConfiguration><CORSRule><AllowedOrigin>*</AllowedOrigin><AllowedMethod>GET</AllowedMethod><MaxAgeSeconds>abc</MaxAgeSeconds></CORSRule></CORSConfiguration>"#.to_vec(),
    };
    let result = GetBucketCorsFluentBuilder::parse_response(&response);
    assert!(matches!(result, Err(Error::InvalidResponse(_))));
}

/// MaxAgeSeconds に i32 範囲オーバーフロー値が入ったケース
#[test]
fn test_overflow_max_age() {
    let response = S3Response {
        status_code: 200,
        headers: vec![],
        body: br#"<CORSConfiguration><CORSRule><AllowedOrigin>*</AllowedOrigin><AllowedMethod>GET</AllowedMethod><MaxAgeSeconds>99999999999</MaxAgeSeconds></CORSRule></CORSConfiguration>"#.to_vec(),
    };
    let result = GetBucketCorsFluentBuilder::parse_response(&response);
    assert!(matches!(result, Err(Error::InvalidResponse(_))));
}
