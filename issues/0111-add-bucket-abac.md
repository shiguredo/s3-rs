# Bucket ABAC API を追加する

- Priority: Medium
- Created: 2026-07-31
- Completed: {YYYY-MM-DD}
- Branch: feature/add-bucket-abac
- Polished: 2026-08-02
- Model: GPT-5

## 目的

aws-sdk-rust に存在する GetBucketAbac と PutBucketAbac を追加し、バケットの属性ベースアクセス制御を S3 API 互換に扱えるようにする。

## 現状

src/api/ に ABAC 用の operation、AbacStatus のモデル、XML リクエスト・レスポンス処理が存在しない。

## 設計方針

- GetBucketAbac と PutBucketAbac の builder を追加する。リクエスト URI は `GET /?abac` と `PUT /?abac`
- 型の構成（aws-sdk-rust 互換）:
  - `AbacStatusValue` enum: `Enabled` / `Disabled` / `Unknown(String)` の 3 variant。`#[non_exhaustive]`、`as_str()` / `From<&str>` / `Display` を実装（`src/types.rs` の enum 方針に準拠）
  - `AbacStatus` 構造体: `status: AbacStatusValue` の 1 フィールド
  - `GetBucketAbacOutput`: `abac_status: Option<AbacStatus>`
  - `PutBucketAbacFluentBuilder`: `abac_status` 必須（未指定時は `Error::InvalidInput`）
- PutBucketAbac の Content-MD5 と x-amz-sdk-checksum-algorithm を既存の XML API（`put_bucket_encryption.rs` 等）と同じ方針で処理する
- `expected_bucket_owner` は pending issue 0057 が全 API 横断で管理しているため、本 issue のスコープ外とする（0057 完了時に追加される）
- GetBucketAbac で ABAC 未設定のバケットに対するレスポンス（`Disabled` 返却か空ボディか）は、実装時に AWS 公式ドキュメントまたは実レスポンスで確認する
- ABAC は IAM 属性（タグ）ベースの機能であり、docs/AWS_SDK_RUST.md の「対応予定無し」基準（IAM ベース、S3 互換ストレージで意味が薄い）と緊張関係がある。本 issue では aws-sdk-rust の API 表面互換のために実装する（利用者が aws-sdk-rust へ移行する際に違和感を感じないようにする CODEBASE.md の方針に従う）

### 変更対象ファイル

- `src/api/get_bucket_abac.rs` (新規作成)
- `src/api/put_bucket_abac.rs` (新規作成)
- `src/api/mod.rs` (モジュール宣言 + FluentBuilder の pub use 追加)
- `src/types.rs` (`AbacStatusValue`, `AbacStatus`, `GetBucketAbacOutput` 追加)
- `src/client.rs` (`get_bucket_abac()` / `put_bucket_abac()` メソッド追加)
- `src/lib.rs` (新しい型の公開)
- `tests/test_bucket_abac.rs` (新規作成)

## 完了条件

- `GET /?abac` と `PUT /?abac` のリクエストを正しい URI、ヘッダー、XML で構築できる
- `Enabled` と `Disabled` のレスポンスを正しくパースできる（`Unknown(String)` のパースも検証する）
- 実際の S3 互換サーバー（MinIO / RustFS / kikyo-local）を `shiguredo_container` で起動した統合テストで検証する。`?abac` サブリソースに未対応のサーバーがある場合は、そのサーバー名と未対応の根拠を issue に追記した上で、対応しているサーバーでのみ統合テストを行う
- 既存のテストが全て通過すること
- `cargo clippy --workspace --all-targets -- -D warnings` が通過すること
- CHANGES.md の `## develop` に `[ADD]` エントリを追加すること

## AWS S3 API Reference

- GetBucketAbac: https://docs.aws.amazon.com/AmazonS3/latest/API/API_GetBucketAbac.html

> Returns the attribute-based access control (ABAC) property of the general purpose bucket.

- PutBucketAbac: https://docs.aws.amazon.com/AmazonS3/latest/API/API_PutBucketAbac.html

> Sets the attribute-based access control (ABAC) property of the general purpose bucket.
