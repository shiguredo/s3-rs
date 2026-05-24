# API 共通入力バリデーションの強化

- Priority: High
- Created: 2026-05-25
- Model: Composer 2.5
- Branch: feature/fix-api-input-validation

## 目的

全 API 共通の入力検証を強化し、空文字列・範囲外値・矛盾するヘッダー組み合わせを builder 段階で拒否する。

## 優先度根拠

`required()` が `Some("")` を受理する問題は全 API に波及する。S3 到達前の早期失敗は aws-sdk-rust 互換の利用者体験に直結する。

## 現状

確認済み:

| 項目 | 場所 |
|------|------|
| `required()` が空文字列を許容 | `src/api/mod.rs:582-584` |
| `ConfigBuilder` が空 region を許容 | `src/client.rs:122-128` |
| `content_length` と `body.len()` 不一致 | `put_object.rs`, `upload_part.rs` |
| SSE-C キーのみ指定（algorithm なし） | 複数 API |
| presigned で `part_number` 範囲未検証 | `get_object.rs`, `head_object.rs` |
| List 系 `max_keys` / `max_uploads` 範囲未検証 | `list_objects_v2.rs` 等 |
| `copy_source` の先頭 `/` による `//` | `copy_object.rs:291`, `upload_part_copy.rs:154` |
| メタデータキーの不正文字 | `put_object.rs` 等 |
| `extract_metadata` がキーを小文字化 | `mod.rs:212-225`（aws-sdk-rust は保持） |

## 設計方針

- `required()` に `trim().is_empty()` チェックを追加
- `content_length` 指定時は `body.len()` と一致を要求
- SSE-C は key と algorithm のペアを必須化
- presigned / build_request で同一バリデーション
- `copy_source` は `strip_prefix('/')` で正規化
- メタデータキーは HTTP トークン規則に沿って検証

## AWS S3 API Reference

- DeleteObject (bucket/key 必須): <https://docs.aws.amazon.com/AmazonS3/latest/API/API_DeleteObject.html>
- CopyObject (CopySource): <https://docs.aws.amazon.com/AmazonS3/latest/API/API_CopyObject.html>

> The name of the source bucket and the key of the source object, separated by a slash (/).

## 完了条件

- 空 bucket / key / copy_source が `InvalidInput` になる
- 上記バリデーションが単体テストまたは統合テストで検証される
- aws-sdk-rust 移行利用者が期待する builder 段階エラーと整合する

## 解決方法

1. `required()` / `ConfigBuilder::build` の強化
2. 各 API の `build_request` / `presigned` に共通ルールを適用
3. `tests/test_api_mod.rs` 等でエラーパスを検証
