# GetBucket 系 API および GetObjectLockConfiguration で XML パースエラーを黙殺する

- Priority: High
- Created: 2026-05-25
- Model: Composer 2.5
- Polished: 2026-06-06
- Completed: 2026-06-06
- Branch: feature/fix-get-bucket-xml-parse-error-swallowing

## 目的

GetBucket 系 API および GetObjectLockConfiguration の XML パーサーが `Err(_)` を握り潰し、破損 XML を正常レスポンスとして返す。S3 互換サーバーとの通信でネットワーク切断やプロキシの異常応答により XML が途中で切断された場合、CORS / 暗号化 / ライフサイクル等の設定が欠落したまま `Ok` になる。データ欠落はサイレントであり、設定ミスや移行失敗の検出を困難にする。

## 優先度根拠

本番環境で顕在化した場合、CORS ルールの一部消失は予期しないアクセス制御の緩みを引き起こし、暗号化設定の欠落はデータ保護の喪失に直結する。設定データを完全に欠落させたまま正常終了するバグの影響度が極めて高いため High とする。

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

`return rules` は不完全なルールセットを返す（部分欠落した Vec）。
`break` はループを抜けて、break 前にパースが完了したフィールド値で構造体を構築する（一部フィールドが欠落した不完全な構造体）。

加えて、以下の 2 つの問題がある。

1. `get_bucket_lifecycle_configuration.rs:235` で `Status` パース失敗時に `ExpirationStatus::Enabled` にフォールバックしている。

```rust
status.parse::<ExpirationStatus>().unwrap_or(ExpirationStatus::Enabled)
```

2. 複数ファイルで `.parse().ok()` による値パース失敗を黙殺している（`ObjectSizeGreaterThan`、`Days`、`MaxAgeSeconds`、`Years` 等）。

## スコープ外

以下の問題は本 issue のスコープ外とする。0075 を先に対応し、0079 で `xml.rs` の内部 API を統一する。

## 解決方法

6 つのヘルパー関数の戻り値を `Result<T, Error>` に変更し、XML パースエラー時に `Error::InvalidResponse` を返すように修正した。

1. `get_bucket_encryption.rs`: `extract_encryption_rules` → `Result<Vec<ServerSideEncryptionRule>, Error>`、`Err(_) => return rules` → `return Err(InvalidResponse(...))`
2. `get_bucket_cors.rs`: `extract_cors_rules` → `Result<Vec<CorsRule>, Error>`、`MaxAgeSeconds` の `.parse().ok()` → `.map_err()?`
3. `get_bucket_lifecycle_configuration.rs`: `extract_lifecycle_rules` → `Result<Vec<LifecycleRule>, Error>`、`.unwrap_or(ExpirationStatus::Enabled)` 除去、全 `.parse().ok()` (14 箇所) → `.map_err()?`
4. `get_object_lock_configuration.rs`: `parse_object_lock_configuration` → `Result<ObjectLockConfiguration, Error>`、`Err(_) => break` → `return Err(...)`、`Days`/`Years` の `.parse().ok()` → `.map_err()?`
5. `get_bucket_website.rs`: `parse_website_configuration` → `Result<GetBucketWebsiteOutput, Error>`、`Err(_) => break` → `return Err(...)`
6. `get_bucket_notification_configuration.rs`: `parse_notification_configuration` → `Result<GetBucketNotificationConfigurationOutput, Error>`、`Err(_) => break` → `return Err(...)`

追加・変更したテスト:
- `tests/test_get_bucket_encryption.rs` — 切断 XML、non-UTF-8 ボディ、SSEAlgorithm 空文字列
- `tests/test_get_bucket_cors.rs` — 切断 XML、非数値 MaxAgeSeconds、オーバーフロー MaxAgeSeconds
- `tests/test_get_bucket_lifecycle_configuration.rs` — 切断 XML、無効 Status、非数値 Days、非数値 ObjectSizeGreaterThan
- `tests/test_get_object_lock_configuration.rs` — 切断 XML、非数値 Days、非数値 Years
- `tests/test_get_bucket_website.rs` — 切断 XML
- `tests/test_get_bucket_notification_configuration.rs` — 切断 XML
- `fuzz/fuzz_targets/fuzz_xml_parse.rs` — `GetBucketCorsFluentBuilder` と `GetObjectLockConfigurationFluentBuilder` を追加

