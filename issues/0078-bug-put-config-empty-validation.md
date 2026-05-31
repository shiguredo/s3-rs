# PutBucket / PutObject 設定 API の空 XML・必須フィールド未検証

- Priority: High
- Created: 2026-05-25
- Model: Composer 2.5
- Polished: 2026-05-31

## 目的

設定系 PUT API が空の XML や必須フィールド未設定のままリクエストを構築できる。クライアント側で早期に `InvalidInput` を返し、S3 側 `MalformedXML` への依存を減らす。aws-sdk-rust は必須構造体の検証を行うため、互換性のためにもクライアント側バリデーションを追加する。

## 優先度根拠

Sans I/O ライブラリとして builder 段階で失敗原因を明確にすることで、利用者のデバッグ効率が向上する。エラーメッセージが S3 サーバーの `MalformedXML` ではなく具体的なフィールド名を含む `InvalidInput` になる。

## 現状

| API | 問題 |
|-----|------|
| `PutBucketEncryption` | `rules` 空で XML 構築 |
| `PutBucketCors` | `cors_rules` 空で XML 構築 |
| `PutBucketLifecycleConfiguration` | `rules` 空で XML 構築 |
| `PutBucketOwnershipControls` | `rules` 空で XML 構築 |
| `PutObjectRetention` | `mode` / `retain_until_date` 未設定で空 XML |
| `PutObjectLockConfiguration` | `object_lock_configuration` が `None` で空 XML |

注: `PutBucketTagging` / `PutObjectTagging` は既に `build_request` で `required()` による検証が行われているため対象外。
| `PutBucketWebsite` | 全未指定で空 XML |
| `PutObjectLegalHold` | `legal_hold_status` 未指定時 `unwrap_or("ON")` |
| `ServerSideEncryptionByDefaultBuilder` | `sse_algorithm` 未設定で空文字列 |
| `DeleteBuilder` | `objects` 空で `build()` 可能（注: `build_request` 側で既に検証済み） |

相互排他未検証:

- `PutBucketWebsite`: `redirect_all_requests_to` と `IndexDocument` / `ErrorDocument` / `RoutingRules` の同時指定
- `PutBucketLifecycleConfiguration`: `Filter` 内の `Prefix` と `Tag` の矛盾する組み合わせ
- `PutObjectLockConfiguration`: `DefaultRetention` の `days` と `years` 同時指定

## 設計方針

### 必須フィールドのバリデーション

各 `build_request` で必須フィールド・空コレクションを `Error::InvalidInput` にする。対象:

- `PutBucketEncryption`: `rules` が空
- `PutBucketCors`: `cors_rules` が空
- `PutBucketLifecycleConfiguration`: `rules` が空
- `PutBucketOwnershipControls`: `rules` が空
- `PutObjectRetention`: `mode` と `retain_until_date` の両方が未設定
- `PutBucketWebsite`: `IndexDocument` / `ErrorDocument` / `RedirectAllRequestsTo` / `RoutingRules` の全てが未指定
- `PutObjectLegalHold`: `legal_hold_status` が未設定
- `PutObjectLockConfiguration`: `object_lock_configuration` が `None`

注: `PutObjectLockConfiguration` の `object_lock_enabled` は AWS S3 API Reference で Required: No のため、未設定チェックの対象外とする。

### `ServerSideEncryptionByDefaultBuilder::build` の扱い

`build()` を `Result<_, Error>` 化するか、呼び出し側 (`PutBucketEncryption::build_request`) で検証する。`build()` は現在 `Self` を返しており、`Result<_, Error>` への変更は後方互換のない変更となる。呼び出し側で検証する方針を優先する。具体的には、`PutBucketEncryption::build_request` で `sse_algorithm` が空文字列の場合に `Error::InvalidInput` を返す。

### `PutObjectLegalHold` の暗黙デフォルト廃止

`legal_hold_status` 未指定時に `"ON"` にフォールバックする現行挙動を廃止し、必須化する。これは後方互換のない変更であるため、`CHANGES.md` に `[CHANGE]` として記載する。

