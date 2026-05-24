# PutBucket / PutObject 設定 API の空 XML・必須フィールド未検証

- Priority: High
- Created: 2026-05-25
- Model: Composer 2.5
- Branch: feature/fix-put-config-validation

## 目的

設定系 PUT API が空の XML や必須フィールド未設定のままリクエストを構築できる。クライアント側で早期に `InvalidInput` を返し、S3 側 `MalformedXML` への依存を減らす。

## 優先度根拠

空 `<CORSConfiguration/>` 等は S3 サーバーで拒否されるが、Sans I/O ライブラリとして builder 段階で失敗原因を明確にすべき。aws-sdk-rust も必須構造体の検証を行う。

## 現状

| API | 問題 |
|-----|------|
| `PutBucketEncryption` | `rules` 空で XML 構築 |
| `PutBucketCors` | `cors_rules` 空で XML 構築 |
| `PutBucketLifecycleConfiguration` | `rules` 空で XML 構築 |
| `PutBucketOwnershipControls` | `rules` 空で XML 構築 |
| `PutObjectRetention` | `mode` / `retain_until_date` 未設定で空 XML |
| `PutObjectLockConfiguration` | 未設定で空 XML |
| `PutBucketWebsite` | 全未指定で空 XML |
| `PutObjectLegalHold` | `legal_hold_status` 未指定時 `unwrap_or("ON")` |
| `ServerSideEncryptionByDefaultBuilder` | `sse_algorithm` 未設定で空文字列 |
| `DeleteBuilder` | `objects` 空で `build()` 可能 |

相互排他未検証:

- `PutBucketWebsite`: `redirect_all_requests_to` と他設定の同時指定
- `PutBucketLifecycleConfiguration`: Filter / Expiration の矛盾する組み合わせ
- `PutObjectLockConfiguration`: DefaultRetention の `days` と `years` 同時指定

## 設計方針

- 各 `build_request` で必須フィールド・空コレクションを `Error::InvalidInput` にする
- `ServerSideEncryptionByDefaultBuilder::build` を `Result<_, Error>` 化するか、呼び出し側で検証
- `PutObjectLegalHold` の暗黙 `"ON"` デフォルトを廃止し必須化
- aws-sdk-rust の必須フィールド定義を参照する

## AWS S3 API Reference

- PutBucketEncryption: <https://docs.aws.amazon.com/AmazonS3/latest/API/API_PutBucketEncryption.html>
- PutBucketCors: <https://docs.aws.amazon.com/AmazonS3/latest/API/API_PutBucketCors.html>
- PutBucketLifecycleConfiguration: <https://docs.aws.amazon.com/AmazonS3/latest/API/API_PutBucketLifecycleConfiguration.html>
- PutObjectRetention: <https://docs.aws.amazon.com/AmazonS3/latest/API/API_PutObjectRetention.html>
- PutObjectLegalHold: <https://docs.aws.amazon.com/AmazonS3/latest/API/API_PutObjectLegalHold.html>

> ServerSideEncryptionConfiguration is required.

## 完了条件

- 上記 API で空 XML / 必須未設定が builder 段階でエラーになる
- 正常系の統合テストが通る
- `PutObjectLegalHold` が未指定時に暗黙 ON にならない

## 解決方法

1. 各 `build_request` にバリデーションを追加
2. `types.rs` の Builder `build` で必須チェック
3. 単体テストでエラーパスを検証
