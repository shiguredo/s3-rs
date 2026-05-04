# PutObjectOutput / UploadPartOutput / CompleteMultipartUploadOutput / CopyObjectOutput に SSE / checksum / request_charged を追加する

Created: 2026-05-04
Completed: 2026-05-04
Model: Opus 4.7

## 根拠

- issue 0064 で `docs/AWS_SDK_RUST.md` の方針を再分類した結果、書き込み系 API の `*Output` が aws-sdk-rust と比べてフィールド不足となっていることが判明した。
- `PutObject` / `UploadPart` / `CompleteMultipartUpload` / `CopyObject` のレスポンスヘッダーには `x-amz-expiration`, `x-amz-server-side-encryption`, `x-amz-server-side-encryption-aws-kms-key-id`, `x-amz-server-side-encryption-bucket-key-enabled`, 各 `x-amz-checksum-*`, `x-amz-checksum-type`, `x-amz-request-charged` 等が含まれており、互換ストレージでも返却される可能性がある。
- 入力側で対応していない SSE/KMS パラメータも、出力側はレスポンスヘッダーをパースするだけで実装コストが軽量で、利用者が結果を確認できる利点がある (issue 0064 で「入力は対応予定無し、出力は対応」と方針整理済み)。
- 個別チェックサムフィールド (issue 0062) と整合させ、書き込み系の出力でも S3 が記録したチェックサムを取得できるようにする。

## 変更内容

### 1. `PutObjectOutput` (`src/types.rs:191-197`) への追加

```rust
pub expiration: Option<String>,
pub server_side_encryption: Option<ServerSideEncryption>,  // issue 0059
pub sse_customer_algorithm: Option<String>,
pub sse_customer_key_md5: Option<String>,
pub ssekms_key_id: Option<String>,
pub bucket_key_enabled: Option<bool>,
pub request_charged: Option<String>,
pub checksum_crc32: Option<String>,
pub checksum_crc32_c: Option<String>,
pub checksum_crc64_nvme: Option<String>,
pub checksum_sha1: Option<String>,
pub checksum_sha256: Option<String>,
pub checksum_type: Option<String>,
```

### 2. `UploadPartOutput` (`src/types.rs:214-218`) への追加

```rust
pub server_side_encryption: Option<ServerSideEncryption>,
pub sse_customer_algorithm: Option<String>,
pub sse_customer_key_md5: Option<String>,
pub ssekms_key_id: Option<String>,
pub bucket_key_enabled: Option<bool>,
pub request_charged: Option<String>,
pub checksum_crc32: Option<String>,
pub checksum_crc32_c: Option<String>,
pub checksum_crc64_nvme: Option<String>,
pub checksum_sha1: Option<String>,
pub checksum_sha256: Option<String>,
```

### 3. `CompleteMultipartUploadOutput` (`src/types.rs:229-238`) への追加

```rust
pub expiration: Option<String>,
pub server_side_encryption: Option<ServerSideEncryption>,
pub ssekms_key_id: Option<String>,
pub bucket_key_enabled: Option<bool>,
pub request_charged: Option<String>,
pub checksum_crc32: Option<String>,
pub checksum_crc32_c: Option<String>,
pub checksum_crc64_nvme: Option<String>,
pub checksum_sha1: Option<String>,
pub checksum_sha256: Option<String>,
pub checksum_type: Option<String>,
```

### 4. `CopyObjectOutput` (issue 0063 で新設したもの) への追加

トップレベルに追加 (`CopyObjectResult` ネスト配下のチェックサムは issue 0063 で対応済み):

```rust
pub expiration: Option<String>,
pub server_side_encryption: Option<ServerSideEncryption>,
pub sse_customer_algorithm: Option<String>,
pub sse_customer_key_md5: Option<String>,
pub ssekms_key_id: Option<String>,
pub ssekms_encryption_context: Option<String>,
pub bucket_key_enabled: Option<bool>,
pub request_charged: Option<String>,
```

### 5. パース処理の追加

各 `parse_response` 内で以下のレスポンスヘッダーをパースする:

- `x-amz-expiration` → `expiration`
- `x-amz-server-side-encryption` → `server_side_encryption`
- `x-amz-server-side-encryption-customer-algorithm` → `sse_customer_algorithm`
- `x-amz-server-side-encryption-customer-key-MD5` → `sse_customer_key_md5`
- `x-amz-server-side-encryption-aws-kms-key-id` → `ssekms_key_id`
- `x-amz-server-side-encryption-context` → `ssekms_encryption_context`
- `x-amz-server-side-encryption-bucket-key-enabled` (bool) → `bucket_key_enabled`
- `x-amz-request-charged` → `request_charged`
- `x-amz-checksum-crc32` 〜 `x-amz-checksum-sha256` → 各 `checksum_*`
- `x-amz-checksum-type` → `checksum_type`

## AWS S3 API Reference

