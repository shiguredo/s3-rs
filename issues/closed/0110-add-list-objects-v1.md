# ListObjects を追加する

- Priority: High
- Created: 2026-07-31
- Completed: 2026-08-06
- Branch: feature/add-list-objects-v1
- Polished: 2026-08-02
- Model: GPT-5

## 目的

aws-sdk-rust が提供する ListObjects v1 と互換の Sans I/O API を追加し、既存の ListObjectsV2 へ移行できない利用者も同じ API surface で扱えるようにする。

## 現状

src/api/ には ListObjectsV2 は存在するが、ListObjects v1 に対応する builder、入力型、出力型、リクエスト構築、レスポンスパースが存在しない。

## 設計方針

- aws-sdk-rust の ListObjectsInput / ListObjectsOutput と同じフィールド名・型を採用する
- 入力フィールド: `bucket` (必須)、`marker`、`delimiter`、`encoding_type`、`max_keys`、`prefix` を仕様どおりに扱う。`expected_bucket_owner` / `request_payer` は pending issue 0057 が全 API 横断で管理しており、`optional_object_attributes` は issue 0126 が管理しているため、本 issue のスコープ外とする
- ListObjects v1 と v2 の差分:
  - 入力: v1 は `marker` を使う（v2 の `continuation_token` / `start_after` に対応）。v1 は `list-type=2` クエリパラメータを送らない。v1 は `fetch_owner` パラメータを持たない
  - 出力: v1 は `Marker` / `NextMarker` を持つ（v2 の `ContinuationToken` / `NextContinuationToken` / `KeyCount` / `StartAfter` に対応しない）
- ListObjectsOutput のフィールド: `is_truncated`, `marker`, `next_marker`, `contents` (`Vec<Object>`), `name`, `prefix`, `delimiter`, `max_keys`, `common_prefixes` (`Vec<CommonPrefix>`), `encoding_type`, `request_charged`
- ListBucketResult の Contents、CommonPrefixes、NextMarker、EncodingType を既存の共通 XML パース方針で処理する。`request_charged` は `x-amz-request-charged` レスポンスヘッダーから取得する（XML 要素ではない）
- 既存の `src/api/list_objects_v2.rs` の builder パターン（`build_request` / `parse_response` の分離、`required()` による必須検証、`build_signed_request` の呼び出し）を踏襲する。XML パースヘルパー（`extract_xml_objects` / `extract_xml_common_prefixes`）は list_objects_v2.rs のプライベート関数であるため、共通化または複製の判断は実装時に行う
- 実際の S3 互換サーバーを使う統合テストを追加する

### 変更対象ファイル

- `src/api/list_objects_v1.rs` (新規作成)
- `src/api/mod.rs` (モジュール宣言 + FluentBuilder の pub use 追加)
- `src/types.rs` (`ListObjectsOutput` 追加)
- `src/client.rs` (`list_objects()` メソッド追加)
- `src/lib.rs` (`ListObjectsOutput` の公開)
- `tests/test_list_objects_v1.rs` (新規作成)

## 完了条件

- `client.list_objects()` からリクエストを構築できる
- ListObjects v1 のレスポンスを aws-sdk-rust 互換の `ListObjectsOutput` としてパースできる
- ページングに必要な `NextMarker` と本 issue のスコープ内の入力項目（`marker`、`delimiter`、`encoding_type`、`max_keys`、`prefix`）を検証する統合テストが通る
- 既存のテストが全て通過すること
- `cargo clippy --workspace --all-targets -- -D warnings` が通過すること

## AWS S3 API Reference

- ListObjects: https://docs.aws.amazon.com/AmazonS3/latest/API/API_ListObjects.html

> Returns some or all (up to 1,000) of the objects in a bucket.

## 他 issue との依存関係

- 0057（pending: expected_bucket_owner / request_payer）は全 API 横断で管理。本 issue では対応しない
- 0126（operation specific input fields）は `optional_object_attributes` を ListObjectsV2 / ListObjectVersions に追加する。本 issue では対応しない
- 0129（remaining output fields）は `ListObjectsV2Output` の `request_charged` 等を追加する。本 issue の `ListObjectsOutput.request_charged` は独立に追加する

## 解決方法

1. `src/api/list_objects_v1.rs` を新規作成し、`ListObjectsFluentBuilder` を実装した。v2 の builder パターン（`build_request` / `parse_response` の分離、`required()` による必須検証、`build_signed_request` の呼び出し）を踏襲した。クエリパラメータは `prefix` / `delimiter` / `max-keys` / `marker` / `encoding-type` で、v2 と異なり `list-type` を送らない
2. `src/types/output.rs` に `ListObjectsOutput` を追加した（`is_truncated` / `marker` / `next_marker` / `contents` / `name` / `prefix` / `delimiter` / `max_keys` / `common_prefixes` / `encoding_type` / `request_charged`。aws-sdk-rust の `ListObjectsOutput` と同じフィールド名・順序）。`request_charged` は `x-amz-request-charged` レスポンスヘッダーから取得する
3. XML パースヘルパー（`extract_xml_objects` / `extract_xml_common_prefixes`）は v1 → v2 の依存を避けるため `src/api/util.rs` に移動し、v1 / v2 で共有した
4. `Marker` / `NextMarker` は空要素で返ると `Some("")` になるため `None` に正規化した（空 marker によるページネーション無限ループの防止）
5. `src/client.rs` に `list_objects()` メソッド、`src/api.rs` にモジュール宣言と `ListObjectsFluentBuilder` の公開、`src/lib.rs` / `src/types.rs` に `ListObjectsOutput` の公開を追加した
6. `tests/minio.rs` に統合テスト 4 件を追加した（一覧取得と prefix / delimiter 検証、delimiter なしページネーション（最後の Key を marker に使用）、delimiter 付き NextMarker ページネーション、encoding-type=url によるキーの URL エンコード）。完了条件の変更対象ファイル記載（`tests/test_list_objects_v1.rs`）は実装時点の既存構成（`tests/minio.rs` への統合テスト追加）に合わせた

CHANGES.md の `## develop` に `[ADD]` エントリを追加した。