- `xml.rs` の `extract_element` (`Err(_) => return None`) と `for_each_element` (`Err(_) => return`) のエラー握り潰し — issue 0079 で対応
- `parse_response` 側の `.and_then(|v| v.parse().ok())` 等のパース失敗黙殺 — 0079 の design に含まれていないため、新規 issue 化が必要
- boolean フィールドの `== "true"` 比較によるサイレントデフォルト化 — 新規 issue 化が必要
- `DefaultRetention.mode` のバリデーション不在（`Option<String>` で `GOVERNANCE` / `COMPLIANCE` の検証なし） — 新規 issue 化が必要
- `_ =>` フォールスルーで未知タグを黙認する問題 — 新規 issue 化が必要
- `current_tag` がコンテキスト遷移時にクリアされない問題 — 本修正でパースエラー発生時に早期リターンするため partial state の後続 Rule への漏洩リスクは軽減される。根本解決は別 issue で対応

## 設計方針

### ヘルパー関数の `Result` 化

6 つのヘルパー関数の戻り値を `Result<T, Error>` に変更する。

| 関数名 | 現行戻り値型 | 変更後戻り値型 |
|--------|-------------|---------------|
| `extract_encryption_rules` | `Vec<ServerSideEncryptionRule>` | `Result<Vec<ServerSideEncryptionRule>, Error>` |
| `extract_lifecycle_rules` | `Vec<LifecycleRule>` | `Result<Vec<LifecycleRule>, Error>` |
| `extract_cors_rules` | `Vec<CorsRule>` | `Result<Vec<CorsRule>, Error>` |
| `parse_object_lock_configuration` | `ObjectLockConfiguration` | `Result<ObjectLockConfiguration, Error>` |
| `parse_website_configuration` | `GetBucketWebsiteOutput` | `Result<GetBucketWebsiteOutput, Error>` |
| `parse_notification_configuration` | `GetBucketNotificationConfigurationOutput` | `Result<GetBucketNotificationConfigurationOutput, Error>` |

パースループ内の変更:
- `Err(_) => return rules` → `Err(_) => return Err(Error::InvalidResponse("..."))`
- `Err(_) => break` → `Err(_) => return Err(Error::InvalidResponse("..."))`

`break` → `return Err(...)` 変更後の注意点: `break` 後の構造体構築コード（`get_object_lock_configuration.rs:148-151`、`get_bucket_website.rs:251-256`、`get_bucket_notification_configuration.rs:280-285`）は、正常系のパスでは引き続き到達するため削除しない。ただし戻り値型が `Result` に変わるため、正常系の末尾式は `Ok(構造体)` に変更する必要がある。

### `parse_response` 側の変更

ヘルパー関数が `Result` を返すようになった後の `parse_response` 側の変更。6 ファイルで既に `parse_response` が `Result<T, Error>` を返している。

**パターン A: 中間変数で受け取るもの**（3 ファイル）

`extract_encryption_rules`、`extract_cors_rules`、`parse_object_lock_configuration` は戻り値が `Vec` または構造体で、`parse_response` 内でさらに加工が必要なため中間変数で受け取る。

```rust
// 現行 (get_bucket_encryption.rs:57-64)
let rules = extract_encryption_rules(body_text);
Ok(GetBucketEncryptionOutput {
    server_side_encryption_configuration: if rules.is_empty() {
        None
    } else {
        Some(ServerSideEncryptionConfiguration { rules })
    },
})

// 変更後
let rules = extract_encryption_rules(body_text)?;
Ok(GetBucketEncryptionOutput {
    server_side_encryption_configuration: if rules.is_empty() {
        None
    } else {
        Some(ServerSideEncryptionConfiguration { rules })
    },
})
```

```rust
// 現行 (get_bucket_cors.rs:52-55)
Ok(GetBucketCorsOutput {
    cors_rules: Some(extract_cors_rules(body_text)),
})

// 変更後
let cors_rules = extract_cors_rules(body_text)?;
Ok(GetBucketCorsOutput {
    cors_rules: Some(cors_rules),
})
```

```rust
// 現行 (get_object_lock_configuration.rs:55-58)
Ok(GetObjectLockConfigurationOutput {
    object_lock_configuration: Some(parse_object_lock_configuration(body_text)),
})

// 変更後
let config = parse_object_lock_configuration(body_text)?;
Ok(GetObjectLockConfigurationOutput {
    object_lock_configuration: Some(config),
})
```

