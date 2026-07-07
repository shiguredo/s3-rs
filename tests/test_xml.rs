//! XML 生成の入力検証テスト
//!
//! XmlWriter が XML 1.0 で禁止された制御文字を検出し、
//! `Error::InvalidInput` を返すことを検証する。

use shiguredo_s3::types::{Delete, ObjectIdentifier, Tag, Tagging};
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

/// DeleteObjects のオブジェクトキーに制御文字が含まれる場合は InvalidInput になる
#[test]
fn test_delete_objects_rejects_control_char_in_key() {
    let client = test_client();
    let now = SystemTime::UNIX_EPOCH + Duration::from_secs(1_700_000_000);

    let delete = Delete::builder()
        .objects(ObjectIdentifier {
            key: "key\x01with\x02control".to_string(),
            version_id: None,
            e_tag: None,
            last_modified_time: None,
            size: None,
        })
        .build();

    let result = client
        .delete_objects()
        .bucket("test-bucket")
        .delete(delete)
        .build_request(now);

    assert!(matches!(result, Err(Error::InvalidInput(_))));
}

/// PutObjectTagging のタグキーに制御文字が含まれる場合は InvalidInput になる
#[test]
fn test_put_object_tagging_rejects_control_char_in_tag_key() {
    let client = test_client();
    let now = SystemTime::UNIX_EPOCH + Duration::from_secs(1_700_000_000);

    let tagging = Tagging::builder()
        .tag_set(Tag {
            key: "bad\x03key".to_string(),
            value: "value".to_string(),
        })
        .build();

    let result = client
        .put_object_tagging()
        .bucket("test-bucket")
        .key("test-key")
        .tagging(tagging)
        .build_request(now);

    assert!(matches!(result, Err(Error::InvalidInput(_))));
}

/// PutObjectTagging のタグ値に制御文字が含まれる場合は InvalidInput になる
#[test]
fn test_put_object_tagging_rejects_control_char_in_tag_value() {
    let client = test_client();
    let now = SystemTime::UNIX_EPOCH + Duration::from_secs(1_700_000_000);

    let tagging = Tagging::builder()
        .tag_set(Tag {
            key: "key".to_string(),
            value: "bad\x04value".to_string(),
        })
        .build();

    let result = client
        .put_object_tagging()
        .bucket("test-bucket")
        .key("test-key")
        .tagging(tagging)
        .build_request(now);

    assert!(matches!(result, Err(Error::InvalidInput(_))));
}

/// 改行、タブ、キャリッジリターンは許可される
#[test]
fn test_xml_allows_whitespace_control_chars() {
    let client = test_client();
    let now = SystemTime::UNIX_EPOCH + Duration::from_secs(1_700_000_000);

    let tagging = Tagging::builder()
        .tag_set(Tag {
            key: "key\nwith\ttab".to_string(),
            value: "value\rwith\nnewline".to_string(),
        })
        .build();

    let result = client
        .put_object_tagging()
        .bucket("test-bucket")
        .key("test-key")
        .tagging(tagging)
        .build_request(now);

    assert!(result.is_ok());
}
