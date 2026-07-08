# CompleteMultipartUpload が checksum をレスポンスヘッダーから読み取る

- Priority: High
- Created: 2026-07-09
- Model: Grok 4.5
- Branch: feature/fix-complete-multipart-upload-checksum-xml

## AWS S3 API Reference

- <https://docs.aws.amazon.com/AmazonS3/latest/API/API_CompleteMultipartUpload.html>

Response Syntax より、checksum はレスポンスヘッダーではなく XML ボディに含まれる:

> ```
> HTTP/1.1 200
> x-amz-expiration: Expiration
> x-amz-server-side-encryption: ServerSideEncryption
> x-amz-version-id: VersionId
> ...
> <CompleteMultipartUploadResult>
>    <Location>string</Location>
>    <Bucket>string</Bucket>
>    <Key>string</Key>
>    <ETag>string</ETag>
>    <ChecksumCRC32>string</ChecksumCRC32>
>    <ChecksumCRC32C>string</ChecksumCRC32C>
>    <ChecksumCRC64NVME>string</ChecksumCRC64NVME>
>    <ChecksumSHA1>string</ChecksumSHA1>
>    <ChecksumSHA256>string</ChecksumSHA256>
>    ...
>    <ChecksumType>string</ChecksumType>
> </CompleteMultipartUploadResult>
> ```

Response Elements より:

> **ChecksumCRC32** — The Base64 encoded, 32-bit CRC32 checksum of the object. This checksum is only present if the checksum was uploaded with the object.

## 目的

CompleteMultipartUpload の成功レスポンスでオブジェクト全体の checksum を正しく取得できるようにする。現状はヘッダーから読むため、仕様どおりの S3 レスポンスでは常に `None` になる。

## 優先度根拠

- チェックサム付き MPU 完了後の整合性検証がクライアント側でできない実害がある
- 同クレートの `CopyObject` は XML から正しく読み取っており、API 間で不整合
- aws-sdk-s3 も body XML から deserialize しており、aws-sdk-rust 互換方針に反する
- `examples/s3cli` が `output.checksum_*` を利用している

## 現状

`src/api/complete_multipart_upload.rs` の `parse_response` で:

- `Location` / `Bucket` / `Key` / `ETag` は `extract_element(body_text, ...)` で XML から取得
- `checksum_crc32` 等は `response.get_header("x-amz-checksum-crc32")` 等でヘッダーから取得

```188:201:src/api/complete_multipart_upload.rs
            checksum_crc32: response
                .get_header("x-amz-checksum-crc32")
                .map(String::from),
            // ...
            checksum_type: response.get_header("x-amz-checksum-type").map(String::from),
```

対比として `CopyObject` は正しい:

```445:450:src/api/copy_object.rs
            checksum_crc32: crate::xml::extract_element(body_text, "ChecksumCRC32")?,
            checksum_crc32_c: crate::xml::extract_element(body_text, "ChecksumCRC32C")?,
            // ...
```

PutObject / GetObject / HeadObject / UploadPart がヘッダーから読むのは各 API の仕様どおりであり、本 issue の対象外。

統合テスト（`tests/minio.rs` / `tests/rustfs.rs`）は CompleteMPU の **output** checksum を assert しておらず、この不具合は CI で検出されない。

## 設計方針

- `CopyObject` と同様に `crate::xml::extract_element(body_text, "ChecksumCRC32")?` 等で XML から取得する
- 対象タグ: `ChecksumCRC32` / `ChecksumCRC32C` / `ChecksumCRC64NVME` / `ChecksumSHA1` / `ChecksumSHA256` / `ChecksumType`
- レスポンスヘッダー側の version_id / expiration / SSE 等は現状どおりヘッダーから読む

## 完了条件

- `CompleteMultipartUploadFluentBuilder::parse_response` が XML ボディから checksum フィールドを取得すること
- チェックサム付き MPU の統合テストまたは単体テストで `output.checksum_*` が `Some` になることを検証すること
- `CHANGES.md` の `## develop` に `[FIX]` を記載すること
