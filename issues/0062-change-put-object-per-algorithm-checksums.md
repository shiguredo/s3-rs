# PutObject / UploadPart のチェックサムを個別フィールド化する

Created: 2026-05-04
Model: Opus 4.7

## 根拠

- 現状の `PutObject` ビルダーは `checksum_algorithm: Option<String>` と `checksum_value: Option<String>` の 2 本立てで、利用者が任意のアルゴリズムと値を文字列で渡す独自設計になっている (`src/api/put_object.rs:28-29`)。
- 一方、aws-sdk-rust では `PutObjectInput` に `checksum_algorithm: Option<ChecksumAlgorithm>` (enum) と、アルゴリズムごとに分かれた `checksum_crc32`, `checksum_crc32_c`, `checksum_crc64_nvme`, `checksum_md5`, `checksum_sha1`, `checksum_sha256`, `checksum_sha512`, `checksum_xxhash128`, `checksum_xxhash3`, `checksum_xxhash64` の各 `Option<String>` フィールドを持つ (`/Users/voluntas/src/aws-sdk-rust/sdk/s3/src/operation/put_object/_put_object_input.rs:74-92`)。
- `UploadPart` も同様の構造 (`/Users/voluntas/src/aws-sdk-rust/sdk/s3/src/operation/upload_part/_upload_part_input.rs:25-43`)。
- 一方、`CopyObject` は `checksum_algorithm` のみを持ち、個別 checksum 入力フィールドを持たない (`/Users/voluntas/src/aws-sdk-rust/sdk/s3/src/operation/copy_object/_copy_object_input.rs:34`)。S3 公式仕様でも CopyObject に `x-amz-checksum-crc32` 等の入力ヘッダーは定義されていないため、CopyObject では本 issue の対象外とする。
- `docs/AWS_SDK_RUST.md` 自身が「`checksum_value` は shiguredo_s3 独自パラメータ」と明記しており、互換性を最優先する方針 (`AGENTS.md`) に反する状態。
- 出力側 (`GetObjectOutput`, `HeadObjectOutput`) は既に `checksum_crc32`, `checksum_crc32c`, `checksum_crc64nvme`, `checksum_sha1`, `checksum_sha256` の個別フィールドを持っており、入出力で表現が非対称になっている。入力側を出力側に揃える。
- `tests/` での `checksum_value` 利用箇所はゼロ。`tests/minio.rs:2482,2498` で `.checksum_algorithm("CRC32C")` 等を指定して自動計算に依存しているのみで、移行コストは小。

## 変更内容

### 1. `PutObjectFluentBuilder` のフィールド変更

`src/api/put_object.rs:28-29` を以下に変更する。

```rust
// 削除
checksum_value: Option<String>,

// 追加 (aws-sdk-rust の PutObjectInput と完全に揃える)
checksum_crc32: Option<String>,
checksum_crc32_c: Option<String>,
checksum_crc64_nvme: Option<String>,
checksum_md5: Option<String>,
checksum_sha1: Option<String>,
checksum_sha256: Option<String>,
checksum_sha512: Option<String>,
checksum_xxhash128: Option<String>,
checksum_xxhash3: Option<String>,
checksum_xxhash64: Option<String>,
```

`checksum_algorithm` は issue 0059 で `Option<ChecksumAlgorithm>` に型化済みとする。

### 2. ビルダーメソッドの変更

```rust
// 削除
pub fn checksum_value(mut self, value: impl Into<String>) -> Self;

// 追加 (10 種すべて、aws-sdk-rust に合わせ impl Into ではなく String を直接受ける)
pub fn checksum_crc32(mut self, input: impl Into<String>) -> Self;
pub fn checksum_crc32_c(mut self, input: impl Into<String>) -> Self;
pub fn checksum_crc64_nvme(mut self, input: impl Into<String>) -> Self;
pub fn checksum_md5(mut self, input: impl Into<String>) -> Self;
pub fn checksum_sha1(mut self, input: impl Into<String>) -> Self;
pub fn checksum_sha256(mut self, input: impl Into<String>) -> Self;
pub fn checksum_sha512(mut self, input: impl Into<String>) -> Self;
pub fn checksum_xxhash128(mut self, input: impl Into<String>) -> Self;
pub fn checksum_xxhash3(mut self, input: impl Into<String>) -> Self;
pub fn checksum_xxhash64(mut self, input: impl Into<String>) -> Self;
```

