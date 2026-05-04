# PutObject / UploadPart / CopyObject のチェックサムを個別フィールド化する

Created: 2026-05-04
Model: Opus 4.7

## 根拠

- 現状の `PutObject` ビルダーは `checksum_algorithm: Option<String>` と `checksum_value: Option<String>` の 2 本立てで、利用者が任意のアルゴリズムと値を文字列で渡す独自設計になっている (`src/api/put_object.rs:28-29`)。
- 一方、aws-sdk-rust では `PutObjectInput` に `checksum_algorithm: Option<ChecksumAlgorithm>` (enum) と、アルゴリズムごとに分かれた `checksum_crc32`, `checksum_crc32_c`, `checksum_crc64_nvme`, `checksum_sha1`, `checksum_sha256` の各 `Option<String>` フィールドを持つ。
- `UploadPart` / `CopyObject` も同様の構造。
- `docs/AWS_SDK_RUST.md` 自身が「`checksum_value` は shiguredo_s3 独自パラメータ」と明記しており、互換性を最優先する方針 (`AGENTS.md`) に反する状態。
- 出力側 (`GetObjectOutput`, `HeadObjectOutput`) は既に `checksum_crc32`, `checksum_crc32c`, `checksum_crc64nvme`, `checksum_sha1`, `checksum_sha256` の個別フィールドを持っており、入出力で表現が非対称になっている。入力側を出力側に揃える。
- `tests/` での `checksum_value` 利用箇所はゼロ。`tests/minio.rs:2482,2498` で `.checksum_algorithm("CRC32C")` 等を指定して自動計算に依存しているのみで、移行コストは小。

## 変更内容

### 1. `PutObjectFluentBuilder` のフィールド変更

`src/api/put_object.rs:28-29` を以下に変更する。

```rust
// 削除
checksum_value: Option<String>,

// 追加
checksum_crc32: Option<String>,
checksum_crc32_c: Option<String>,
checksum_crc64_nvme: Option<String>,
checksum_sha1: Option<String>,
checksum_sha256: Option<String>,
```

`checksum_algorithm` は issue 0059 で `Option<ChecksumAlgorithm>` に型化済みとする。

### 2. ビルダーメソッドの変更

```rust
// 削除
pub fn checksum_value(mut self, value: impl Into<String>) -> Self;

// 追加
pub fn checksum_crc32(mut self, value: impl Into<String>) -> Self;
pub fn checksum_crc32_c(mut self, value: impl Into<String>) -> Self;
pub fn checksum_crc64_nvme(mut self, value: impl Into<String>) -> Self;
pub fn checksum_sha1(mut self, value: impl Into<String>) -> Self;
pub fn checksum_sha256(mut self, value: impl Into<String>) -> Self;
```

### 3. 自動計算ロジックの維持

`build_request` 内のチェックサム計算ロジック (`src/api/put_object.rs:278-` 付近) は以下のルールで動作する。

- `checksum_*` 個別フィールドが指定されている場合: その値を直接ヘッダーに使う。`checksum_algorithm` の指定があれば一致確認、なければ個別フィールドからアルゴリズムを推定する。
- `checksum_*` 個別フィールドが未指定で `checksum_algorithm` のみ指定: 従来通り body から自動計算する。
- いずれも未指定: 従来通り CRC32 を自動計算 (デフォルト動作)。

### 4. `UploadPart` / `CopyObject` も同様に変更

- `src/api/upload_part.rs:24-25` の `checksum_value` を削除し、個別 5 フィールドを追加する。
- `src/api/copy_object.rs:45` には現状 `checksum_value` フィールドはなく `checksum_algorithm` のみあるが、`CopyObject` でも個別フィールドを追加する (利用シーンは少ないが互換のため)。

### 5. 検証ヘッダーの推定

aws-sdk-rust の挙動に合わせ、複数の個別チェックサムフィールドが同時に指定された場合は全て送信する (S3 サーバが矛盾を検出した場合は 400 を返す)。

## AWS S3 API Reference

- [PutObject](https://docs.aws.amazon.com/AmazonS3/latest/API/API_PutObject.html)

  > x-amz-checksum-crc32 — This header can be used as a data integrity check to verify that the data received is the same data that was originally sent. This header specifies the Base64 encoded, 32-bit CRC32 checksum of the object.

  > x-amz-checksum-crc32c — Base64 encoded, 32-bit CRC32C checksum of the object.

  > x-amz-checksum-crc64nvme — Base64 encoded, 64-bit CRC64NVME checksum of the part.

  > x-amz-checksum-sha1 — Base64 encoded, 160-bit SHA1 checksum.

  > x-amz-checksum-sha256 — Base64 encoded, 256-bit SHA256 checksum.

- [UploadPart](https://docs.aws.amazon.com/AmazonS3/latest/API/API_UploadPart.html)

  > x-amz-checksum-crc32 / x-amz-checksum-crc32c / x-amz-checksum-crc64nvme / x-amz-checksum-sha1 / x-amz-checksum-sha256 — This header specifies the Base64 encoded checksum of the part. If you provide an individual checksum, Amazon S3 ignores any provided ChecksumAlgorithm parameter.

- [Checking object integrity](https://docs.aws.amazon.com/AmazonS3/latest/userguide/checking-object-integrity.html)

  > Object integrity checks help ensure that data uploaded or downloaded from Amazon S3 has not been corrupted or altered in transit. Each checksum algorithm produces a checksum value that S3 uses to verify the data.

  S3 API はアルゴリズムごとに個別ヘッダー (`x-amz-checksum-{algorithm}`) を持っており、aws-sdk-rust はそれをそのまま個別フィールドとしてミラーしている。shiguredo_s3 でも同じ構造に揃えることで仕様との整合性が取れる。

## 影響範囲

- `src/api/put_object.rs:28-29,206-211,278-` 付近のフィールド・メソッド・自動計算ロジック変更。
- `src/api/upload_part.rs:24-25,82-86,128-` 付近のフィールド・メソッド・自動計算ロジック変更。
- `src/api/copy_object.rs:45,209-213,348-352` 付近のフィールド・メソッド変更。
- `src/lib.rs` への型公開追加は不要 (個別フィールドは `Option<String>` のため)。
- `tests/` の影響は `checksum_algorithm` の引数を issue 0059 の enum に置き換える程度。

## 依存関係

- 本 issue は issue 0059 (`ChecksumAlgorithm` enum 化) に依存する。

## 優先度

中

## CHANGES.md への記載

- `[CHANGE] PutObject の checksum_value を廃止し checksum_crc32 / checksum_crc32_c / checksum_crc64_nvme / checksum_sha1 / checksum_sha256 の個別フィールドに分解する`
- `[CHANGE] UploadPart の checksum_value を廃止し個別チェックサムフィールドに分解する`
- `[ADD] CopyObject に checksum_crc32 等の個別チェックサムフィールドを追加する`
