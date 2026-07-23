# PutBucketNotificationConfiguration で Content-MD5 ヘッダーが欠落している問題を修正する

- Priority: High
- Created: 2026-07-12
- Completed: 2026-07-23
- Model: Composer 2.5 Fast
- Polished: 2026-07-23
- Branch: feature/fix-put-bucket-notification-configuration-content-md5

## AWS S3 API Reference

- <https://docs.aws.amazon.com/AmazonS3/latest/API/API_PutBucketNotificationConfiguration.html>

> Enables notifications of specified events for a bucket.

## 目的

`put_bucket_notification_configuration.rs` でのみ、XML ボディ送信時に `Content-MD5` ヘッダーが欠落している問題を修正する。

## 優先度根拠

他の全 XML ボディ送信 API（DeleteObjects, PutBucketCors, PutBucketEncryption, PutBucketLifecycleConfiguration, PutBucketOwnershipControls, PutBucketPolicy, PutBucketTagging, PutBucketVersioning, PutBucketWebsite, PutPublicAccessBlock, PutObjectRetention, PutObjectLegalHold, PutObjectLockConfiguration, PutObjectTagging）では `Content-MD5` が付与されている。このファイルだけ欠落しているのは他 API テンプレートからのコピペ漏れであり、一貫性の問題。S3 互換ストアによっては Content-MD5 の有無で挙動が変わる可能性がある。

## 現状

`src/api/put_bucket_notification_configuration.rs:82-88`:

```rust
let mut extra_headers: Vec<(&str, &str)> = vec![("content-type", "application/xml")];
```

`base64_md5(xml_body)` の計算と `content-md5` ヘッダーの追加が欠落している。

## 設計方針

- `put_bucket_notification_configuration.rs` の `build_request` に、他 API（`put_bucket_cors.rs` 等）と同様に `base64_md5` の計算と `content-md5` ヘッダーの追加を行う
- `super::base64_md5` の import を追加する
- 当該 API には `presigned` メソッドが存在しないため修正不要

## 完了条件

- `build_request()` で `Content-MD5` ヘッダーが付与されること
- 既存のテストが全て通過すること
- `CHANGES.md` の `## develop` に `[FIX]` エントリを記載すること

## 解決方法

1. `build_request()` 内で `let content_md5 = base64_md5(xml_body.as_bytes());` を計算し、extra_headers に `("content-md5", content_md5.as_str())` を追加する
2. import に `super::base64_md5` を追加する
3. CHANGES.md の `## develop` に `[FIX]` エントリを追加する
