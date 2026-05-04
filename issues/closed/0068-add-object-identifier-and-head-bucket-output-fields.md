# ObjectIdentifier / CompletedPart / HeadBucketOutput にフィールドを追加する

Created: 2026-05-04
Completed: 2026-05-04
Model: Opus 4.7

## 根拠

- issue 0064 で `docs/AWS_SDK_RUST.md` の方針を再分類した結果、`ObjectIdentifier` / `CompletedPart` / `HeadBucketOutput` 等の構造体が aws-sdk-rust と比べてフィールド不足となっていることが判明した。
- `ObjectIdentifier` は aws-sdk-rust では Conditional Delete 対応のため `e_tag`, `last_modified_time`, `size` を持つ。
- `CompletedPart` は aws-sdk-rust では各アルゴリズム別チェックサムフィールドを持つ。
- `HeadBucketOutput` は aws-sdk-rust では `bucket_arn`, `bucket_location_type`, `bucket_location_name`, `access_point_alias` を持つ (S3 互換ストレージで返却されない可能性が高いが、aws-sdk-rust 互換のため定義する)。
- 出力フィールドの追加は後方互換性を持つ。

## 変更内容

### 1. `ObjectIdentifier` (`src/types.rs:280-284`) への追加

S3 Conditional Delete 対応のため:

```rust
pub e_tag: Option<String>,
pub last_modified_time: Option<SystemTime>,  // issue 0060 の方針
pub size: Option<i64>,
```

`ObjectIdentifier` は入力構造体 (`DeleteObjects` の入力に使う) のため、ビルダーメソッドも追加する:

```rust
pub fn e_tag(mut self, input: impl Into<String>) -> Self;
pub fn set_e_tag(mut self, input: Option<String>) -> Self;
pub fn last_modified_time(mut self, input: SystemTime) -> Self;
pub fn set_last_modified_time(mut self, input: Option<SystemTime>) -> Self;
pub fn size(mut self, input: i64) -> Self;
pub fn set_size(mut self, input: Option<i64>) -> Self;
```

シリアライズ側では `<Object>` 要素内にこれらの値を XML として出力する。

### 2. `CompletedPart` (`src/types.rs:286-291`) への追加

```rust
pub checksum_crc32: Option<String>,
pub checksum_crc32_c: Option<String>,
pub checksum_crc64_nvme: Option<String>,
pub checksum_sha1: Option<String>,
pub checksum_sha256: Option<String>,
```

`CompletedPart` も入力構造体 (`CompleteMultipartUpload` の入力に使う) のため、ビルダーメソッドも追加する:

```rust
pub fn checksum_crc32(mut self, input: impl Into<String>) -> Self;
pub fn set_checksum_crc32(mut self, input: Option<String>) -> Self;
// 他 4 種も同様
```

XML シリアライズ側で `<Part>` 要素内に出力する。

### 3. `HeadBucketOutput` (`src/types.rs:374-377`) への追加

```rust
pub bucket_arn: Option<String>,
pub bucket_location_type: Option<String>,  // issue 0059 後続で型化検討
pub bucket_location_name: Option<String>,
pub access_point_alias: Option<bool>,
```

### 4. パース処理の追加

- `src/api/head_bucket.rs` の `parse_response` で各 `x-amz-bucket-*` ヘッダーをパース。
- `CompletedPart` / `ObjectIdentifier` は入力構造体のため、シリアライズ側 (XML 出力) を更新する。

## AWS S3 API Reference

