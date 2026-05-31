# GetBucket 系 API および GetObjectLockConfiguration で XML パースエラーを黙殺する

- Priority: High
- Created: 2026-05-25
- Model: Composer 2.5
- Polished: 2026-05-31

## 目的

GetBucket 系 API および GetObjectLockConfiguration の XML パーサーが `Err(_)` を握り潰し、破損 XML を正常レスポンスとして返す。利用者が不完全な設定データを正しい値と誤認するため、早期に `Error::InvalidResponse` を返す必要がある。S3 互換サーバーとの通信でネットワーク切断やプロキシの異常応答により XML が途中で切断された場合、CORS / 暗号化 / ライフサイクル等の設定が欠落したまま `Ok` になる。データ欠落はサイレントであり、設定ミスや移行失敗の検出を困難にする。

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

加えて、以下の 2 つの問題がある。

1. `get_bucket_lifecycle_configuration.rs:235` で `Status` パース失敗時に `ExpirationStatus::Enabled` にフォールバックしている。

```rust
status.parse::<ExpirationStatus>().unwrap_or(ExpirationStatus::Enabled)
```

2. 複数ファイルで `.parse().ok()` による値パース失敗を黙殺している（`ObjectSizeGreaterThan`、`Days`、`MaxAgeSeconds`、`Years` 等）。これらも「破損データを正常レスポンスとして返す」という点で同一カテゴリのバグであるため、本 issue で対応する。

## スコープ外

以下の問題は本 issue のスコープ外とする。

- `xml.rs` の `extract_element` (`Err(_) => return None`) と `for_each_element` (`Err(_) => return`) のエラー握り潰し（issue 0079 で対応）
- `parse_response` 側の `.and_then(|v| v.parse().ok())` 等のパース失敗黙殺（`src/api/mod.rs:206` 等）
- boolean フィールドの `== "true"` 比較によるサイレントデフォルト化（`get_bucket_encryption.rs:132`、`get_bucket_lifecycle_configuration.rs:369`）
- `DefaultRetention.mode` のバリデーション不在（`get_object_lock_configuration.rs:129`、`Option<String>` で `GOVERNANCE` / `COMPLIANCE` の検証なし）
- `_ =>` フォールスルーで未知タグを黙認する問題
- `current_tag` がコンテキスト遷移時にクリアされない問題

## 設計方針

### ヘルパー関数の `Result` 化

EventReader を直接使用する 6 つのヘルパー関数の戻り値を `Result<T, Error>` に変更し、`parse_response` 側で `?` 演算子で伝播する。0075 のヘルパー関数は EventReader を直接使用しており `xml.rs` の API を使っていないため、0079 の影響を受けない。

| 関数名 | 現行戻り値型 | 変更後戻り値型 |
|--------|-------------|---------------|
| `extract_encryption_rules` | `Vec<ServerSideEncryptionRule>` | `Result<Vec<ServerSideEncryptionRule>, Error>` |
| `extract_lifecycle_rules` | `Vec<LifecycleRule>` | `Result<Vec<LifecycleRule>, Error>` |
| `extract_cors_rules` | `Vec<CorsRule>` | `Result<Vec<CorsRule>, Error>` |
| `parse_object_lock_configuration` | `ObjectLockConfiguration` | `Result<ObjectLockConfiguration, Error>` |
| `parse_website_configuration` | `GetBucketWebsiteOutput` | `Result<GetBucketWebsiteOutput, Error>` |
| `parse_notification_configuration` | `GetBucketNotificationConfigurationOutput` | `Result<GetBucketNotificationConfigurationOutput, Error>` |

パースループ内の `Err(_) => return rules` / `Err(_) => break` を `Err(_) => return Err(Error::InvalidResponse("..."))` に変更する。`break` 後にデフォルト値で構造体を構築している箇所（`get_object_lock_configuration.rs:148-151`、`get_bucket_website.rs:251-256`、`get_bucket_notification_configuration.rs:280-285`）は、`break` を `return Err(...)` に置き換える。

### `.parse().ok()` の `InvalidResponse` 化

`.parse::<i32>().ok()` 等で値パース失敗を `None` に変換している箇所を、パース失敗時に `Error::InvalidResponse` を返すように変更する。`.ok_or(Error::InvalidResponse(...))` のパターンで `Option` → `Result` 変換を行う。対象箇所:

