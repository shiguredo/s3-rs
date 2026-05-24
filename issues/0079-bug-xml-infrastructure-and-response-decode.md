# xml インフラのエラーハンドリングとレスポンス UTF-8 デコード統一

- Priority: High
- Created: 2026-05-25
- Model: Composer 2.5
- Branch: feature/fix-xml-infrastructure

## 目的

`src/xml.rs` とレスポンスパース経路のエラーハンドリングを統一し、破損 XML・非 UTF-8 ボディ・スコープ外要素抽出を正しく扱う。

## 優先度根拠

`extract_element` は 20 以上の API で使用される。同名タグの誤抽出やパースエラーの `None` 化は、List 系 API を中心に silent data corruption を引き起こす。

## 現状

1. **`extract_element`**: ドキュメント全体の最初の同名タグを返す。List 系で `Prefix` 等が誤抽出されうる
2. **`extract_element`**: ネスト mixed content で `"outerinner"` になる
3. **`for_each_element`**: 終了タグ不一致時に `path_stack` と `depth` が desync（131-144 行）
4. **`parse_s3_error` / `extract_element`**: `Err(_)` → `None`
5. **`complete_multipart_upload.rs:167` / `copy_object.rs:415`**: `from_utf8().ok()` で非 UTF-8 を握りつぶし
6. **`check_body_error` (`mod.rs:609-619`)**: 10MB 超ボディの `<Error>` を検出しない

## 設計方針

- List 系は `for_each_element` / スコープ付き抽出に統一し、`extract_element` のフラット検索を段階的に廃止
- 内部 XML API を `Result` 化し `InvalidResponse` を返す
- 成功パースも `xml_body_text` 経由でサイズ上限と UTF-8 を統一
- `check_body_error` は先頭数 KB の `<Error>` スキャン等を検討

## AWS S3 API Reference

- ListObjectsV2: <https://docs.aws.amazon.com/AmazonS3/latest/API/API_ListObjectsV2.html>
- CopyObject: <https://docs.aws.amazon.com/AmazonS3/latest/API/API_CopyObject.html>
- CompleteMultipartUpload: <https://docs.aws.amazon.com/AmazonS3/latest/API/API_CompleteMultipartUpload.html>

> If the object is created by the CopyObject operation, the response includes the CopyObjectResult element.

## 完了条件

- List 系 API で `Prefix` 等の誤抽出が解消される
- 非 UTF-8 成功レスポンスが `InvalidResponse` になる（CopyObject / CompleteMultipartUpload）
- 10MB 超 `<Error>` ボディの扱いが方針化され実装される
- fuzz / 単体テストで malformed XML がパニックしない

## 解決方法

1. `xml.rs` の `Result` 化と `for_each_element` の不一致検出
2. `copy_object` / `complete_multipart_upload` を `xml_body_text` に統一
3. `check_body_error` の改善
4. `tests/test_xml.rs` と fuzz 拡充
