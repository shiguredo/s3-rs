# parse::<bool>().ok() による真偽値パースエラー握り潰しを修正する

- Priority: High
- Created: 2026-07-12
- Model: Composer 2.5 Fast
- Branch: feature/fix-parse-bool-error-swallowing

## 目的

18 箇所で使用されている `parse::<bool>().ok()` により、S3 レスポンス XML 内の真偽値フィールドのパース失敗が `None` に握り潰される問題を修正する。

## 優先度根拠

S3 が不正な真偽値（例: `"yes"`, `"1"`, `""`）を返した場合、パースエラーが検出されず `None` として処理される。利用者はフィールド不在と不正値を区別できず、誤った判断につながる。

## 現状

以下の 9 ファイル 18 箇所で `.and_then(|v| v.parse::<bool>().ok())` または `.and_then(|s| s.parse::<bool>().ok())` が使用されている:

- `src/api/get_public_access_block.rs:56,58,60,65` (4箇所)
- `src/api/get_object.rs:408,428` (2箇所)
- `src/api/head_object.rs:327` (1箇所)
- `src/api/put_object.rs:626` (1箇所)
- `src/api/complete_multipart_upload.rs:184` (1箇所)
- `src/api/copy_object.rs:477` (1箇所)
- `src/api/upload_part.rs:369` (1箇所)
- `src/api/list_object_versions.rs:134,180` (2箇所)
- `src/api/list_parts.rs:163` (1箇所)
- `src/api/list_objects_v2.rs:132,177` (2箇所)
- `src/api/head_bucket.rs:62` (1箇所)
- `src/api/list_multipart_uploads.rs:144` (1箇所)

## 設計方針

1. パース失敗時は `None` ではなく `Error::InvalidResponse` を返す
2. 共通のヘルパー関数 `parse_xml_bool(text: &str) -> Result<bool, Error>` を `crate::xml` に追加し、全箇所から使用する

## 完了条件

- 全 18 箇所で `parse::<bool>().ok()` が削除され、適切なエラー処理に置き換えられていること
- 既存のテストが全て通過すること
- 単体テストが追加されていること

## 解決方法

1. `src/xml.rs` に `pub(crate) fn parse_xml_bool(text: &str) -> Result<bool, Error>` を追加する
2. 全 18 箇所を `parse_xml_bool` 呼び出しに置き換える
3. テストを追加する（`tests/test_xml.rs` または `tests/test_parse_bool.rs`）
4. CHANGES.md の `## develop` に `[FIX]` エントリを追加する
