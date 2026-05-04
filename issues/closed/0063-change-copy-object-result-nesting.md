# CopyObjectOutput を CopyObjectResult ネスト構造に変更する

Created: 2026-05-04
Completed: 2026-05-04
Model: Opus 4.7

## 根拠

- 現状の `CopyObjectOutput` は `e_tag`, `last_modified`, `version_id`, `copy_source_version_id` を直接フィールドとしてフラットに持つ (`src/types.rs:244-253`)。
- 一方、aws-sdk-rust の `CopyObjectOutput` は以下のネスト構造である:
  - `copy_object_result: Option<CopyObjectResult>` ← `e_tag`, `last_modified`, `checksum_crc32`, `checksum_crc32_c`, `checksum_crc64_nvme`, `checksum_sha1`, `checksum_sha256`, `checksum_type` をネスト
  - トップレベルに `expiration`, `version_id`, `copy_source_version_id`, `server_side_encryption`, `sse_customer_algorithm`, `sse_customer_key_md5`, `ssekms_key_id`, `ssekms_encryption_context`, `bucket_key_enabled`, `request_charged`
- これは AWS S3 公式 API のレスポンス XML 構造に厳密に対応している (`<CopyObjectResult>` 要素の中に `<ETag>` と `<LastModified>` が入る)。
- shiguredo_s3 のフラット構造は AWS API 仕様と乖離しており、aws-sdk-rust 互換性も損なわれている。
- `AGENTS.md` / `CLAUDE.md` 「Amazon S3 API の仕様と aws-sdk-rust との互換性を最優先」の方針に整合させる。
- Output が将来 `checksum_*` 等を持つ際 (issue 0064 のフィールド補完) に、ネスト構造に揃っていないと拡張時に再度破壊的変更が必要になる。今のうちに整える。

## 変更内容

### 1. `CopyObjectResult` 型の新設

`src/types.rs` に以下を追加する。

```rust
/// CopyObject の結果の中身
///
/// AWS S3 API のレスポンス XML <CopyObjectResult> 要素に対応する。
#[derive(Debug, Clone)]
pub struct CopyObjectResult {
    pub e_tag: Option<String>,
    pub last_modified: Option<String>,
    pub checksum_crc32: Option<String>,
    pub checksum_crc32_c: Option<String>,
    pub checksum_crc64_nvme: Option<String>,
    pub checksum_sha1: Option<String>,
    pub checksum_sha256: Option<String>,
    pub checksum_type: Option<String>,  // (issue 0064 で型化検討)
}
```

### 2. `CopyObjectOutput` の再構成

`src/types.rs:244-253` を以下に変更する。

```rust
#[derive(Debug)]
pub struct CopyObjectOutput {
    pub copy_object_result: Option<CopyObjectResult>,
    pub copy_source_version_id: Option<String>,
    pub version_id: Option<String>,
    /// 残りのフィールドは issue 0064 で補完する
    /// expiration, server_side_encryption, sse_customer_algorithm,
    /// sse_customer_key_md5, ssekms_key_id, ssekms_encryption_context,
    /// bucket_key_enabled, request_charged
}
```

本 issue ではコア構造の変更 (フラット → ネスト) のみを行い、SSE/checksum 等のフィールド追加は issue 0064 に委ねる。

### 3. パース処理の更新

`src/api/copy_object.rs:382-392` の `parse_response` を更新する。

```rust
pub fn parse_response(response: &super::S3Response) -> Result<CopyObjectOutput, Error> {
    if !response.is_success() {
        return Err(parse_error_response(response));
    }
    check_body_error(response)?;

    let body_text = std::str::from_utf8(&response.body).ok();

    let copy_object_result = body_text.map(|t| CopyObjectResult {
        e_tag: crate::xml::extract_element(t, "ETag"),
        last_modified: crate::xml::extract_element(t, "LastModified"),
        checksum_crc32: crate::xml::extract_element(t, "ChecksumCRC32"),
        checksum_crc32_c: crate::xml::extract_element(t, "ChecksumCRC32C"),
        checksum_crc64_nvme: crate::xml::extract_element(t, "ChecksumCRC64NVME"),
        checksum_sha1: crate::xml::extract_element(t, "ChecksumSHA1"),
        checksum_sha256: crate::xml::extract_element(t, "ChecksumSHA256"),
        checksum_type: crate::xml::extract_element(t, "ChecksumType"),
    });

    Ok(CopyObjectOutput {
        copy_object_result,
        copy_source_version_id: response.get_header("x-amz-copy-source-version-id").map(String::from),
        version_id: response.get_header("x-amz-version-id").map(String::from),
    })
}
```

### 4. 利用箇所の書き換え

旧:
```rust
let output = CopyObjectFluentBuilder::parse_response(&response)?;
println!("etag: {:?}", output.e_tag);
```

新:
```rust
let output = CopyObjectFluentBuilder::parse_response(&response)?;
let result = output.copy_object_result.ok_or(...)?;
println!("etag: {:?}", result.e_tag);
```

