# PutBucketNotificationConfiguration で Content-MD5 ヘッダーが欠落している問題を修正する

- Priority: High
- Created: 2026-07-12
- Model: Composer 2.5 Fast
- Branch: feature/fix-put-bucket-notification-configuration-content-md5

## 目的

`put_bucket_notification_configuration.rs` でのみ、XML ボディ送信時に `Content-MD5` ヘッダーが欠落している問題を修正する。

## 優先度根拠

他の全 XML PUT API（DeleteObjects, PutBucketCors, PutBucketEncryption, PutBucketLifecycleConfiguration, PutBucketOwnershipControls, PutBucketPolicy, PutBucketTagging, PutBucketVersioning, PutBucketWebsite, PutPublicAccessBlock, PutObjectRetention, PutObjectLegalHold, PutObjectLockConfiguration）では `Content-MD5` が付与されている。このファイルだけ欠落しているのは他 API テンプレートからのコピペ漏れであり、一貫性の問題。S3 互換ストアによっては Content-MD5 の有無で挙動が変わる可能性がある。

## 現状

`src/api/put_bucket_notification_configuration.rs:82-88`:

```rust
let mut extra_headers: Vec<(&str, &str)> = vec![("content-type", "application/xml")];
```

`base64_md5(xml_body)` の計算と `content-md5` ヘッダーの追加が欠落している。

## 設計方針

他の全 XML PUT API と同様に `Content-MD5` ヘッダーを追加する。

## 完了条件

- `build_request()` で `Content-MD5` ヘッダーが付与されること
- 既存のテストが全て通過すること

## 解決方法

1. `build_request()` 内で `base64_md5(xml_body)` を計算し `content-md5` ヘッダーを追加する
2. `presigned()` にも同様の修正を適用する
3. CHANGES.md の `## develop` に `[FIX]` エントリを追加する