- [PutObject](https://docs.aws.amazon.com/AmazonS3/latest/API/API_PutObject.html)

  > Response Headers: x-amz-expiration, x-amz-server-side-encryption, x-amz-server-side-encryption-aws-kms-key-id, x-amz-server-side-encryption-bucket-key-enabled, x-amz-checksum-crc32, x-amz-checksum-crc32c, x-amz-checksum-crc64nvme, x-amz-checksum-sha1, x-amz-checksum-sha256, x-amz-checksum-type, x-amz-request-charged, x-amz-version-id.

- [UploadPart](https://docs.aws.amazon.com/AmazonS3/latest/API/API_UploadPart.html)

  > Response Headers: x-amz-server-side-encryption, x-amz-server-side-encryption-customer-algorithm, x-amz-server-side-encryption-customer-key-MD5, x-amz-server-side-encryption-aws-kms-key-id, x-amz-server-side-encryption-bucket-key-enabled, x-amz-checksum-crc32, x-amz-checksum-crc32c, x-amz-checksum-crc64nvme, x-amz-checksum-sha1, x-amz-checksum-sha256, x-amz-request-charged.

- [CompleteMultipartUpload](https://docs.aws.amazon.com/AmazonS3/latest/API/API_CompleteMultipartUpload.html)

  > Response Headers: x-amz-expiration, x-amz-server-side-encryption, x-amz-server-side-encryption-aws-kms-key-id, x-amz-server-side-encryption-bucket-key-enabled, x-amz-version-id, x-amz-request-charged, x-amz-checksum-crc32, x-amz-checksum-crc32c, x-amz-checksum-crc64nvme, x-amz-checksum-sha1, x-amz-checksum-sha256, x-amz-checksum-type.

- [CopyObject](https://docs.aws.amazon.com/AmazonS3/latest/API/API_CopyObject.html)

  > Response Headers: x-amz-expiration, x-amz-server-side-encryption, x-amz-server-side-encryption-customer-algorithm, x-amz-server-side-encryption-customer-key-MD5, x-amz-server-side-encryption-aws-kms-key-id, x-amz-server-side-encryption-context, x-amz-server-side-encryption-bucket-key-enabled, x-amz-request-charged.

## 影響範囲

- `src/types.rs` の `PutObjectOutput` / `UploadPartOutput` / `CompleteMultipartUploadOutput` / `CopyObjectOutput` へのフィールド追加。
- `src/api/put_object.rs` / `src/api/upload_part.rs` / `src/api/complete_multipart_upload.rs` / `src/api/copy_object.rs` の `parse_response` 拡張。
- `examples/s3cli`, `tests/` の既存出力アクセスは影響を受けない (フィールド追加のみのため)。

## 依存関係

- 本 issue は issue 0059 (enum 化), issue 0062 (個別チェックサム入力), issue 0063 (CopyObjectResult) の後に実施する。

## 優先度

中

## CHANGES.md への記載

- `[ADD] PutObjectOutput / UploadPartOutput / CompleteMultipartUploadOutput / CopyObjectOutput に SSE / checksum / request_charged 等のフィールドを追加する`

## 解決方法

### 実施した変更

1. **`src/types.rs` の書き込み系 `*Output` にフィールド追加**
   - `PutObjectOutput`: `expiration` / `server_side_encryption` (`ServerSideEncryption`) / `sse_customer_algorithm` / `sse_customer_key_md5` / `ssekms_key_id` / `bucket_key_enabled` (bool) / `request_charged` / `checksum_crc32` 〜 `checksum_sha256` / `checksum_type`
   - `UploadPartOutput`: 上記から `expiration` / `checksum_type` を除いた SSE / 個別 checksum / `request_charged`
   - `CompleteMultipartUploadOutput`: `expiration` / `server_side_encryption` / `ssekms_key_id` / `bucket_key_enabled` / `request_charged` / `checksum_crc32` 〜 `checksum_sha256` / `checksum_type`
   - `CopyObjectOutput` (issue 0063 で新設したもの): トップレベルに `expiration` / `server_side_encryption` / `sse_customer_algorithm` / `sse_customer_key_md5` / `ssekms_key_id` / `ssekms_encryption_context` / `bucket_key_enabled` / `request_charged` を追加

2. **`src/api/put_object.rs` / `upload_part.rs` / `complete_multipart_upload.rs` / `copy_object.rs` の `parse_response` 拡張**
   - レスポンスヘッダー (`x-amz-expiration`, `x-amz-server-side-encryption`, `x-amz-server-side-encryption-aws-kms-key-id`, `x-amz-server-side-encryption-bucket-key-enabled`, 各 `x-amz-checksum-*`, `x-amz-checksum-type`, `x-amz-request-charged`, `x-amz-server-side-encryption-context` 等) からフィールドを抽出
   - `server_side_encryption` は `From<&str>` で `ServerSideEncryption` に変換
   - `bucket_key_enabled` は `parse::<bool>().ok()` でパース失敗を None に落とす

3. **`examples/s3cli/src/upload.rs` の追従**
   - `CompleteMultipartUploadOutput` から `PutObjectOutput` を構築する箇所で、追加フィールドを `output.<field>` から引き継ぎ、SSE-C 関連 (sse_customer_*) は s3cli では非対応のため `None` で初期化

### 検証結果

- `cargo check --workspace --all-targets`: 成功
- `cargo clippy --workspace --all-targets`: 警告ゼロ
- `cargo test --lib`: 28 tests passed
- `cargo test --workspace`: 18 統合テスト全て passed (rustfs)
- pre-commit hook (cargo fmt / clippy / test) すべて pass