### 5. `lib.rs` の `pub use` 追加

```rust
pub use types::{CopyObjectOutput, CopyObjectResult};
```

### 6. `docs/AWS_SDK_RUST.md` の更新

`docs/AWS_SDK_RUST.md:548-558` の `CopyObjectOutput` 対応表を更新し、`copy_object_result` 経由のフィールド一覧に整理する。

## AWS S3 API Reference

- [CopyObject](https://docs.aws.amazon.com/AmazonS3/latest/API/API_CopyObject.html)

  > Response Body: This response XML returns the version ID of the copied object as well as the source object's version ID. If the request specified server-side encryption with Amazon S3 managed keys (SSE-S3), the response includes the `x-amz-server-side-encryption` header.

  レスポンス XML:

  > ```xml
  > <CopyObjectResult>
  >    <LastModified>timestamp</LastModified>
  >    <ETag>string</ETag>
  >    <ChecksumCRC32>string</ChecksumCRC32>
  >    <ChecksumCRC32C>string</ChecksumCRC32C>
  >    <ChecksumCRC64NVME>string</ChecksumCRC64NVME>
  >    <ChecksumSHA1>string</ChecksumSHA1>
  >    <ChecksumSHA256>string</ChecksumSHA256>
  >    <ChecksumType>string</ChecksumType>
  > </CopyObjectResult>
  > ```

  XML 構造として `<CopyObjectResult>` がトップレベル要素で、`<ETag>` 等はその子要素。aws-sdk-rust はこの構造をそのまま `CopyObjectResult` 構造体としてミラーしており、shiguredo_s3 でも同様の構造にすることで API 仕様との整合性が取れる。

- [CopyObjectResult](https://docs.aws.amazon.com/AmazonS3/latest/API/API_CopyObjectResult.html)

  > Container for all response elements. ETag - Returns the ETag of the new object. LastModified - Creation date of the object.

## 影響範囲

- `src/types.rs:244-253` の構造体変更、`CopyObjectResult` 型追加。
- `src/api/copy_object.rs:382-392` のパース処理更新。
- `src/lib.rs` の `pub use` 更新。
- `examples/s3cli`, `tests/minio.rs`, `tests/rustfs.rs` の出力受け取り箇所を `output.copy_object_result.<field>` 形式に書き換え。
- `docs/AWS_SDK_RUST.md` のレスポンスフィールド表を更新。

## 依存関係

- 本 issue は単独で実施可能だが、issue 0059 (`ChecksumAlgorithm` 等の enum 導入) の後に実施することで `checksum_type` 等の型化と整合させやすい。

## 優先度

中

## CHANGES.md への記載

- `[CHANGE] CopyObjectOutput のフラット構造を CopyObjectResult ネスト構造に変更する`
- `[ADD] CopyObjectResult 型を追加する`
- `[ADD] CopyObjectResult に checksum_crc32 / checksum_crc32_c / checksum_crc64_nvme / checksum_sha1 / checksum_sha256 / checksum_type フィールドを追加する`

## 解決方法

### 実施した変更

1. **`src/types.rs` に `CopyObjectResult` 型を新設**
   - `e_tag`, `last_modified` (`Option<SystemTime>`), `checksum_crc32`, `checksum_crc32_c`, `checksum_crc64_nvme`, `checksum_sha1`, `checksum_sha256`, `checksum_type` フィールドを持つ
   - aws-sdk-rust の `aws_sdk_s3::types::CopyObjectResult` と同じ構造

2. **`CopyObjectOutput` の再構成**
   - フラットな `e_tag` / `last_modified` を削除
   - `copy_object_result: Option<CopyObjectResult>` を追加
   - `version_id`, `copy_source_version_id` はトップレベルに残す
   - SSE / `expiration` 等の追加フィールドは issue 0067 で対応予定 (本 issue ではコア構造変更のみ)

3. **`src/api/copy_object.rs` の `parse_response` 更新**
   - XML body から `<CopyObjectResult>` 配下の各フィールド (ETag / LastModified / Checksum*) を抽出
   - `last_modified` は `parse_iso8601` で `SystemTime` に変換
   - body が UTF-8 として読めない場合は `copy_object_result: None`

4. **`src/lib.rs` の `pub use` 更新**
   - `CopyObjectResult` を公開

5. **利用箇所の書き換え**
   - `tests/minio.rs`, `tests/rustfs.rs` の `output.e_tag.is_some()` 等を `output.copy_object_result.as_ref().and_then(|r| r.e_tag.as_ref()).is_some()` に書き換え
   - `UploadPartCopyOutput` (別構造体) はフラットなままなので影響なし

### 検証結果

- `cargo check --workspace --all-targets`: 成功
- `cargo clippy --workspace --all-targets`: 警告ゼロ
- `cargo test --lib`: 28 tests passed
- `cargo test --test minio test_copy_object test_copy_object_metadata_replace`: 2 件 passed
- pre-commit hook (cargo fmt / clippy / test) すべて pass