`String` 型の値は `impl Into<String>` で受ける (これは enum の `impl Into<T>` 問題とは別で、`String` のビルダーは aws-sdk-rust も `impl Into<String>` を採用している)。

### 3. PutObject の自動計算ロジック

`build_request` 内のチェックサム計算ロジック (`src/api/put_object.rs:278-` 付近) は **PutObject の仕様** に従う。

> If the individual checksum value you provide through `x-amz-checksum-algorithm` doesn't match the checksum algorithm you set through `x-amz-sdk-checksum-algorithm`, Amazon S3 fails the request with a `BadDigest` error. ([PutObject API Reference](https://docs.aws.amazon.com/AmazonS3/latest/API/API_PutObject.html))

ルール:

- `checksum_*` 個別フィールドが指定されている場合: その値を直接該当ヘッダー (`x-amz-checksum-{algorithm}`) に設定する。
- `checksum_algorithm` (`x-amz-sdk-checksum-algorithm`) が指定されている場合: そのまま `x-amz-sdk-checksum-algorithm` ヘッダーに設定する。整合性チェックは S3 サーバ側で行われる (一致しない場合 `BadDigest` が返る)。クライアント側で先に弾くことはしない (aws-sdk-rust と同じ挙動)。
- 個別フィールドも `checksum_algorithm` も未指定: 従来通り CRC32 を自動計算する (デフォルト動作)。
- 複数の個別チェックサムフィールドが同時に指定された場合: 全て送信する (S3 サーバが矛盾を検出した場合は 400 を返す)。

### 4. `UploadPart` も個別フィールド化 (ただし PutObject とは挙動が異なる)

- `src/api/upload_part.rs:24-25` の `checksum_value` を削除し、上記と同じ 10 種の個別フィールドを追加する。

UploadPart 専用のルールとして:

> If you provide an individual checksum, Amazon S3 ignores any provided `ChecksumAlgorithm` parameter. ([UploadPart API Reference](https://docs.aws.amazon.com/AmazonS3/latest/API/API_UploadPart.html))

ビルダー実装ルール:

- `checksum_*` 個別フィールドが指定されている場合: 該当ヘッダーに設定し、`checksum_algorithm` (`x-amz-sdk-checksum-algorithm`) ヘッダーは送信しない (S3 が無視するため、明示的に削除する)。クライアント側で `checksum_algorithm` の指定を黙って無視するため、ログ等で警告は不要だが、テストでこの挙動を固定する。
- 個別フィールドが未指定で `checksum_algorithm` のみ指定: 従来通り body から自動計算する。
- いずれも未指定: 従来通り CRC32 を自動計算する。

### 5. CopyObject は本 issue の対象外

CopyObject の入力には個別 checksum ヘッダーが定義されていない (`x-amz-checksum-algorithm` のみ)。`src/api/copy_object.rs` は `checksum_algorithm` の型化 (issue 0059) のみで完結し、個別フィールドの追加はしない。

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

- `src/api/put_object.rs:28-29,206-211,278-` 付近のフィールド・メソッド・自動計算ロジック変更 (10 種の個別フィールド追加)。
- `src/api/upload_part.rs:24-25,82-86,128-` 付近のフィールド・メソッド・自動計算ロジック変更 (10 種の個別フィールド追加、PutObject と異なるルールで `checksum_algorithm` を扱う)。
- `src/api/copy_object.rs` は本 issue の対象外。
- `src/lib.rs` への型公開追加は不要 (個別フィールドは `Option<String>` のため)。
- `tests/` の影響は `checksum_algorithm` の引数を issue 0059 の enum に置き換える程度。

## 依存関係

- 本 issue は issue 0059 (`ChecksumAlgorithm` enum 化) に依存する。

## 優先度

中

## CHANGES.md への記載

- `[CHANGE] PutObject の checksum_value を廃止し checksum_crc32 / checksum_crc32_c / checksum_crc64_nvme / checksum_md5 / checksum_sha1 / checksum_sha256 / checksum_sha512 / checksum_xxhash128 / checksum_xxhash3 / checksum_xxhash64 の個別フィールドに分解する`
- `[CHANGE] UploadPart の checksum_value を廃止し 10 種の個別チェックサムフィールドに分解する。個別 checksum 指定時に checksum_algorithm を無視する S3 仕様に合わせる`
