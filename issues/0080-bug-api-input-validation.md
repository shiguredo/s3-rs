# API 共通入力バリデーションの強化

- Priority: High
- Created: 2026-05-25
- Model: Composer 2.5
- Polished: 2026-05-31

## 目的

全 API 共通の入力検証を強化し、空文字列・範囲外値・矛盾するヘッダー組み合わせを builder 段階で拒否する。

## 優先度根拠

`required()` が `Some("")` を受理する問題は全 API に波及する。S3 到達前の早期失敗は aws-sdk-rust 互換の利用者体験に直結する。

## 現状

確認済み:

| 項目 | 場所 | 影響範囲 |
|------|------|----------|
| `required()` が空文字列を許容 | `src/api/mod.rs:582-584` | 全 API の `bucket` / `key` 等 |
| `ConfigBuilder` が空 region を許容 | `src/client.rs:122-128` | 全 API の署名 |
| `content_length` と `body.len()` 不一致 | `put_object.rs`, `upload_part.rs` | PutObject / UploadPart |
| SSE-C キーのみ指定（algorithm なし） | `get_object.rs`, `head_object.rs` 等 | GetObject / HeadObject / UploadPartCopy |
| presigned で `part_number` 範囲未検証 | `get_object.rs`, `head_object.rs` | GetObject / HeadObject presigned |
| List 系 `max_keys` / `max_uploads` 範囲未検証 | `list_objects_v2.rs` 等 | ListObjectsV2 / ListMultipartUploads |
| `copy_source` の先頭 `/` による `//` | `copy_object.rs:291`, `upload_part_copy.rs:154` | CopyObject / UploadPartCopy |
| メタデータキーの不正文字 | `put_object.rs` 等 | PutObject / CopyObject / CreateMultipartUpload |
| `extract_metadata` がキーを小文字化 | `mod.rs:212-225`（aws-sdk-rust は保持） | GetObject / HeadObject のメタデータ取得 |

## 設計方針

### `required()` の空文字列チェック

`required()` に `is_empty()` チェックを追加し、空文字列の場合に `Error::InvalidInput` を返す。`trim()` は含めない（S3 の bucket/key 命名規則とは異なるレイヤーのバリデーションのため）。

### `ConfigBuilder` の空 region チェック

`ConfigBuilder::build` で `region` が空文字列の場合に `Error::InvalidInput` を返す。

### `content_length` と `body.len()` の一致チェック

`content_length` 指定時は `body.len()` と一致を要求し、不一致の場合に `Error::InvalidInput` を返す。aws-sdk-rust はこのチェックを行わないため、互換性を考慮し、バリデーションは行わない方針を採用する。

### SSE-C の key と algorithm のペア必須化

SSE-C キーが指定されている場合は algorithm も必須とし、片方のみの指定をエラーにする。aws-sdk-rust もこのバリデーションを行っていないため、S3 サーバ側のエラーに委ねる方針を採用する。

### presigned の `part_number` 範囲検証

AWS S3 API Reference に基づき、`part_number` の範囲を検証する。AWS S3 API では `part_number` は 1 から 10000 の範囲。`build_request` と `presigned` の両方で同一バリデーションを適用する。

### List 系の `max_keys` / `max_uploads` / `max_parts` 範囲検証

AWS S3 API Reference に基づき、`max_keys` / `max_uploads` / `max_parts` の範囲を検証する。対象: ListObjectsV2 / ListMultipartUploads / ListObjectVersions / ListParts。

### `copy_source` の正規化

`copy_source` の先頭 `/` を `strip_prefix('/')` で正規化し、`//` を防ぐ。

### メタデータキーの検証

メタデータキーは HTTP トークン規則に沿って検証する。aws-sdk-rust はメタデータキーの文字種検証を行わないため、互換性を考慮し、バリデーションは行わない方針を採用する。

### `extract_metadata` のキー小文字化

`extract_metadata` がキーを小文字化する問題を修正し、aws-sdk-rust と同じくキーを保持する。

## AWS S3 API Reference

- DeleteObject (bucket/key 必須): <https://docs.aws.amazon.com/AmazonS3/latest/API/API_DeleteObject.html>
- CopyObject (CopySource): <https://docs.aws.amazon.com/AmazonS3/latest/API/API_CopyObject.html>
  - > The name of the source bucket and the key of the source object, separated by a slash (/).
- GetObject (partNumber): <https://docs.aws.amazon.com/AmazonS3/latest/API/API_GetObject.html>
  - > Part number of the object being read. This is a positive integer between 1 and 10,000.
- ListObjectsV2 (MaxKeys): <https://docs.aws.amazon.com/AmazonS3/latest/API/API_ListObjectsV2.html>
  - > Sets the maximum number of keys returned in the response body.
- PutObject (x-amz-server-side-encryption-customer-key): <https://docs.aws.amazon.com/AmazonS3/latest/API/API_PutObject.html>
  - > Specifies the customer-provided encryption key for Amazon S3 to use in encrypting data.

## 完了条件

- 空 bucket / key / copy_source が `InvalidInput` になる
- 上記バリデーションが単体テストまたは統合テストで検証される
- aws-sdk-rust 移行利用者が期待する builder 段階エラーと整合する

## 解決方法

1. `required()` / `ConfigBuilder::build` の強化
2. 各 API の `build_request` / `presigned` に共通ルールを適用
3. `tests/test_api_mod.rs` 等でエラーパスを検証
