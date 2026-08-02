# Bucket Accelerate Configuration API を追加する

- Priority: Medium
- Created: 2026-07-31
- Completed: {YYYY-MM-DD}
- Branch: feature/add-bucket-accelerate-configuration
- Polished: 2026-08-02
- Model: GPT-5

## 目的

GetBucketAccelerateConfiguration と PutBucketAccelerateConfiguration を追加し、Transfer Acceleration の設定を aws-sdk-rust 互換に扱えるようにする。

## 現状

src/api/ に accelerate サブリソース用の operation と AccelerateConfiguration の XML 処理が存在しない。

## 設計方針

- GetBucketAccelerateConfiguration と PutBucketAccelerateConfiguration の builder を追加する。リクエスト URI は `GET /?accelerate` と `PUT /?accelerate`
- 型の構成（aws-sdk-rust 互換）:
  - `BucketAccelerateStatus` enum: `Enabled` / `Suspended` / `Unknown(String)` の 3 variant。`#[non_exhaustive]`、`as_str()` / `From<&str>` / `Display` を実装（`src/types.rs` の enum 方針に準拠）
  - `AccelerateConfiguration` 構造体: `status: Option<BucketAccelerateStatus>` の 1 フィールド
  - `GetBucketAccelerateConfigurationOutput`: `status: Option<BucketAccelerateStatus>`
  - `PutBucketAccelerateConfigurationFluentBuilder`: `accelerate_configuration` 必須（未指定時は `Error::InvalidInput`）
- PutBucketAccelerateConfiguration の Content-MD5 と x-amz-sdk-checksum-algorithm を既存の XML API（`put_bucket_encryption.rs` 等）と同じ方針で処理する
- `expected_bucket_owner` は pending issue 0057 が全 API 横断で管理しているため、本 issue のスコープ外とする（0057 完了時に追加される）
- GetBucketAccelerateConfiguration で accelerate 未設定のバケットに対するレスポンス（空ボディか `Suspended` 返却か）は、実装時に AWS 公式ドキュメントまたは実レスポンスで確認する
- Transfer Acceleration は AWS 固有（CloudFront 依存）の機能であり、S3 互換ストレージで意味が薄い。本 issue では aws-sdk-rust の API 表面互換のために実装する（CODEBASE.md の方針に従う）
- directory bucket（S3 Express One Zone）では Transfer Acceleration は利用できないが、directory bucket は issue 0125 で未実装のため、本 issue では directory bucket の制約をテストに反映しない。0125 完了時に制約の検証を追加する

### 変更対象ファイル

- `src/api/get_bucket_accelerate_configuration.rs` (新規作成)
- `src/api/put_bucket_accelerate_configuration.rs` (新規作成)
- `src/api/mod.rs` (モジュール宣言 + FluentBuilder の pub use 追加)
- `src/types.rs` (`BucketAccelerateStatus`, `AccelerateConfiguration`, `GetBucketAccelerateConfigurationOutput` 追加)
- `src/client.rs` (`get_bucket_accelerate_configuration()` / `put_bucket_accelerate_configuration()` メソッド追加)
- `src/lib.rs` (新しい型の公開)
- `tests/test_bucket_accelerate_configuration.rs` (新規作成)

## 完了条件

- `GET /?accelerate` と `PUT /?accelerate` のリクエストを正しい URI、ヘッダー、XML で構築できる
- `Enabled`、`Suspended` のレスポンスを正しくパースできる（`Unknown(String)` のパースも検証する）
- 実際の S3 互換サーバー（MinIO / RustFS / kikyo-local）を `shiguredo_container` で起動した統合テストで検証する。`?accelerate` サブリソースに未対応のサーバーがある場合は、そのサーバー名と未対応の根拠を issue に追記した上で、対応しているサーバーでのみ統合テストを行う
- 既存のテストが全て通過すること
- `cargo clippy --workspace --all-targets -- -D warnings` が通過すること
- CHANGES.md の `## develop` に `[ADD]` エントリを追加すること

## AWS S3 API Reference

- GetBucketAccelerateConfiguration: https://docs.aws.amazon.com/AmazonS3/latest/API/API_GetBucketAccelerateConfiguration.html

> Returns the Transfer Acceleration state of a bucket.

- PutBucketAccelerateConfiguration: https://docs.aws.amazon.com/AmazonS3/latest/API/API_PutBucketAccelerateConfiguration.html

> Sets the accelerate configuration of an existing bucket.
