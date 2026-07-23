//! DeleteObjects の LastModifiedTime フォーマット単体テスト
//!
//! 条件付き削除の LastModifiedTime が IMF-fixdate (HTTP-date) 形式で
//! XML ボディにシリアライズされることを検証する。

use shiguredo_s3::types::{Delete, ObjectIdentifier};
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

/// LastModifiedTime が IMF-fixdate 形式で XML ボディに含まれることを検証する
#[test]
fn test_last_modified_time_imf_fixdate_format() {
    let client = test_client();
    // 2024-10-15 15:04:05 UTC
    let last_modified = SystemTime::UNIX_EPOCH + Duration::from_secs(1_729_004_645);

    let request = client
        .delete_objects()
        .bucket("test-bucket")
        .delete(Delete {
            objects: vec![ObjectIdentifier {
                key: "test-key".to_string(),
                version_id: None,
                e_tag: None,
                last_modified_time: Some(last_modified),
                size: None,
            }],
            quiet: None,
        })
        .build_request(SystemTime::UNIX_EPOCH + Duration::from_secs(1_729_004_645))
        .expect("build_request failed");

    let body = String::from_utf8(request.body).expect("body should be valid UTF-8");
    // IMF-fixdate 形式: "Tue, 15 Oct 2024 15:04:05 GMT"
    assert!(
        body.contains("<LastModifiedTime>Tue, 15 Oct 2024 15:04:05 GMT</LastModifiedTime>"),
        "LastModifiedTime should be in IMF-fixdate format, got: {body}"
    );
}

/// epoch 前の SystemTime が Error::InvalidInput で拒否されることを検証する
#[test]
fn test_last_modified_time_before_epoch_returns_error() {
    let client = test_client();
    // epoch 前の時刻
    let before_epoch = SystemTime::UNIX_EPOCH - Duration::from_secs(1);

    let result = client
        .delete_objects()
        .bucket("test-bucket")
        .delete(Delete {
            objects: vec![ObjectIdentifier {
                key: "test-key".to_string(),
                version_id: None,
                e_tag: None,
                last_modified_time: Some(before_epoch),
                size: None,
            }],
            quiet: None,
        })
        .build_request(SystemTime::UNIX_EPOCH + Duration::from_secs(1_729_004_645));

    assert!(
        matches!(result, Err(Error::InvalidInput(_))),
        "epoch 前の SystemTime は InvalidInput になるべき"
    );
}
