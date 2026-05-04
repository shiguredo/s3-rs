# GetObjectOutput / HeadObjectOutput に取得系レスポンスフィールドを追加する

Created: 2026-05-04
Completed: 2026-05-04
Model: Opus 4.7

## 根拠

- issue 0064 で `docs/AWS_SDK_RUST.md` の方針を再分類した結果、`GetObjectOutput` / `HeadObjectOutput` が aws-sdk-rust と比べてフィールド不足となっていることが判明した。
- `GetObject` / `HeadObject` のレスポンスヘッダーには `Content-Encoding`, `Content-Disposition`, `Content-Language`, `Cache-Control`, `Expires`, `x-amz-storage-class`, `x-amz-server-side-encryption`, 各 `x-amz-checksum-*`, `x-amz-mp-parts-count`, `accept-ranges`, `x-amz-replication-status`, `x-amz-restore`, `x-amz-expiration` 等が含まれており、互換ストレージでも返却される可能性がある。
- 入力側で対応していない SSE/KMS パラメータも、出力側はレスポンスヘッダーをパースするだけで実装コストが軽量で、利用者が結果を確認できる利点がある (issue 0064 で「入力は対応予定無し、出力は対応」と方針整理済み)。
- 出力フィールドの追加は後方互換性を持つ。

## 変更内容

### 1. `GetObjectOutput` (`src/types.rs:144-189`) への追加

```rust
pub content_encoding: Option<String>,
pub content_disposition: Option<String>,
pub content_language: Option<String>,
pub cache_control: Option<String>,
pub expires: Option<SystemTime>,  // issue 0060 の方針
pub storage_class: Option<StorageClass>,  // issue 0059 で型化
pub parts_count: Option<i32>,
pub accept_ranges: Option<String>,
pub delete_marker: Option<bool>,  // GetObjectOutput のみ
pub replication_status: Option<String>,  // S3 互換ストレージで意味が薄い場合 None
pub restore: Option<String>,
pub expiration: Option<String>,
pub server_side_encryption: Option<ServerSideEncryption>,  // issue 0059
pub sse_customer_algorithm: Option<String>,
pub sse_customer_key_md5: Option<String>,
pub ssekms_key_id: Option<String>,
pub bucket_key_enabled: Option<bool>,
pub request_charged: Option<String>,  // issue 0059 後続で RequestCharged 型化
pub tag_count: Option<i32>,  // GetObjectOutput のみ
```

### 2. `HeadObjectOutput` への追加

`GetObjectOutput` と同じフィールドを `delete_marker` / `tag_count` を除いて追加する。

```rust
pub content_encoding: Option<String>,
pub content_disposition: Option<String>,
pub content_language: Option<String>,
pub cache_control: Option<String>,
pub expires: Option<SystemTime>,
pub storage_class: Option<StorageClass>,
pub parts_count: Option<i32>,
pub accept_ranges: Option<String>,
pub replication_status: Option<String>,
pub restore: Option<String>,
pub expiration: Option<String>,
pub server_side_encryption: Option<ServerSideEncryption>,
pub sse_customer_algorithm: Option<String>,
pub sse_customer_key_md5: Option<String>,
pub ssekms_key_id: Option<String>,
pub bucket_key_enabled: Option<bool>,
pub request_charged: Option<String>,
```

### 3. パース処理の追加

`src/api/get_object.rs` および `src/api/head_object.rs` の `parse_response` 内で、各レスポンスヘッダーをパースしてフィールドに反映する処理を追加する。

- `Content-Encoding` → `content_encoding`
- `Content-Disposition` → `content_disposition`
- `Content-Language` → `content_language`
- `Cache-Control` → `cache_control`
- `Expires` (IMF-fixdate) → `expires` (SystemTime に変換)
- `x-amz-storage-class` → `storage_class` (`StorageClass::from`)
- `x-amz-mp-parts-count` (i32) → `parts_count`
- `Accept-Ranges` → `accept_ranges`
- `x-amz-delete-marker` (bool) → `delete_marker` (GetObject のみ)
- `x-amz-replication-status` → `replication_status`
- `x-amz-restore` → `restore`
- `x-amz-expiration` → `expiration`
- `x-amz-server-side-encryption` → `server_side_encryption`
- `x-amz-server-side-encryption-customer-algorithm` → `sse_customer_algorithm`
- `x-amz-server-side-encryption-customer-key-MD5` → `sse_customer_key_md5`
- `x-amz-server-side-encryption-aws-kms-key-id` → `ssekms_key_id`
- `x-amz-server-side-encryption-bucket-key-enabled` (bool) → `bucket_key_enabled`
- `x-amz-request-charged` → `request_charged`
- `x-amz-tagging-count` (i32) → `tag_count` (GetObject のみ)