### `PutObjectRetention` の必須/任意

AWS S3 API Reference によると、`Retention` 要素は必須だが `Mode` と `RetainUntilDate` はそれぞれ任意。空の `Retention` 要素は S3 サーバーで拒否されるため、少なくともどちらか一方は必須とする。

### 相互排他の検証

`PutBucketWebsite` の `redirect_all_requests_to` と `IndexDocument` / `ErrorDocument` / `RoutingRules` の同時指定をエラーにする。他の 2 つ（`PutBucketLifecycleConfiguration` の Filter 矛盾、`PutObjectLockConfiguration` の days / years 同時指定）は S3 サーバーで適切にエラーが返るため、クライアント側検証は不要（現状維持）。判断基準: `PutBucketWebsite` の相互排他は S3 サーバーでエラーにならないケースがあるためクライアント側で検証する。

## AWS S3 API Reference

- PutBucketEncryption: <https://docs.aws.amazon.com/AmazonS3/latest/API/API_PutBucketEncryption.html>
  - > ServerSideEncryptionConfiguration is required. Rule is required.
- PutBucketCors: <https://docs.aws.amazon.com/AmazonS3/latest/API/API_PutBucketCors.html>
  - > CORSConfiguration is required. CORSRule is required.
- PutBucketLifecycleConfiguration: <https://docs.aws.amazon.com/AmazonS3/latest/API/API_PutBucketLifecycleConfiguration.html>
  - > LifecycleConfiguration is required. Rule is required.
- PutBucketOwnershipControls: <https://docs.aws.amazon.com/AmazonS3/latest/API/API_PutBucketOwnershipControls.html>
  - > OwnershipControls is required.
- PutObjectRetention: <https://docs.aws.amazon.com/AmazonS3/latest/API/API_PutObjectRetention.html>
  - > Retention is required. Mode: Required: No. RetainUntilDate: Required: No.
- PutObjectLockConfiguration: <https://docs.aws.amazon.com/AmazonS3/latest/API/API_PutObjectLockConfiguration.html>
  - > ObjectLockConfiguration is required.
- PutBucketWebsite: <https://docs.aws.amazon.com/AmazonS3/latest/API/API_PutBucketWebsite.html>
  - > WebsiteConfiguration is required.
- PutObjectLegalHold: <https://docs.aws.amazon.com/AmazonS3/latest/API/API_PutObjectLegalHold.html>
  - > LegalHold is required. Status: Required: No. Valid Values: ON | OFF.

## 完了条件

- 上記 API で空 XML / 必須未設定が builder 段階でエラーになる
- 正常系の統合テストが通る
- `PutObjectLegalHold` が未指定時に暗黙 ON にならない
- CHANGES.md に `[CHANGE]` エントリ（`PutObjectLegalHold` の暗黙デフォルト廃止）と `[ADD]` エントリ（バリデーション追加）を追記する
- 各 API の単体テストでエラーパスを検証する（モックを使わず、builder の `build_request` を呼び出して `Error::InvalidInput` が返されることを検証）

## 解決方法

1. `put_bucket_encryption.rs` の `build_request` で `rules` 空チェックと `sse_algorithm` 空チェックを追加
2. `put_bucket_cors.rs` の `build_request` で `cors_rules` 空チェックを追加
3. `put_bucket_lifecycle_configuration.rs` の `build_request` で `rules` 空チェックを追加
4. `put_bucket_ownership_controls.rs` の `build_request` で `rules` 空チェックを追加
5. `put_object_retention.rs` の `build_request` で `mode` と `retain_until_date` の両方未設定チェックを追加
6. `put_object_lock_configuration.rs` の `build_request` で `object_lock_configuration` が `None` の場合のチェックを追加
7. `put_bucket_website.rs` の `build_request` で全フィールド未指定チェックと相互排他チェックを追加
8. `put_object_legal_hold.rs` の `build_request` で `legal_hold_status` 未指定チェックを追加（暗黙 `"ON"` 廃止）
9. 各 API の単体テストでエラーパスを検証（モック不使用）
