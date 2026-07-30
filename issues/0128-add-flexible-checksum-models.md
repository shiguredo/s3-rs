# Flexible Checksum の残りを aws-sdk-rust 互換にする

- Priority: High
- Created: 2026-07-31
- Completed: {YYYY-MM-DD}
- Branch: feature/add-flexible-checksum-models
- Polished: {YYYY-MM-DD}
- Model: GPT-5

## 目的

Flexible Checksums の入力・出力・multipart model に残る差分を解消し、MD5、SHA-512、XXHASH 系を含む SDK の checksum fields を失わずに扱えるようにする。

## 現状

src/checksum.rs の自動 header 名解決と checksum 計算は CRC32、CRC32C、CRC64NVME、SHA1、SHA256 に限られる。src/types.rs と各 parse_response には、以下の残件がある。

- GetObjectOutput、HeadObjectOutput、PutObjectOutput、UploadPartOutput、CompleteMultipartUploadOutput の拡張 checksum output
- CreateMultipartUploadOutput、ListPartsOutput の checksum_algorithm / checksum_type
- CopyObjectResult の MD5、SHA-512、XXHASH 系
- Part の checksum fields
- CompletedPart の SHA-512、MD5、XXHASH 系
- CompleteMultipartUpload の top-level checksum input

PutObject と UploadPart の個別 checksum setter 自体は存在するため、既存 API を壊さず残件を補完する。

## 設計方針

- aws-sdk-rust の field 名をそのまま採用する。
- header 名、XML 要素名、base64 値を API ごとに公式仕様へ合わせる。
- 計算できないアルゴリズムを無条件に自動計算するのではなく、明示値の転送と自動計算の責務を分ける。
- ChecksumType の型化は共通 model と整合させ、文字列のまま残さない。

## 完了条件

- 全ての対象 input / output / nested model に checksum fields が存在する。
- build_request、presigned、parse_response が各 checksum header / XML 要素を正しく扱う。
- 既知の checksum vector と実際の S3 互換サーバーを使う統合テストが通る。

## AWS S3 API Reference

- PutObject: https://docs.aws.amazon.com/AmazonS3/latest/API/API_PutObject.html

> This header can be used as a data integrity check to verify that the data received is the same data that was originally sent.

- UploadPart: https://docs.aws.amazon.com/AmazonS3/latest/API/API_UploadPart.html

> If you provide an individual checksum, Amazon S3 ignores any provided ChecksumAlgorithm parameter.

- CompleteMultipartUpload: https://docs.aws.amazon.com/AmazonS3/latest/API/API_CompleteMultipartUpload.html

> Completes a multipart upload by assembling previously uploaded parts.

- GetObject: https://docs.aws.amazon.com/AmazonS3/latest/API/API_GetObject.html

> If you request checksum mode, you must have the kms:Decrypt permission.