**パターン B: 直接伝播**（3 ファイル）

```rust
// 現行 (get_bucket_lifecycle_configuration.rs:58-61)
Ok(GetBucketLifecycleConfigurationOutput {
    rules: extract_lifecycle_rules(body_text),
})

// 変更後: .map() で Output にラップする (戻り値は Vec<LifecycleRule>)
extract_lifecycle_rules(body_text).map(|rules| GetBucketLifecycleConfigurationOutput { rules })
```

```rust
// 現行 (get_bucket_website.rs:54)
Ok(parse_website_configuration(body_text))

// 変更後: Ok ラッパーを除去する (戻り値型と Output 型が一致)
parse_website_configuration(body_text)
```

```rust
// 現行 (get_bucket_notification_configuration.rs:57)
Ok(parse_notification_configuration(body_text))

// 変更後: Ok ラッパーを除去する (戻り値型と Output 型が一致)
parse_notification_configuration(body_text)
```

### `.parse().ok()` の置換

`.parse::<i32>().ok()` を `Some(.parse().map_err(|_| Error::InvalidResponse("..."))?)` に置き換える。
代入先の変数型は全て `Option<i32>` または `Option<i64>` であるため、`Some(...)` でラップする必要がある。
14 箇所すべて `Some(.parse().map_err(|_| Error::InvalidResponse("..."))?)` のパターンで統一する。

`.map_err()` 内のエラーメッセージは XML イベントの `Err(_)` 分岐と同じフォーマット `"failed to parse {element} in {parent_element}"` を使う。各箇所の `{element}` と `{parent_element}` は以下の「エラーメッセージ」節のマッピング表から決定する。

対象箇所（全 14 箇所）:

- `get_bucket_lifecycle_configuration.rs:281,283` — `ObjectSizeGreaterThan`、`ObjectSizeLessThan`（Filter）
- `get_bucket_lifecycle_configuration.rs:327,328` — 同上（Filter > And）
- `get_bucket_lifecycle_configuration.rs:366` — `Days`（Expiration）
- `get_bucket_lifecycle_configuration.rs:391` — `Days`（Transition）
- `get_bucket_lifecycle_configuration.rs:413,415` — `NoncurrentDays`、`NewerNoncurrentVersions`
- `get_bucket_lifecycle_configuration.rs:439,444` — 同上（NoncurrentVersionTransition）
- `get_bucket_lifecycle_configuration.rs:464` — `DaysAfterInitiation`
- `get_bucket_cors.rs:121` — `MaxAgeSeconds`
- `get_object_lock_configuration.rs:132,135` — `Days`、`Years`

`get_bucket_website.rs` と `get_bucket_notification_configuration.rs` には `.parse().ok()` が存在しないため、`Err(_) => break` の修正のみでよい。

### `Status` フォールバックの除去

`get_bucket_lifecycle_configuration.rs:235` の `.unwrap_or(ExpirationStatus::Enabled)` を削除する。`ExpirationStatus::FromStr`（`src/types.rs:1353-1364`）により、`"Enabled"` / `"Disabled"` 以外の値および空文字列で `Err` を返し、`?` で伝播される。

### エラーメッセージ

XML イベントの `Err(_)` 分岐および `.map_err()` のメッセージフォーマット: `"failed to parse {element} in {parent_element}"`。

`{element}`: `current_tag` の値。`current_tag` が `None` の場合は `"unknown"` とする。
`{parent_element}`: パーサーの状態変数から以下のマッピング表に従って決定する。

**get_bucket_encryption.rs**（boolean フラグベース）:

| 条件 | parent_element |
|------|---------------|
| `inside_rule = false` および `inside_default = false` | `"ServerSideEncryptionConfiguration"` |
| `inside_rule = true`, `inside_default = true` | `"ApplyServerSideEncryptionByDefault"` |
| `inside_rule = true`, `inside_default = false` | `"Rule"` |

**get_bucket_cors.rs**（boolean フラグベース）:

| 条件 | parent_element |
|------|---------------|
| `inside_rule = false` | `"CORSConfiguration"` |
| `inside_rule = true` | `"CORSRule"` |

**get_object_lock_configuration.rs**（boolean フラグベース）:

