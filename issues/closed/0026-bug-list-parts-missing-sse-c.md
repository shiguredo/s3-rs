# ListParts が SSE-C header を扱えない

## 概要

`ListPartsFluentBuilder` に SSE-C 関連のフィールド / setter がなく、
`build_request` でも対応 header を組み立てていない。
SSE-C で作成した multipart upload に対して `ListParts` を呼ぶと `400 Bad Request` になる。

## 仕様根拠

`ListParts` の request syntax には以下の header が含まれる:

- `x-amz-server-side-encryption-customer-algorithm`
- `x-amz-server-side-encryption-customer-key`
- `x-amz-server-side-encryption-customer-key-MD5`

- https://docs.aws.amazon.com/AmazonS3/latest/API/API_ListParts.html
- https://docs.aws.amazon.com/AmazonS3/latest/userguide/specifying-s3-c-encryption.html

## 修正方針

1. `ListPartsFluentBuilder` に `sse_customer_algorithm` / `sse_customer_key` / `sse_customer_key_md5` フィールドと setter を追加する
2. `build_request` で設定済みの SSE-C header を `extra_headers` として渡す

## 解決方法

1. `ListPartsFluentBuilder` に `sse_customer_algorithm` / `sse_customer_key` / `sse_customer_key_md5` フィールドと setter を追加した
2. `build_request` で SSE-C header を `extra_headers` として `build_signed_request` に渡すようにした
