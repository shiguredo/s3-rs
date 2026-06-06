# API 共通入力バリデーションの強化

- Priority: High
- Created: 2026-05-25
- Completed: 2026-06-06
- Model: Composer 2.5
- Polished: 2026-06-06
- Branch: feature/fix-api-input-validation

## 目的

全 API 共通の入力検証を強化し、空文字列・範囲外値・矛盾するヘッダー組み合わせを builder 段階で拒否する。

## 優先度根拠

`required()` が `Some("")` を受理する問題は全 API に波及する。S3 到達前の早期失敗は aws-sdk-rust 互換の利用者体験に直結する。

## 現状

確認済みの問題:

| 項目 | 場所 | 影響範囲 |
|------|------|----------|
| `required()` が空文字列を許容 | `src/api/mod.rs:582-584` | 全 API の `bucket` / `key` 等 |
| `ConfigBuilder` が空 region を許容 | `src/client.rs` | 全 API の署名 |
| `copy_source` の先頭 `/` による `//` | 各 API | CopyObject / UploadPartCopy |
| `extract_metadata` がキーを小文字化 | `mod.rs`（aws-sdk-rust は保持） | GetObject / HeadObject |

以下の項目は調査したが、aws-sdk-rust 互換性の観点からバリデーション追加は行わない:
- `content_length` と `body.len()` 不一致 (aws-sdk-rust 非チェック)
- SSE-C キーのみ指定で algorithm なし (aws-sdk-rust 非チェック)
- メタデータキーの不正文字 (aws-sdk-rust 非チェック)

## 設計方針

### `required()` の空文字列チェック

`required()` に `is_empty()` チェックを追加し、空文字列の場合に `Error::InvalidInput` を返す。`trim()` は含めない（S3 の bucket/key 命名規則とは異なるレイヤーのバリデーションのため）。

### `ConfigBuilder` の空 region チェック

`ConfigBuilder::build` で `region` が空文字列の場合に `Error::InvalidInput` を返す。

### `copy_source` の正規化

`copy_source` の先頭 `/` を `strip_prefix('/')` で正規化し、`//` を防ぐ。

### `extract_metadata` のキー小文字化修正

`extract_metadata` がキーを小文字化する問題を修正し、aws-sdk-rust と同じくキーを保持する。後方互換のない変更であるため、CHANGES.md に `[CHANGE]` として記載する。

## AWS S3 API Reference

- DeleteObject (bucket/key 必須): <https://docs.aws.amazon.com/AmazonS3/latest/API/API_DeleteObject.html>
- CopyObject (CopySource): <https://docs.aws.amazon.com/AmazonS3/latest/API/API_CopyObject.html>
  - > The name of the source bucket and the key of the source object, separated by a slash (/).

## 完了条件

- 空 bucket / key / copy_source が `Error::InvalidInput` になる
- 空 region の `ConfigBuilder::build` が `Error::InvalidInput` になる
- `extract_metadata` がキーを小文字化しない
- `tests/test_api_mod.rs` に `required()` の単体テストを追加する

## 解決方法

1. `src/api/mod.rs` の `required()` 関数に空文字列チェック (`is_empty()`) を追加し、空文字列の場合 `Error::InvalidInput` を返すようにした。`trim()` は含めない。
2. `src/client.rs` の `ConfigBuilder::build()` に region 空文字列チェックを追加した。
3. `src/api/copy_object.rs` および `src/api/upload_part_copy.rs` の `copy_source` を `strip_prefix('/')` で正規化し、`//` を防ぐようにした。
4. `src/api/mod.rs` の `extract_metadata()` で、`to_ascii_lowercase()` によるキー小文字化を廃止し、大文字小文字を区別しないプレフィックス照合と元のキーケース保持の両立を実装した。
5. `src/api/mod.rs` の `#[cfg(test)] mod required_tests` に `required()` の単体テスト（非空正常系、None 拒否、空文字拒否、空白のみ許容）を追加した。
