//! HeadObject の単体テスト
//!
//! Presigned URL 生成と入力検証をサーバー不要で検証する。

use shiguredo_s3::types::ChecksumMode;
use shiguredo_s3::{Client, Config, Credentials, Error};
use std::time::{Duration, SystemTime};

/// テスト用の S3 クライアントを作成する
fn test_client() -> Client {
    let config = Config::builder()
        .region("ap-northeast-1")
        .credentials_provider(Credentials::new(
            "AKIAIOSFODNN7EXAMPLE",
            "wJalrXUtnFEMI/K7MDENG/bPxRfiCYEXAMPLEKEY",
            None,
            None,
            "test",
        ))
        .build()
        .expect("config build failed");
    Client::from_conf(config)
}

/// Presigned URL に条件ヘッダーが署名対象として含まれることを検証する
#[test]
fn test_presigned_includes_conditional_headers() {
    let client = test_client();
    let if_modified_since = SystemTime::UNIX_EPOCH + Duration::from_secs(1_700_000_000);
    let presigned = client
        .head_object()
        .bucket("test-bucket")
        .key("test-key")
        .range("bytes=0-999")
        .if_match("\"abc123\"")
        .if_none_match("\"def456\"")
        .if_modified_since(if_modified_since)
        .if_unmodified_since(if_modified_since)
        .checksum_mode(ChecksumMode::Enabled)
        .presigned(
            3600,
            SystemTime::UNIX_EPOCH + Duration::from_secs(1_700_000_000),
        )
        .expect("presigned URL generation failed");

    assert_eq!(presigned.method, "HEAD");

    // PresignedRequest.headers に全ての条件ヘッダーが含まれる
    let header_names: Vec<&str> = presigned.headers.iter().map(|(k, _)| k.as_str()).collect();
    assert!(header_names.contains(&"range"));
    assert!(header_names.contains(&"if-match"));
    assert!(header_names.contains(&"if-none-match"));
    assert!(header_names.contains(&"if-modified-since"));
    assert!(header_names.contains(&"if-unmodified-since"));
    assert!(header_names.contains(&"x-amz-checksum-mode"));

    // URL の X-Amz-SignedHeaders にも同じ header 名が含まれる
    let signed_headers = presigned
        .url
        .split('&')
        .find(|s| s.starts_with("X-Amz-SignedHeaders="))
        .expect("X-Amz-SignedHeaders not found");
    assert!(signed_headers.contains("range"));
    assert!(signed_headers.contains("if-match"));
    assert!(signed_headers.contains("if-none-match"));
    assert!(signed_headers.contains("if-modified-since"));
    assert!(signed_headers.contains("if-unmodified-since"));
    assert!(signed_headers.contains("x-amz-checksum-mode"));
}

/// Presigned URL の part_number が範囲外の場合は InvalidInput になる
#[test]
fn test_presigned_part_number_out_of_range() {
    let client = test_client();
    let now = SystemTime::UNIX_EPOCH + Duration::from_secs(1_700_000_000);

    let result = client
        .head_object()
        .bucket("test-bucket")
        .key("test-key")
        .part_number(0)
        .presigned(3600, now);
    assert!(matches!(result, Err(Error::InvalidInput(_))));

    let result = client
        .head_object()
        .bucket("test-bucket")
        .key("test-key")
        .part_number(10001)
        .presigned(3600, now);
    assert!(matches!(result, Err(Error::InvalidInput(_))));
}

/// Presigned URL の part_number が範囲内の場合は成功する
#[test]
fn test_presigned_part_number_in_range() {
    let client = test_client();
    let now = SystemTime::UNIX_EPOCH + Duration::from_secs(1_700_000_000);

    let presigned = client
        .head_object()
        .bucket("test-bucket")
        .key("test-key")
        .part_number(1)
        .presigned(3600, now)
        .expect("presigned URL generation failed");
    assert!(presigned.url.contains("partNumber=1"));

    let presigned = client
        .head_object()
        .bucket("test-bucket")
        .key("test-key")
        .part_number(10000)
        .presigned(3600, now)
        .expect("presigned URL generation failed");
    assert!(presigned.url.contains("partNumber=10000"));
}
