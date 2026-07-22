//! CompleteMultipartUpload の parse_response 単体テスト
//!
//! checksum フィールドが XML ボディから正しく読み取られることを検証する。
//! S3 仕様では CompleteMultipartUpload の checksum はレスポンスヘッダーではなく
//! XML ボディの `<CompleteMultipartUploadResult>` 内に含まれる。

use shiguredo_s3::api::{CompleteMultipartUploadFluentBuilder, S3Response};

/// XML ボディに checksum が含まれる場合、parse_response が checksum を Some で返す
#[test]
fn test_parse_response_checksums_from_xml_body() {
    let xml = r#"<CompleteMultipartUploadResult>
  <Location>https://example-bucket.s3.amazonaws.com/test-key</Location>
  <Bucket>example-bucket</Bucket>
  <Key>test-key</Key>
  <ETag>"3858f62230ac3c915f300c664312c11f-9"</ETag>
  <ChecksumCRC32>c2VydmljZQ==</ChecksumCRC32>
  <ChecksumCRC32C>c2VydmljZQ==</ChecksumCRC32C>
  <ChecksumCRC64NVME>c2VydmljZQ==</ChecksumCRC64NVME>
  <ChecksumSHA1>c2VydmljZQ==</ChecksumSHA1>
  <ChecksumSHA256>c2VydmljZQ==</ChecksumSHA256>
  <ChecksumType>COMPOSITE</ChecksumType>
</CompleteMultipartUploadResult>"#;

    let response = S3Response {
        status_code: 200,
        headers: vec![],
        body: xml.as_bytes().to_vec(),
    };
    let output =
        CompleteMultipartUploadFluentBuilder::parse_response(&response).expect("parse failed");

    // XML ボディから読み取った checksum が Some になる
    assert_eq!(output.checksum_crc32.as_deref(), Some("c2VydmljZQ=="));
    assert_eq!(output.checksum_crc32_c.as_deref(), Some("c2VydmljZQ=="));
    assert_eq!(output.checksum_crc64_nvme.as_deref(), Some("c2VydmljZQ=="));
    assert_eq!(output.checksum_sha1.as_deref(), Some("c2VydmljZQ=="));
    assert_eq!(output.checksum_sha256.as_deref(), Some("c2VydmljZQ=="));
    assert_eq!(output.checksum_type.as_deref(), Some("COMPOSITE"));

    // 基本フィールドも XML から読み取れる
    assert_eq!(output.bucket.as_deref(), Some("example-bucket"));
    assert_eq!(output.key.as_deref(), Some("test-key"));
    assert!(output.e_tag.is_some());
}

/// XML ボディに checksum が含まれない場合、parse_response が checksum を None で返す
#[test]
fn test_parse_response_no_checksums_in_xml_body() {
    let xml = r#"<CompleteMultipartUploadResult>
  <Location>https://example-bucket.s3.amazonaws.com/test-key</Location>
  <Bucket>example-bucket</Bucket>
  <Key>test-key</Key>
  <ETag>"3858f62230ac3c915f300c664312c11f-9"</ETag>
</CompleteMultipartUploadResult>"#;

    let response = S3Response {
        status_code: 200,
        headers: vec![],
        body: xml.as_bytes().to_vec(),
    };
    let output =
        CompleteMultipartUploadFluentBuilder::parse_response(&response).expect("parse failed");

    // checksum タグが無ければ None
    assert!(output.checksum_crc32.is_none());
    assert!(output.checksum_crc32_c.is_none());
    assert!(output.checksum_crc64_nvme.is_none());
    assert!(output.checksum_sha1.is_none());
    assert!(output.checksum_sha256.is_none());
    assert!(output.checksum_type.is_none());
}

/// レスポンスヘッダーに checksum があっても XML ボディに無ければ None を返す
///
/// 修正前はヘッダーから読んでいたため Some になっていたが、
/// S3 仕様では checksum は XML ボディに含まれるため、ヘッダーは無視する。
#[test]
fn test_parse_response_ignores_checksum_headers() {
    let xml = r#"<CompleteMultipartUploadResult>
  <Location>https://example-bucket.s3.amazonaws.com/test-key</Location>
  <Bucket>example-bucket</Bucket>
  <Key>test-key</Key>
  <ETag>"3858f62230ac3c915f300c664312c11f-9"</ETag>
</CompleteMultipartUploadResult>"#;

    let response = S3Response {
        status_code: 200,
        headers: vec![
            (
                "x-amz-checksum-crc32".to_string(),
                "header-value".to_string(),
            ),
            (
                "x-amz-checksum-sha256".to_string(),
                "header-value".to_string(),
            ),
        ],
        body: xml.as_bytes().to_vec(),
    };
    let output =
        CompleteMultipartUploadFluentBuilder::parse_response(&response).expect("parse failed");

    // ヘッダーに値があっても XML ボディに無ければ None
    assert!(output.checksum_crc32.is_none());
    assert!(output.checksum_sha256.is_none());
}

/// XML が途中で切断されたケースで InvalidResponse エラーが返る
#[test]
fn test_malformed_xml_truncated() {
    let response = S3Response {
        status_code: 200,
        headers: vec![],
        body: b"<CompleteMultipartUploadResult><Bucket>example-bucket</Bucket><Key>test-key"
            .to_vec(),
    };
    let result = CompleteMultipartUploadFluentBuilder::parse_response(&response);
    assert!(matches!(
        result,
        Err(shiguredo_s3::Error::InvalidResponse(_))
    ));
}

/// 200 OK でもボディに Error 要素が含まれる場合はエラーが返る
///
/// S3 は CompleteMultipartUpload で 200 OK でもボディにエラーを返すことがある。
#[test]
fn test_parse_response_body_error() {
    let xml = r#"<Error><Code>InternalError</Code><Message>We encountered an internal error. Please try again.</Message></Error>"#;
    let response = S3Response {
        status_code: 200,
        headers: vec![],
        body: xml.as_bytes().to_vec(),
    };
    let result = CompleteMultipartUploadFluentBuilder::parse_response(&response);
    assert!(result.is_err());
}
