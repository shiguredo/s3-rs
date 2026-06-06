//! GetBucketEncryption の XML パースエラー単体テスト
//!
//! 破損 XML に対して Error::InvalidResponse が返ることを検証する。

use shiguredo_s3::Error;
use shiguredo_s3::api::{GetBucketEncryptionFluentBuilder, S3Response};

/// XML が途中で切断されたケース
#[test]
fn test_malformed_xml_truncated() {
    let response = S3Response {
        status_code: 200,
        headers: vec![],
        body: b"<ServerSideEncryptionConfiguration><Rule><ApplyServerSideEncryptionByDefault><SSEAlgorithm>AES256</SSEAlgorithm></ApplyServerSideEncryptionByDefault></Ru".to_vec(),
    };
    let result = GetBucketEncryptionFluentBuilder::parse_response(&response);
    assert!(matches!(result, Err(Error::InvalidResponse(_))));
}

/// SSEAlgorithm が空文字列のケース（正常系）
#[test]
fn test_sse_algorithm_empty() {
    let response = S3Response {
        status_code: 200,
        headers: vec![],
        body: br#"<ServerSideEncryptionConfiguration><Rule><ApplyServerSideEncryptionByDefault><SSEAlgorithm></SSEAlgorithm></ApplyServerSideEncryptionByDefault></Rule></ServerSideEncryptionConfiguration>"#.to_vec(),
    };
    let result = GetBucketEncryptionFluentBuilder::parse_response(&response);
    assert!(result.is_ok());
}

/// non-UTF-8 バイナリデータのケース
#[test]
fn test_non_utf8_body() {
    // 有効な ASCII の後に非 UTF-8 バイトを含むデータ
    let response = S3Response {
        status_code: 200,
        headers: vec![],
        body: vec![0x3C, 0x52, 0x75, 0x6C, 0x65, 0x3E, 0xFF, 0xFE],
    };
    let result = GetBucketEncryptionFluentBuilder::parse_response(&response);
    assert!(matches!(result, Err(Error::InvalidResponse(_))));
}