## AWS S3 API Reference

- [GetObject](https://docs.aws.amazon.com/AmazonS3/latest/API/API_GetObject.html)

  > Response Headers: x-amz-server-side-encryption, x-amz-storage-class, x-amz-expiration, x-amz-restore, x-amz-checksum-crc32, x-amz-checksum-crc32c, x-amz-checksum-crc64nvme, x-amz-checksum-sha1, x-amz-checksum-sha256, accept-ranges, x-amz-mp-parts-count, x-amz-tagging-count, x-amz-replication-status, x-amz-delete-marker.

- [HeadObject](https://docs.aws.amazon.com/AmazonS3/latest/API/API_HeadObject.html)

  > Response Headers: Content-Encoding, Content-Language, Content-Disposition, Cache-Control, Expires, x-amz-storage-class, x-amz-server-side-encryption, x-amz-mp-parts-count, x-amz-tagging-count, x-amz-replication-status, x-amz-restore, x-amz-checksum-crc32, x-amz-checksum-crc32c, x-amz-checksum-crc64nvme, x-amz-checksum-sha1, x-amz-checksum-sha256, accept-ranges.

## 影響範囲

- `src/types.rs` の `GetObjectOutput` / `HeadObjectOutput` へのフィールド追加。
- `src/api/get_object.rs` / `src/api/head_object.rs` の `parse_response` 拡張。
- `examples/s3cli`, `tests/` の既存出力アクセスは影響を受けない (フィールド追加のみのため)。

## 依存関係

- 本 issue は issue 0059 (enum 化), issue 0060 (時刻処理) の後に実施する。

## 優先度

中

## CHANGES.md への記載

- `[ADD] GetObjectOutput / HeadObjectOutput に取得系レスポンスフィールドを追加する`

## 解決方法

### 実施した変更

1. **`src/types.rs` の `GetObjectOutput` / `HeadObjectOutput` にフィールド追加**
   - 共通: `content_encoding`, `content_disposition`, `content_language`, `cache_control`, `expires` (Option<SystemTime>), `parts_count` (i32), `accept_ranges`, `replication_status`, `restore`, `expiration`, `server_side_encryption` (ServerSideEncryption enum), `sse_customer_algorithm`, `sse_customer_key_md5`, `ssekms_key_id`, `bucket_key_enabled` (bool), `request_charged`
   - `GetObjectOutput` のみ: `delete_marker` (bool), `tag_count` (i32), `storage_class` (StorageClass enum)
   - `HeadObjectOutput` の `storage_class` は既存のため変更なし

2. **`src/api/get_object.rs` / `src/api/head_object.rs` の `parse_response` 拡張**
   - レスポンスヘッダー (`Content-Encoding`, `Content-Disposition`, ..., `x-amz-mp-parts-count`, `x-amz-tagging-count`, `x-amz-server-side-encryption-bucket-key-enabled` 等) からフィールドを抽出
   - `expires` は `parse_imf_fixdate` で `SystemTime` に変換
   - `storage_class` / `server_side_encryption` は `From<&str>` で enum 化
   - `parts_count` / `tag_count` / `delete_marker` / `bucket_key_enabled` は `parse::<T>().ok()` でパース失敗を None に落とす

### 検証結果

- `cargo check --workspace --all-targets`: 成功
- `cargo clippy --workspace --all-targets`: 警告ゼロ
- `cargo test --lib`: 28 tests passed
- `cargo test --test minio test_object_put_get_head_delete`: passed
- pre-commit hook (cargo fmt / clippy / test) すべて pass
