# presigned と build_request のパラメータ非対称

- Priority: High
- Created: 2026-05-25
- Completed: 2026-06-06
- Model: Composer 2.5
- Polished: 2026-06-06
- Branch: feature/fix-presigned-build-request-parity

## 目的

Fluent Builder で設定したパラメータが `presigned` 経路で署名対象に含まれない API がある。aws-sdk-rust 互換を標榜する以上、同一 Builder の `build_request` と `presigned` で同等のパラメータが反映されるべき。

## 優先度根拠

利用者が builder に `acl()` / `metadata()` / 条件付き header を設定して presigned URL を生成すると、設定が静かに無視される。本番で権限・チェックサム・条件付き GET が効かない。

## 現状

| API | build_request にあって presigned にない例 |
|-----|------------------------------------------|
| PutObject | `acl`, `metadata`, `tagging`, `server_side_encryption`, `ssekms_key_id`, SSE-C, `if-match`, `if-none-match`, content-related |
| GetObject | `range`, `if-match`, `if-none-match`, `if-modified-since`, `if-unmodified-since`, `checksum_mode`, `part_number` 範囲検証 |
| CreateMultipartUpload | SSE-C, `checksum_algorithm`, `acl`, `metadata`, `tagging`, `storage_class`, content-related |
| CompleteMultipartUpload | `content-type: application/xml`, `if-match`, `if-none-match`, SSE-C |
| UploadPart | `content_length` |

注: PutObject のデフォルト CRC32 自動計算は Sans I/O 制約（ボディ不在）により presigned では不可能。doc comment で build との差異を明記する。

## 設計方針

### presigned へのヘッダー/クエリパラメータの反映

presigned でも builder 設定済み header / query を署名に含める。各 API の `presigned` メソッドで `build_request` と同じヘッダー/クエリパラメータを `extra_headers` / `extra_queries` に追加する。

### パラメータの分類

- ヘッダー（`extra_headers` に追加）: `acl`, `metadata`, `tagging`, `server_side_encryption`, `ssekms_key_id`, SSE-C 系, `if-match`, `if-none-match`, `if-modified-since`, `if-unmodified-since`, `range`, `checksum_mode`, content-related headers, `storage_class`, `content-type`
- クエリパラメータ（`extra_queries` に追加）: `part_number`, `response-*` 系

### GetObject の条件付きヘッダー

GetObject の `if-match`, `if-none-match`, `if-modified-since`, `if-unmodified-since` を presigned URL の署名対象に含める。

### part_number 範囲検証

`build_request` と `presigned` の両方で `part_number` の `1..=10000` 範囲検証を適用する。

## AWS S3 API Reference

- PutObject: <https://docs.aws.amazon.com/AmazonS3/latest/API/API_PutObject.html>
- GetObject: <https://docs.aws.amazon.com/AmazonS3/latest/API/API_GetObject.html>
  - > Part number of the object being read. This is a positive integer between 1 and 10,000.
- CreateMultipartUpload: <https://docs.aws.amazon.com/AmazonS3/latest/API/API_CreateMultipartUpload.html>
- CompleteMultipartUpload: <https://docs.aws.amazon.com/AmazonS3/latest/API/API_CompleteMultipartUpload.html>
- UploadPart: <https://docs.aws.amazon.com/AmazonS3/latest/API/API_UploadPart.html>
- SigV4 query-string auth: <https://docs.aws.amazon.com/AmazonS3/latest/API/sigv4-query-string-auth.html>

## 完了条件

- 上記 API で builder 設定が presigned 署名に反映される
- 意図的な差異（CRC32 自動計算不可等）は doc comment に記載される
- presigned 統合テスト（MinIO）で主要パラメータを検証する

## 解決方法

各 API の `presigned()` メソッドに、`build_request()` と同等のヘッダー/クエリパラメータを追加した:

- **PutObject**: `acl`、`metadata`、`tagging`、`server_side_encryption`、`ssekms_key_id`、`if_match`、`if_none_match`、content関連 (`content-encoding`、`content-disposition`、`content-language`、`cache-control`、`expires`)、`content_length`、`storage_class` を `extra_headers` に追加。doc comment に CRC32 自動計算不可を明記。
- **GetObject**: `range`、`if_match`、`if_none_match`、`if_modified_since`、`if_unmodified_since`、`checksum_mode` を `extra_headers` に追加。`part_number` に `1..=10000` 範囲検証を追加。
- **CreateMultipartUpload**: `acl`、`metadata`、`tagging`、`server_side_encryption`、`ssekms_key_id`、`checksum_algorithm`、content関連ヘッダー、`storage_class` を `extra_headers` に追加。
- **CompleteMultipartUpload**: `content-type: application/xml`、`if_match`、`if_none_match`、SSE-C ヘッダーを `extra_headers` に追加。
- **UploadPart**: `content_length` を `extra_headers` に追加。
