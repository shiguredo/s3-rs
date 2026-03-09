# Presigned リクエストが署名対象 header を表現できない

## 概要

`PresignedRequest` 構造体に `headers` フィールドがなく、`build_presigned_url` は `X-Amz-SignedHeaders=host` に固定されている。
そのため builder に設定した SSE-C / Content-Type / checksum 系 header が presigned request から落ちる。

## 影響範囲

- `GetObject::presigned()` — SSE-C / Range が落ちる
- `HeadObject::presigned()` — SSE-C が落ちる
- `PutObject::presigned()` — SSE-C / Content-Type / checksum / metadata 等が落ちる
- `CreateMultipartUpload::presigned()` — SSE-C / Content-Type 等が落ちる
- `UploadPart::presigned()` — SSE-C / checksum が落ちる

## 仕様根拠

SigV4 の query-string auth は `host` と送信する `x-amz-*` header を署名対象に含める前提。
presigned URL で SSE-C を使う場合、3 つの `x-amz-server-side-encryption-customer-*` header を
`X-Amz-SignedHeaders` に含め、リクエスト時にも付与する必要がある。

- https://docs.aws.amazon.com/AmazonS3/latest/API/sigv4-query-string-auth.html
- https://docs.aws.amazon.com/AmazonS3/latest/userguide/specifying-s3-c-encryption.html

## 修正方針

1. `PresignedRequest` に `headers: Vec<(String, String)>` を追加する
2. `build_presigned_url` に `extra_headers: &[(&str, &str)]` 引数を追加する
3. 各 API の `presigned()` で builder に設定済みの header を渡す
4. `X-Amz-SignedHeaders` を `host` 固定ではなく実際の署名対象 header から構築する

## 解決方法

1. `PresignedRequest` に `headers: Vec<(String, String)>` フィールドを追加した
2. `build_presigned_url` に `extra_headers: &[(&str, &str)]` 引数を追加し、署名対象 header を可変にした
3. `X-Amz-SignedHeaders` を `host` 固定ではなく、実際の署名対象 header 名をソートして `;` 区切りで構築するようにした
4. 各 API の `presigned()` メソッドで、builder に設定済みの SSE-C / Content-Type header を `extra_headers` として渡し、`PresignedRequest.headers` にも含めるようにした
   - `GetObject` / `HeadObject`: SSE-C 3 header
   - `PutObject` / `CreateMultipartUpload`: Content-Type + SSE-C 3 header
   - `UploadPart`: SSE-C 3 header
   - `AbortMultipartUpload` / `CompleteMultipartUpload` / `DeleteObject`: header なし (変更なし)
