# presigned と build_request のパラメータ非対称

- Priority: High
- Created: 2026-05-25
- Model: Composer 2.5
- Branch: feature/fix-presigned-build-request-parity

## 目的

Fluent Builder で設定したパラメータが `presigned` 経路で署名対象に含まれない API がある。aws-sdk-rust 互換を標榜する以上、同一 Builder の `build_request` と `presigned` で同等のパラメータが反映されるべき。

## 優先度根拠

利用者が builder に `acl()` / `metadata()` / 条件付き header を設定して presigned URL を生成すると、設定が静かに無視される。本番で権限・チェックサム・条件付き GET が効かない。

## 現状

| API | build_request にあって presigned にない例 |
|-----|------------------------------------------|
| PutObject | acl, metadata, tagging, SSE, if-match, デフォルト CRC32 |
| GetObject | range, 条件付き header, checksum_mode |
| CreateMultipartUpload | checksum_algorithm, acl, metadata, tagging |
| CompleteMultipartUpload | part 検証, Content-Type, if-match |
| UploadPart | デフォルト CRC32 |

PutObject のデフォルト CRC32 未計算は Sans I/O 制約（ボディ不在）として説明可能だが、doc で build との差異を明記する必要がある。

## 設計方針

- presigned でも builder 設定済み header / query を署名に含める
- Sans I/O 制約で不可能なもの（ボディ依存 CRC32 自動計算）は doc comment で明示
- CompleteMultipartUpload presigned に `content-type: application/xml` を署名対象に含める

## AWS S3 API Reference

- PutObject (presigned URL): <https://docs.aws.amazon.com/AmazonS3/latest/API/sigv4-query-string-auth.html>
- GetObject: <https://docs.aws.amazon.com/AmazonS3/latest/API/API_GetObject.html>

> Amazon S3 uses the authorization information in the query string to authenticate the request.

## 完了条件

- 上記 API で builder 設定が presigned 署名に反映される
- 意図的な差異は doc comment に記載される
- presigned 統合テスト（MinIO）で主要パラメータを検証

## 解決方法

1. 各 API の `presigned` を `build_request` と diff し header / query を揃える
2. CompleteMultipartUpload の part 検証を presigned にも適用
3. 統合テスト拡充