- [DeleteObjects (Conditional)](https://docs.aws.amazon.com/AmazonS3/latest/API/API_ObjectIdentifier.html)

  > ETag - The entity tag of the object.
  > LastModifiedTime - The time at which the object was last modified.
  > Size - The size of the object in bytes.

- [HeadBucket](https://docs.aws.amazon.com/AmazonS3/latest/API/API_HeadBucket.html)

  > Response Headers: x-amz-bucket-region, x-amz-bucket-arn, x-amz-bucket-location-type, x-amz-bucket-location-name, x-amz-access-point-alias.

- [CompleteMultipartUpload](https://docs.aws.amazon.com/AmazonS3/latest/API/API_CompleteMultipartUpload.html)

  `<Part>` 要素には各 checksum サブ要素が含まれる:

  > ```xml
  > <Part>
  >    <ChecksumCRC32>string</ChecksumCRC32>
  >    <ChecksumCRC32C>string</ChecksumCRC32C>
  >    <ChecksumCRC64NVME>string</ChecksumCRC64NVME>
  >    <ChecksumSHA1>string</ChecksumSHA1>
  >    <ChecksumSHA256>string</ChecksumSHA256>
  >    <ETag>string</ETag>
  >    <PartNumber>integer</PartNumber>
  > </Part>
  > ```

## 影響範囲

- `src/types.rs` の `ObjectIdentifier` / `CompletedPart` / `HeadBucketOutput` へのフィールド追加。
- `src/api/delete_objects.rs` の XML シリアライズ拡張 (`<Object>` 要素)。
- `src/api/complete_multipart_upload.rs` の XML シリアライズ拡張 (`<Part>` 要素)。
- `src/api/head_bucket.rs` の `parse_response` 拡張。
- `examples/s3cli`, `tests/` の既存出力アクセスは影響を受けない (フィールド追加のみのため)。

## 依存関係

- 本 issue は issue 0060 (時刻処理) の後に実施する (`SystemTime` 採用のため)。
- issue 0061 (`DeleteObjects` の入力を `Delete` 構造体経由に変更) の後に実施する (`ObjectIdentifier` のビルダー API 設計が確定するため)。

## 優先度

中

## CHANGES.md への記載

- `[ADD] ObjectIdentifier に e_tag / last_modified_time / size を追加する`
- `[ADD] CompletedPart にアルゴリズム別チェックサムフィールドを追加する`
- `[ADD] HeadBucketOutput に bucket_arn / bucket_location_type / bucket_location_name / access_point_alias を追加する`

## 解決方法

### 実施した変更

1. **`src/types.rs` の構造体拡張**
   - `ObjectIdentifier`: `e_tag` / `last_modified_time` (`Option<SystemTime>`) / `size` (`Option<i64>`) を追加
   - `CompletedPart`: `checksum_crc32` / `checksum_crc32_c` / `checksum_crc64_nvme` / `checksum_sha1` / `checksum_sha256` を追加
   - `HeadBucketOutput`: `bucket_arn` / `bucket_location_type` / `bucket_location_name` / `access_point_alias` (bool) を追加

2. **`src/api/delete_objects.rs` の XML シリアライズ拡張**
   - `<Object>` 要素配下に `<ETag>` / `<LastModifiedTime>` (ISO 8601 RFC 3339) / `<Size>` を出力
   - `last_modified_time` の整形は `crate::datetime::civil_from_unix_timestamp` を使用 (秒精度)

3. **`src/api/complete_multipart_upload.rs` の XML シリアライズ拡張**
   - `<Part>` 要素配下に `<ChecksumCRC32>` / `<ChecksumCRC32C>` / `<ChecksumCRC64NVME>` / `<ChecksumSHA1>` / `<ChecksumSHA256>` を出力

4. **`src/api/head_bucket.rs` の `parse_response` 拡張**
   - `x-amz-bucket-arn` / `x-amz-bucket-location-type` / `x-amz-bucket-location-name` を文字列で抽出
   - `x-amz-access-point-alias` (bool) を `parse::<bool>().ok()` でパース

5. **利用箇所への対応**
   - `examples/s3cli/src/ops.rs` / `examples/s3cli/src/upload.rs`
   - `tests/minio.rs` / `tests/rustfs.rs`
   - 既存の `ObjectIdentifier` / `CompletedPart` リテラル構築箇所に追加フィールドを `: None` で初期化 (Python スクリプトで一括処理)

### 検証結果

- `cargo check --workspace --all-targets`: 成功
- `cargo clippy --workspace --all-targets`: 警告ゼロ
- `cargo test --lib`: 28 tests passed
- `cargo test --workspace`: 18 統合テスト全て passed (rustfs)
- pre-commit hook (cargo fmt / clippy / test) すべて pass