| 条件 | parent_element |
|------|---------------|
| `inside_rule = false` および `inside_default_retention = false` | `"ObjectLockConfiguration"` |
| `inside_rule = true`, `inside_default_retention = false` | `"Rule"` |
| `inside_rule = true`, `inside_default_retention = true` | `"DefaultRetention"` |

**get_bucket_lifecycle_configuration.rs**（enum Context ベース）:

| Context | parent_element |
|---------|---------------|
| `Context::None` | `"LifecycleConfiguration"` |
| `Context::Rule` | `"Rule"` |
| `Context::Filter` | `"Filter"` |
| `Context::FilterAnd` | `"Filter > And"` |
| `Context::FilterTag` | `"Filter > Tag"` |
| `Context::FilterAndTag` | `"Filter > And > Tag"` |
| `Context::Expiration` | `"Expiration"` |
| `Context::Transition` | `"Transition"` |
| `Context::NoncurrentVersionExpiration` | `"NoncurrentVersionExpiration"` |
| `Context::NoncurrentVersionTransition` | `"NoncurrentVersionTransition"` |
| `Context::AbortIncompleteMultipartUpload` | `"AbortIncompleteMultipartUpload"` |

**get_bucket_website.rs**（enum Ctx ベース）:

| Ctx | parent_element |
|-----|---------------|
| `Ctx::Root` または `Ctx::RoutingRules` | `"WebsiteConfiguration"` |
| `Ctx::IndexDocument` | `"IndexDocument"` |
| `Ctx::ErrorDocument` | `"ErrorDocument"` |
| `Ctx::RedirectAll` | `"RedirectAllRequestsTo"` |
| `Ctx::RoutingRule` | `"RoutingRule"` |
| `Ctx::Condition` | `"Condition"` |
| `Ctx::Redirect` | `"Redirect"` |

**get_bucket_notification_configuration.rs**（enum Context ベース）:

| Context | parent_element |
|---------|---------------|
| `Context::Root` | `"NotificationConfiguration"` |
| `Context::Topic` | `"TopicConfiguration"` |
| `Context::Queue` | `"QueueConfiguration"` |
| `Context::Lambda` | `"CloudFunctionConfiguration"` |
| `Context::Filter` | `"Filter"` |
| `Context::S3Key` | `"S3Key"` |
| `Context::FilterRule` | `"FilterRule"` |

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
- ヘルパー関数の戻り値を `Result` に変更し、`parse_response` で適切に伝播する
- `Status` フォールバックが除去され、パース失敗はエラーになる
- `.parse().ok()` による値パース失敗の黙殺が `Some(.parse().map_err()?)` に置き換わる
- MinIO / RustFS 統合テストで正常系が引き続き通る
- `fuzz/fuzz_targets/fuzz_xml_parse.rs` に以下を追加する:
  - `use` 行に `GetBucketCorsFluentBuilder` と `GetObjectLockConfigurationFluentBuilder` を追加
  - `fuzz_target!` ブロック内に `let _ = GetBucketCorsFluentBuilder::parse_response(&response);` と `let _ = GetObjectLockConfigurationFluentBuilder::parse_response(&response);` を追加
- 以下のテストファイルを新規作成し malformed XML のエラーパス単体テストを追加する:
  - `tests/test_get_bucket_encryption.rs` — 不完全タグ、`SSEAlgorithm` 空文字列
  - `tests/test_get_bucket_lifecycle_configuration.rs` — 不完全タグ、`Status` 無効値、非数値 `Days`/`ObjectSizeGreaterThan`
  - `tests/test_get_bucket_cors.rs` — 不完全タグ、非数値 `MaxAgeSeconds`（`"-1"` はパース成功するため、`"abc"` 等の非数値文字列および `i32` 範囲オーバーフロー値でテストする）
  - `tests/test_get_object_lock_configuration.rs` — 不完全タグ、非数値 `Days`/`Years`
  - `tests/test_get_bucket_website.rs` — 不完全タグ
  - `tests/test_get_bucket_notification_configuration.rs` — 不完全タグ
- CHANGES.md に以下を追記する（担当者行を含める）:
  - `- [FIX] GetBucket 系 API および GetObjectLockConfiguration で XML パースエラーを握り潰すバグを修正する`
    - `@voluntas`
  - `- [FIX] .parse().ok() による数値フィールドのパース失敗黙殺を修正する`
    - `@voluntas`
  - `- [FIX] Status パース失敗時の ExpirationStatus::Enabled フォールバックを除去する`
    - `@voluntas`
