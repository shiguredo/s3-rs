# xml インフラのエラーハンドリングとレスポンス UTF-8 デコード統一

- Priority: High
- Created: 2026-05-25
- Model: Composer 2.5
- Polished: 2026-06-06
- Branch: feature/fix-xml-infra-error-handling

## 目的

`src/xml.rs` とレスポンスパース経路のエラーハンドリングを統一し、破損 XML・非 UTF-8 ボディ・スコープ外要素抽出を正しく扱う。

## 優先度根拠

`extract_element` は 20 以上の API で使用される。パースエラーの `None` 化は、List 系 API を中心に silent data corruption を引き起こす可能性がある。非 UTF-8 成功レスポンスや 10MB 超エラーボディは S3 では稀だが、堅牢性の観点から対応が必要。

## 現状

1. **`extract_element`**: ドキュメント全体の最初の同名タグを返す。List 系で `Prefix` 等が誤抽出されうる
2. **`extract_element`**: ネスト mixed content で `"outerinner"` になる
3. **`for_each_element`**: 終了タグ不一致時に `path_stack` と `depth` が desync（131-144 行）
4. **`parse_s3_error` / `extract_element`**: `Err(_)` → `None`
5. **`complete_multipart_upload.rs` / `copy_object.rs`**: `from_utf8().ok()` で非 UTF-8 を握りつぶし
6. **`check_body_error` (`mod.rs`)** : 10MB 超ボディの `<Error>` を検出しない

## 設計方針

### サブタスク 1: `extract_element` の `Result` 化とスコープ問題の解決

`extract_element` の戻り値を `Option<String>` から `Result<Option<String>, Error>` に変更する。パースエラー時に `Error::InvalidResponse` を返す。「要素が見つからない」場合は `Ok(None)` を返す。全呼び出し側で `?` 演算子で伝播する。

スコープ問題（ドキュメント全体の最初の同名タグを返す）については、`extract_element` にスコープ指定パラメータ（親タグ名）を追加するか、呼び出し側を `for_each_element` でスコープ限定する形に変更する。

### サブタスク 2: `for_each_element` の desync 修正

`for_each_element` の戻り値を `void` から `Result<(), Error>` に変更する。`path_stack` と `depth` が desync する問題を修正し、終了タグ不一致時に `Error::InvalidResponse` を返す。全呼び出し側で `?` で伝播する。

### サブタスク 3: 非 UTF-8 成功レスポンスの `InvalidResponse` 化

`copy_object.rs` と `complete_multipart_upload.rs` の `from_utf8().ok()` を既存の `xml_body_text` 関数（サイズチェック + UTF-8 検証）に置き換える。サブタスク 1 の完了後に実施する。

### サブタスク 4: `check_body_error` の 10MB 超対応

`check_body_error` で 10MB 超ボディの `<Error>` を検出する。先頭 8KB をスキャンして `<Error>` タグの有無を確認する。

### サブタスク 5: `extract_element` の mixed content 修正

`extract_element` のネスト mixed content で `"outerinner"` になる問題を修正する。ネストされた子要素が存在する場合はエラーを返す。

### サブタスク 6: `parse_s3_error` の `Result` 化

`parse_s3_error` の `Err(_)` → `None` 問題を修正し、パースエラー時に `Error::InvalidResponse` を返す。`extract_element` の `Result` 化に依存する。

## AWS S3 API Reference

- ListObjectsV2: <https://docs.aws.amazon.com/AmazonS3/latest/API/API_ListObjectsV2.html>
  - > Prefix: Starts after the specified key in the bucket.
- CopyObject: <https://docs.aws.amazon.com/AmazonS3/latest/API/API_CopyObject.html>
  - > If the object is created by the CopyObject operation, the response includes the CopyObjectResult element.
- CompleteMultipartUpload: <https://docs.aws.amazon.com/AmazonS3/latest/API/API_CompleteMultipartUpload.html>
  - > If the request is successful, the service sends back an HTTP 200 response.

## 完了条件

- List 系 API で `Prefix` 等の誤抽出が解消される
- 非 UTF-8 成功レスポンスが `InvalidResponse` になる（CopyObject / CompleteMultipartUpload）
- 10MB 超 `<Error>` ボディの扱いが方針化され実装される
- fuzz / 単体テストで malformed XML がパニックしない
- `tests/test_xml.rs` に以下のテストケースを追加する:
  - 終了タグ不一致の XML
  - ネスト深さ異常の XML
  - 巨大ボディの XML
  - 非 UTF-8 ボディ
  - mixed content を含む XML
