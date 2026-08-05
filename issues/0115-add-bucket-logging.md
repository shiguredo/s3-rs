# Bucket Logging API を追加する

- Priority: Medium
- Created: 2026-07-31
- Completed: {YYYY-MM-DD}
- Branch: feature/add-bucket-logging
- Polished: 2026-08-06
- Model: GPT-5

## 目的

GetBucketLogging と PutBucketLogging を追加し、server access logging の設定を aws-sdk-rust 互換に扱えるようにする。

## 現状

src/api/ に logging サブリソース用の operation と BucketLoggingStatus、LoggingEnabled の XML モデルが存在しない。

## 設計方針

- リクエスト URI は `GET /?logging` と `PUT /?logging`。既存の `get_bucket_encryption.rs` / `put_bucket_encryption.rs` と同様に、`build_signed_request` へ query param `("logging", "")` を渡して構築する
- 型の構成（aws-sdk-rust 互換）:
  - `BucketLoggingStatus` 構造体: `logging_enabled: Option<LoggingEnabled>`
  - `LoggingEnabled` 構造体: `target_bucket: String`（必須）/ `target_grants: Option<Vec<TargetGrant>>` / `target_prefix: String`（必須）/ `target_object_key_format: Option<TargetObjectKeyFormat>`
  - `TargetGrant` 構造体: `grantee: Option<Grantee>` / `permission: Option<BucketLogsPermission>`
  - `BucketLogsPermission` enum: `FullControl`（ワイヤー値 `FULL_CONTROL`）/ `Read`（`READ`）/ `Write`（`WRITE`）/ `Unknown(String)` の 4 variant（aws-sdk-rust の `BucketLogsPermission` に準拠。`as_str()` / `From<&str>` はワイヤー値を正しくマップする）
  - `TargetObjectKeyFormat` 構造体: `simple_prefix: Option<SimplePrefix>` / `partitioned_prefix: Option<PartitionedPrefix>`
  - `PartitionedPrefix` 構造体: `partition_date_source: Option<PartitionDateSource>`
  - `PartitionDateSource` enum: `DeliveryTime` / `EventTime` / `Unknown(String)` の 3 variant（aws-sdk-rust の `PartitionDateSource` に準拠。closed 0059 / 0084 の「関連機能を実装する際にその一部として enum 化を導入する」方針に従い、本 issue で導入する）
  - `SimplePrefix` 構造体: フィールドなし。XML では空要素 `<SimplePrefix/>` として生成・パースする
  - `GetBucketLoggingOutput`: `logging_enabled: Option<LoggingEnabled>`
  - `PutBucketLoggingOutput`: フィールドなし（空構造体）
  - 全 enum に `#[non_exhaustive]`、`as_str()` / `From<&str>` / `Display` を実装（`src/types.rs` の enum 方針に準拠）
- `Grantee` 構造体と `Type` enum、および `src/xml.rs` の `xsi:type` 属性サポートは issue 0113 が導入するため、本 issue では再利用する（XML の `<Grantee xmlns:xsi="..." xsi:type="CanonicalUser">` 形式）
- 新規モデル型（`BucketLoggingStatus` / `LoggingEnabled` / `TargetGrant` / `TargetObjectKeyFormat` 等）は既存の `ServerSideEncryptionConfiguration` と同じ builder パターンで構築する。`LoggingEnabledBuilder::build()` は `target_bucket` / `target_prefix` 未指定時に `Error::InvalidInput` を返す（aws-sdk-rust の `LoggingEnabled` は両フィールドが必須）。`TargetObjectKeyFormatBuilder::build()` は `simple_prefix` と `partitioned_prefix` の同時指定を `Error::InvalidInput` で弾く（AWS 仕様が排他を要求している。既存の PutBucketWebsite の排他検証と同じ方針）
- `PutBucketLoggingFluentBuilder::build_request()` は `bucket_logging_status` 未指定時に `Error::InvalidInput` を返す。空の `BucketLoggingStatus`（無効化）は有効な入力だが、`bucket_logging_status` 自体の未指定は不正（既存の PutBucketEncryption の空入力検証と同じ方針。aws-sdk-rust の `bucket_logging_status` も必須フィールド）
- レスポンスの `<LoggingEnabled>` 内に `<TargetBucket>` / `<TargetPrefix>` が欠落している場合は `Error::InvalidResponse` を返す（0113 の必須フィールド欠落方針と同じ。aws-sdk-rust は空文字で埋めるが、本プロジェクトの慣行に従う）
- XML の `target_grants` は `<TargetGrants>` 要素が `<Grant>` 要素を包む構造（フィールド名 `target_grants` と XML 要素名 `Grant` は不一致。aws-sdk-rust の `shape_logging_enabled` に従う）。生成・パースの両方で `<TargetGrants>` ラッパー要素を扱う
- `<TargetObjectKeyFormat>` 内の要素順は aws-sdk-rust の `shape_target_object_key_format` に従う
- PutBucketLogging の XML ボディに対する Content-MD5 と x-amz-sdk-checksum-algorithm は既存の XML API（`put_bucket_encryption.rs` 等）と同じ方針で処理する
- logging の無効化は、`logging_enabled` を持たない空の `<BucketLoggingStatus/>` を `PUT /?logging` で送ることで実現する（aws-sdk-rust の `PutBucketLoggingFluentBuilder.bucket_logging_status` の `logging_enabled` 未指定に相当）
- directory bucket（S3 Express One Zone）では GetBucketLogging / PutBucketLogging は非サポートだが、directory bucket は issue 0125 で未実装のため、本 issue では directory bucket の制約をテストに反映しない。0125 完了時に制約の検証を追加する
- 統合テストでは EmailAddress 指定の target grant を検証しない（2025-10-01 に AWS が Email Grantee ACL を廃止済み。0113 と同じ扱い）

