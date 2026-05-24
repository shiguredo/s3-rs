# GetBucket 系 API で XML パースエラーを黙殺する

- Priority: High
- Created: 2026-05-25
- Model: Composer 2.5
- Branch: feature/fix-xml-parse-error-swallowing

## 目的

GetBucket 系 API の XML パーサーが `Err(_)` を握り潰し、破損 XML を正常レスポンスとして返す。利用者が不完全な設定データを正しい値と誤認するため、早期に `Error::InvalidResponse` を返す必要がある。

## 優先度根拠

本番環境で XML 破損・途中切断が発生した場合、CORS / 暗号化 / ライフサイクル等の設定が欠落したまま `Ok` になる。データ欠落はサイレントで、設定ミスや移行失敗の原因になりうる。

## 現状

以下 6 ファイルで XML パースエラー時に部分結果を返すか、ループを中断するだけでエラーにしない。

| ファイル | 行 | 挙動 |
|----------|-----|------|
| `src/api/get_bucket_encryption.rs` | 140 | `Err(_) => return rules` |
| `src/api/get_bucket_lifecycle_configuration.rs` | 471 | `Err(_) => return rules` |
| `src/api/get_bucket_cors.rs` | 128 | `Err(_) => return rules` |
| `src/api/get_object_lock_configuration.rs` | 143 | `Err(_) => break` |
| `src/api/get_bucket_website.rs` | 246 | `Err(_) => break` |
| `src/api/get_bucket_notification_configuration.rs` | 275 | `Err(_) => break` |

## 設計方針

- XML パース `Err` は `Error::InvalidResponse` に変換して `parse_response` から返す
- 正常系の部分パース結果を返す経路は削除する
- `get_bucket_lifecycle_configuration.rs` の `Status` パース失敗時 `Enabled` フォールバック（233-235 行付近）も `InvalidResponse` に変更する

## AWS S3 API Reference

- GetBucketEncryption: <https://docs.aws.amazon.com/AmazonS3/latest/API/API_GetBucketEncryption.html>
- GetBucketLifecycleConfiguration: <https://docs.aws.amazon.com/AmazonS3/latest/API/API_GetBucketLifecycleConfiguration.html>
- GetBucketCors: <https://docs.aws.amazon.com/AmazonS3/latest/API/API_GetBucketCors.html>
- GetObjectLockConfiguration: <https://docs.aws.amazon.com/AmazonS3/latest/API/API_GetObjectLockConfiguration.html>
- GetBucketWebsite: <https://docs.aws.amazon.com/AmazonS3/latest/API/API_GetBucketWebsite.html>
- GetBucketNotificationConfiguration: <https://docs.aws.amazon.com/AmazonS3/latest/API/API_GetBucketNotificationConfiguration.html>

> For successful operations, Amazon S3 returns the XML response body.

## 完了条件

- 上記 6 API すべてで、不正 XML に対して `Error::InvalidResponse` を返す
- MinIO / RustFS 統合テストで正常系が引き続き通る
- `Status` フォールバックが除去され、パース失敗はエラーになる

## 解決方法

1. 各 `extract_*` 関数を `Result<_, Error>` 化するか、パースループ内で `Err` を `InvalidResponse` に変換する
2. 単体テストまたは統合テストで malformed XML のエラーパスを検証する
