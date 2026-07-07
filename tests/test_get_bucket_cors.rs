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

/// ID 要素のパース
#[test]
fn test_id_parse() {
    let response = S3Response {
        status_code: 200,
        headers: vec![],
        body: br#"<CORSConfiguration><CORSRule><ID>my-rule-id</ID><AllowedOrigin>*</AllowedOrigin><AllowedMethod>GET</AllowedMethod></CORSRule></CORSConfiguration>"#.to_vec(),
    };
    let result = GetBucketCorsFluentBuilder::parse_response(&response);
    let output = result.expect("パース成功");
    let rules = output.cors_rules.expect("CORS ルールあり");
    assert_eq!(rules.len(), 1);
    assert_eq!(rules[0].id.as_deref(), Some("my-rule-id"));
}

/// ID なしルールの既存挙動（後方互換）
#[test]
fn test_id_none() {
    let response = S3Response {
        status_code: 200,
        headers: vec![],
        body: br#"<CORSConfiguration><CORSRule><AllowedOrigin>*</AllowedOrigin><AllowedMethod>GET</AllowedMethod></CORSRule></CORSConfiguration>"#.to_vec(),
    };
    let result = GetBucketCorsFluentBuilder::parse_response(&response);
    let output = result.expect("パース成功");
    let rules = output.cors_rules.expect("CORS ルールあり");
    assert_eq!(rules.len(), 1);
    assert_eq!(rules[0].id, None);
}

/// CorsRuleBuilder::id で 255 文字超えは InvalidInput になる
#[test]
fn test_cors_rule_id_too_long() {
    use shiguredo_s3::Error;
    use shiguredo_s3::types::CorsRule;

    let long_id = "a".repeat(256);
    let result = CorsRule::builder()
        .id(long_id)
        .allowed_origins("*")
        .allowed_methods("GET")
        .build();
    assert!(matches!(result, Err(Error::InvalidInput(_))));
}

/// CorsRuleBuilder::id で空文字列は許容
#[test]
fn test_cors_rule_id_empty() {
    use shiguredo_s3::types::CorsRule;

    let rule = CorsRule::builder()
        .id("")
        .allowed_origins("*")
        .allowed_methods("GET")
        .build()
        .expect("build should succeed");
    assert_eq!(rule.id.as_deref(), Some(""));
}

/// CorsRuleBuilder::build で allowed_methods が空の場合は InvalidInput になる
#[test]
fn test_cors_rule_empty_allowed_methods() {
    use shiguredo_s3::Error;
    use shiguredo_s3::types::CorsRule;

    let result = CorsRule::builder().allowed_origins("*").build();
    assert!(matches!(result, Err(Error::InvalidInput(_))));
}

/// CorsRuleBuilder::build で allowed_origins が空の場合は InvalidInput になる
#[test]
fn test_cors_rule_empty_allowed_origins() {
    use shiguredo_s3::Error;
    use shiguredo_s3::types::CorsRule;

    let result = CorsRule::builder().allowed_methods("GET").build();
    assert!(matches!(result, Err(Error::InvalidInput(_))));
}