### 変更対象ファイル

- `src/api/get_bucket_logging.rs` (新規作成)
- `src/api/put_bucket_logging.rs` (新規作成)
- `src/api/mod.rs` (モジュール宣言 + FluentBuilder の pub use 追加)
- `src/types.rs` (`BucketLoggingStatus`, `LoggingEnabled`, `TargetGrant`, `BucketLogsPermission`, `TargetObjectKeyFormat`, `PartitionedPrefix`, `PartitionDateSource`, `SimplePrefix`, `GetBucketLoggingOutput`, `PutBucketLoggingOutput` 追加)
- `src/client.rs` (`get_bucket_logging()` / `put_bucket_logging()` メソッド追加)
- `src/lib.rs` (新しいモデル型の crate ルート再エクスポート追加。Output 型は既存パターンどおりルート再エクスポートしない)
- `tests/test_bucket_logging.rs` (新規作成)
- `tests/minio.rs` / `tests/rustfs.rs` / `tests/kikyo.rs` (統合テストを追加。`?logging` サブリソースに未対応のサーバーは完了条件のとおり統合テスト対象外)

## 完了条件

- `GET /?logging` のリクエストを正しい URI、ヘッダーで構築でき、`PUT /?logging` のリクエストを正しい URI、ヘッダー（Content-MD5 含む）、XML ボディで構築できる。checksum_algorithm 指定時は x-amz-sdk-checksum-algorithm と対応するチェックサムヘッダーが付与される
- logging 有効時（target bucket / target prefix / target grants / target object key format）と無効時（空の `<BucketLoggingStatus/>`）のレスポンスを正しくパースし、無効時は `logging_enabled: None` になる（未知の値は `Unknown(String)` としてパースされることも検証する。`TargetObjectKeyFormat` / `PartitionedPrefix` / `SimplePrefix` / `PartitionDateSource` の往復も検証する）
- 実際の S3 互換サーバー（MinIO / RustFS / kikyo-local）を `shiguredo_container` で起動した統合テストで検証する。`?logging` サブリソースに未対応のサーバーがある場合は、そのサーバー名と未対応の根拠を issue に追記した上で、対応しているサーバーでのみ統合テストを行う
- 既存のテストが全て通過すること
- `cargo clippy --workspace --all-targets -- -D warnings` が通過すること
- CHANGES.md の `## develop` に `[ADD]` エントリを追加すること

## AWS S3 API Reference

- GetBucketLogging: https://docs.aws.amazon.com/AmazonS3/latest/API/API_GetBucketLogging.html

> Returns the logging status of a bucket and the permissions users have to view and modify that status.

- PutBucketLogging: https://docs.aws.amazon.com/AmazonS3/latest/API/API_PutBucketLogging.html

> Set the logging parameters for a bucket and to specify permissions for who can view and modify the logging parameters.

## 他 issue との依存関係

- 0057（pending: expected_bucket_owner / request_payer）は全 API 横断で管理。本 issue では対応しない（0057 完了時に追加される）
- 0113（Bucket / Object ACL API）は `Type` enum、`Grantee` 構造体、`src/xml.rs` の `xsi:type` 属性サポートを導入する。本 issue はこれらを再利用するため、0113 の実装後に着手する。0113 実装完了時に、属性サポートが `TargetGrants` → `Grant` → `Grantee` のネスト構造で使えることを確認する
- 0131（Fluent Builder の set_* 残件）は src/api/ 配下の全 Fluent Builder を対象とする。本 issue で新規追加する Fluent Builder の `set_*` メソッドは 0131 の管轄とする