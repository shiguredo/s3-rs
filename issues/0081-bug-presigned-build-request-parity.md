# presigned と build_request のパラメータ非対称

- Priority: High
- Created: 2026-05-25
- Model: Composer 2.5
- Polished: 2026-05-31

## 目的

Fluent Builder で設定したパラメータが `presigned` 経路で署名対象に含まれない API がある。aws-sdk-rust 互換を標榜する以上、同一 Builder の `build_request` と `presigned` で同等のパラメータが反映されるべき。

## 優先度根拠

利用者が builder に `acl()` / `metadata()` / 条件付き header を設定して presigned URL を生成すると、設定が静かに無視される。本番で権限・チェックサム・条件付き GET が効かない。

## 現状

| API | build_request にあって presigned にない例 |
|-----|------------------------------------------|
| PutObject | `acl`, `metadata`, `tagging`, `server_side_encryption`, `ssekms_key_id`, `if-match`, `if-none-match`, `content_encoding`, `content_disposition`, `content_language`, `cache_control`, `expires`, `storage_class`, `content_length`, デフォルト CRC32 |
| GetObject | `range`, `if-match`, `if-none-match`, `if-modified-since`, `if-unmodified-since`, `checksum_mode`, `part_number` 範囲検証 |
| CreateMultipartUpload | `checksum_algorithm`, `acl`, `metadata`, `tagging`, `storage_class`, `content_encoding`, `content_disposition`, `content_language`, `cache_control`, `expires`, `server_side_encryption`, `ssekms_key_id` |
| CompleteMultipartUpload | `content-type: application/xml`, `if-match`, `if-none-match`, SSE-C ヘッダー |
| UploadPart | `content_length`, デフォルト CRC32（自動計算のみ、個別 checksum ヘッダーは存在） |

注: PutObject のデフォルト CRC32 未計算は Sans I/O 制約（ボディ不在）として説明可能だが、doc で build との差異を明記する必要がある。

注: HeadObject / DeleteObject / AbortMultipartUpload も `presigned` メソッドを持つが、本 issue のスコープ外とする。同様のパラメータ非対称が発生する可能性があるため、別途確認が必要。

## 設計方針

### presigned へのヘッダー/クエリパラメータの反映

presigned でも builder 設定済み header / query を署名に含める。各 API の `presigned` メソッドで、`build_request` と同じヘッダー/クエリパラメータを `extra_headers` / `extra_queries` に追加する。

### Sans I/O 制約の明示

ボディ依存 CRC32 自動計算は Sans I/O 制約で不可能。doc comment で明示する。

### CompleteMultipartUpload presigned のヘッダー署名

CompleteMultipartUpload presigned に以下のヘッダーを署名対象に含める:
- `content-type: application/xml`
- `if-match` / `if-none-match`
- SSE-C ヘッダー (`sse_customer_algorithm`, `sse_customer_key`, `sse_customer_key_md5`)

### presigned の `part_number` 範囲検証

`build_request` と `presigned` の両方で `part_number` の `1..=10000` 範囲検証を適用する。現在、`get_object.rs` の `presigned` では検証が欠落している。

### GetObject の条件付きヘッダーの presigned 署名

GetObject の `if-match`, `if-none-match`, `if-modified-since`, `if-unmodified-since` を presigned URL の署名対象に含める。aws-sdk-rust も builder に設定された条件付きヘッダーを署名対象に含める。

### ヘッダー / クエリパラメータの分類

各フィールドの分類基準:
- ヘッダー: `acl`, `metadata`, `tagging`, `server_side_encryption`, `ssekms_key_id`, `if-match`, `if-none-match`, `if-modified-since`, `if-unmodified-since`, `range`, `checksum_mode`, `content_encoding`, `content_disposition`, `content_language`, `cache_control`, `expires`, `storage_class`, SSE-C 系
- クエリパラメータ: `part_number`, `response-*` 系

## AWS S3 API Reference

- PutObject (presigned URL): <https://docs.aws.amazon.com/AmazonS3/latest/API/sigv4-query-string-auth.html>
  - > Amazon S3 uses the authorization information in the query string to authenticate the request.
- GetObject: <https://docs.aws.amazon.com/AmazonS3/latest/API/API_GetObject.html>
- CreateMultipartUpload: <https://docs.aws.amazon.com/AmazonS3/latest/API/API_CreateMultipartUpload.html>
- CompleteMultipartUpload: <https://docs.aws.amazon.com/AmazonS3/latest/API/API_CompleteMultipartUpload.html>
- UploadPart: <https://docs.aws.amazon.com/AmazonS3/latest/API/API_UploadPart.html>

## 完了条件

- 上記 API で builder 設定が presigned 署名に反映される
- 意図的な差異は doc comment に記載される
- 将来の再発防止として、共通のヘッダー構築ヘルパーを抽出するか、`build_request` と `presigned` の差分を検出するテストを追加する
- presigned 統合テスト（MinIO）で以下のパラメータを検証する:
  - PutObject: `acl`, `metadata`, `tagging`, `server_side_encryption`, `if-match`, `if-none-match`
  - GetObject: `range`, `if-match`, `if-none-match`, `checksum_mode`, `part_number`
  - CreateMultipartUpload: `checksum_algorithm`, `acl`, `metadata`, `tagging`
  - CompleteMultipartUpload: `content-type`, `if-match`, `if-none-match`, SSE-C
  - UploadPart: `content_length`, 個別 checksum ヘッダー

## 解決方法

1. 各 API の `presigned` を `build_request` と diff し、欠落しているヘッダー/クエリパラメータを `extra_headers` / `extra_queries` に追加する
2. CompleteMultipartUpload の `content-type: application/xml` を presigned に追加する
3. `part_number` の範囲検証を `presigned` にも適用する
4. Sans I/O 制約の差異を doc comment に記載する
5. 統合テスト拡充
