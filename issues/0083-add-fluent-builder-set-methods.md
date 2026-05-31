# Fluent Builder の set_* メソッド追加

- Priority: Medium
- Created: 2026-05-25
- Model: Composer 2.5
- Polished: 2026-05-31

## 目的

aws-sdk-rust の Fluent Builder は全 `Option` フィールドに `set_*` バリアント（`Option` を直接受ける）を提供する。shiguredo_s3 では enum 化対象に限って `set_*` があり、String / SystemTime / bool 系が大量に欠落している。

## 優先度根拠

移行利用者は `builder.set_foo(None)` パターンを前提にコードを書く。欠落はコンパイルエラーではなく API 再学習コストになる。

## 現状

CHANGES.md L92「全 enum 化対象のビルダーに set_* バリアントを追加」は部分的にのみ達成。現在、enum 化対象でない String 系フィールドにも `set_*` が既に存在する（`set_checksum_crc32: Option<String>` 等）。

欠落例:

- `PutObjectFluentBuilder`: `set_body`, `set_content_type`, `set_if_match`, `set_if_none_match`, `set_content_length`, `set_tagging`, `set_metadata` 等
- `GetObjectFluentBuilder`: `set_range`, `set_part_number`, `set_checksum_mode` 等（注: `set_if_match`, `set_if_none_match`, `set_if_modified_since`, `set_if_unmodified_since` は既に存在）
- `CompleteMultipartUploadFluentBuilder`: `set_if_match`, `set_if_none_match`, `set_sse_customer_algorithm`, `set_sse_customer_key` 等の `Option<String>` フィールドに `set_*` なし
- `ConfigBuilder`: `set_region`, `set_credentials_provider` 等 0 件
- PutBucket 系: `set_cors_configuration`, `set_rules` 等

## 設計方針

### `set_*` のシグネチャ

各 Fluent Builder の `Option<T>` フィールドごとに `set_*` を追加する。型ごとの設計:

- `String` 系: `set_foo(self, input: Option<String>) -> Self`
- `i64` 系: `set_foo(self, input: Option<i64>) -> Self`
- `bool` 系: `set_foo(self, input: Option<bool>) -> Self`
- `SystemTime` 系: `set_foo(self, input: Option<SystemTime>) -> Self`
- `Vec<u8>` 系 (body): `set_body(self, input: Option<Vec<u8>>) -> Self`
- enum 系: 既存の `set_*` を維持

### `set_metadata` の設計

`set_metadata` のシグネチャは `set_metadata(self, input: Option<HashMap<String, String>>) -> Self` とする。aws-sdk-rust の `set_metadata` は `Option<HashMap<String, String>>` を受け取るため、互換性のために `Option<HashMap>` を受け取る設計にする。内部で `Vec<(String, String)>` に変換する。

### `ConfigBuilder` の `set_*`

`ConfigBuilder` にも aws-sdk-rust 同等の `set_*` を追加する。`set_region(self, input: Option<String>) -> Self`、`set_credentials_provider(self, input: Option<Credentials>) -> Self` 等。`ConfigBuilder` は `build()` で `Result<Config, Error>` を返す非 Fluent Builder だが、`set_*` の追加は Fluent Builder のパターンと同様に行う。

### スコープ

本 issue のスコープは Fluent Builder（`src/api/` 配下）と `ConfigBuilder`（`src/client.rs`）のみとする。types.rs の非 Fluent Builder（`DeleteBuilder`, `TaggingBuilder`, `CorsRuleBuilder` 等）は対象外。根拠: 非 Fluent Builder は `build()` メソッドを通じて値を設定する設計であり、`set_*` は Fluent Builder のパターン。`ConfigBuilder` は `build()` で `Result<Config, Error>` を返すが、aws-sdk-rust 互換のために `set_*` を追加する。

### 既存 `foo(value)` メソッドの維持

既存 `foo(value)` メソッドは維持する（後方互換）。

## AWS S3 API Reference

- PutObject: <https://docs.aws.amazon.com/AmazonS3/latest/API/API_PutObject.html>
- GetObject: <https://docs.aws.amazon.com/AmazonS3/latest/API/API_GetObject.html>
- CopyObject: <https://docs.aws.amazon.com/AmazonS3/latest/API/API_CopyObject.html>
- CompleteMultipartUpload: <https://docs.aws.amazon.com/AmazonS3/latest/API/API_CompleteMultipartUpload.html>
- HeadObject: <https://docs.aws.amazon.com/AmazonS3/latest/API/API_HeadObject.html>
- DeleteObject: <https://docs.aws.amazon.com/AmazonS3/latest/API/API_DeleteObject.html>
- CreateMultipartUpload: <https://docs.aws.amazon.com/AmazonS3/latest/API/API_CreateMultipartUpload.html>
- UploadPart: <https://docs.aws.amazon.com/AmazonS3/latest/API/API_UploadPart.html>

## 完了条件

- 全 Fluent Builder（Put/Get/Copy/CompleteMultipartUpload/Config/Head/Delete/CreateMultipartUpload/UploadPart 等）に aws-sdk-rust 相当の `set_*` が揃う
- 命名が aws-sdk-rust と一致する
- `set_foo(Some(v))` と `foo(v)` が等価であることを PBT で検証する（`build_request` 経由でリクエストヘッダー/ボディを比較）
- `set_foo(None)` でフィールドがクリアされることを PBT で検証する（`build_request` 経由でヘッダーが出力されないことを確認）
- 既存テストが通る

## 解決方法

1. aws-sdk-rust ソースで各 operation builder の `set_*` 一覧を取得
2. マクロで `Option<T>` フィールドの `set_foo(Option<T>)` を自動生成する（`String`, `i64`, `bool`, `SystemTime`, enum 系）。Vec 系フィールド（`body`, `metadata`）は手動実装する
3. PBT で `set_foo(Some(v))` と `foo(v)` の等価性と `set_foo(None)` のクリア動作を検証
4. コンパイル確認