- `get_bucket_lifecycle_configuration.rs:281,283` — `ObjectSizeGreaterThan`、`ObjectSizeLessThan`
- `get_bucket_lifecycle_configuration.rs:327,328` — And 内の同上
- `get_bucket_lifecycle_configuration.rs:366` — `Days`
- `get_bucket_lifecycle_configuration.rs:391` — Transition の `Days`
- `get_bucket_lifecycle_configuration.rs:413,415` — `NoncurrentDays`、`NewerNoncurrentVersions`
- `get_bucket_lifecycle_configuration.rs:439,444` — NoncurrentVersionTransition の同上
- `get_bucket_lifecycle_configuration.rs:464` — `DaysAfterInitiation`
- `get_bucket_cors.rs:121` — `MaxAgeSeconds`
- `get_object_lock_configuration.rs:132,135` — `Days`、`Years`

`get_bucket_website.rs` と `get_bucket_notification_configuration.rs` には `.parse().ok()` パターンが存在しない（全て文字列として格納）。

### `Status` フォールバックの除去

`get_bucket_lifecycle_configuration.rs:235` の `.unwrap_or(ExpirationStatus::Enabled)` を削除し、パース失敗時に `Error::InvalidResponse` を返す。`ExpirationStatus` の `FromStr` 実装（`src/types.rs:1353-1364`）は `"Enabled"` と `"Disabled"` のみ `Ok` を返し、空文字列は `Err` を返すため、`unwrap_or` 除去で正しくエラーになる。

### エラーメッセージ

`Error::InvalidResponse(String)` には、どの要素のパースに失敗したかを含める。エラーメッセージのフォーマット: `"failed to parse {element} in {parent_element}"`。`parent_element` はパーサーの状態機械のコンテキストから取得する（例: `inside_default` フラグ、`ctx` enum 値）。具体例:

- `"failed to parse SSEAlgorithm in ApplyServerSideEncryptionByDefault"`
- `"failed to parse Status in LifecycleRule"`
- `"failed to parse MaxAgeSeconds in CORSRule"`
- `"failed to parse Days in DefaultRetention"`
- `"failed to parse ObjectSizeGreaterThan in Filter > And"`

### CHANGES.md の種別

全て `[FIX]`（バグ修正）として記載する。

## AWS S3 API Reference

- GetBucketEncryption: <https://docs.aws.amazon.com/AmazonS3/latest/API/API_GetBucketEncryption.html>
  - > The server-side encryption configuration information.
- GetBucketLifecycleConfiguration: <https://docs.aws.amazon.com/AmazonS3/latest/API/API_GetBucketLifecycleConfiguration.html>
  - > The lifecycle configuration information.
- GetBucketCors: <https://docs.aws.amazon.com/AmazonS3/latest/API/API_GetBucketCors.html>
  - > The CORS configuration information.
- GetObjectLockConfiguration: <https://docs.aws.amazon.com/AmazonS3/latest/API/API_GetObjectLockConfiguration.html>
  - > The Object Lock configuration information.
- GetBucketWebsite: <https://docs.aws.amazon.com/AmazonS3/latest/API/API_GetBucketWebsite.html>
  - > The website configuration information.
- GetBucketNotificationConfiguration: <https://docs.aws.amazon.com/AmazonS3/latest/API/API_GetBucketNotificationConfiguration.html>
  - > The notification configuration information.

## 完了条件

- 上記 6 API すべてで、不正 XML に対して `Error::InvalidResponse` を返す
- ヘルパー関数の戻り値を `Result` に変更し、`parse_response` で `?` で伝播する
- `Status` フォールバックが除去され、パース失敗はエラーになる
- `.parse().ok()` による値パース失敗の黙殺が解消される
- MinIO / RustFS 統合テストで正常系が引き続き通る
- 各 API のテストファイルに malformed XML のエラーパス単体テストを追加する（新規作成）
- `fuzz/fuzz_targets/fuzz_xml_parse.rs` に `GetBucketCorsFluentBuilder` と `GetObjectLockConfigurationFluentBuilder` のパース呼び出しを追加する（現状 15 API 登録済みのうち、この 2 が未登録）

## 0079 との関係

本 issue は EventReader を直接使っている 6 ファイルの `Err(_)` ハンドリングと `.parse().ok()` の黙殺に焦点を当てている。issue 0079 は `xml.rs` の内部 API（`extract_element`、`for_each_element`）のエラーハンドリング統一を対象としており、対象範囲が異なる。0075 を先に対応し、0079 で `xml.rs` の内部 API を統一する。
