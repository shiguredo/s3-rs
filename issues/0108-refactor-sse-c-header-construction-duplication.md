# SSE-C ヘッダー構築ロジックの重複を解消する

- Priority: Medium
- Created: 2026-07-12
- Model: Composer 2.5 Fast
- Branch: feature/refactor-sse-c-header-construction-duplication

## 目的

SSE-C (Server-Side Encryption with Customer-provided keys) のヘッダー構築ロジックが 9 ファイル 16 箇所で完全に重複している。共通ヘルパー関数として抽出する。

## 優先度根拠

同一コードのコピペによる修正漏れ（特に MD5 自動計算ロジックの変更時）のリスクが高い。コードベースの保守性を向上させる。

## 現状

以下の 9 ファイルで以下のパターンが重複:

```rust
let computed_key_md5;
if let Some(ref v) = self.sse_customer_key {
    extra_headers.push(("x-amz-server-side-encryption-customer-key", v.as_str()));
    computed_key_md5 = super::compute_sse_c_key_md5(v)?;
    extra_headers.push(("x-amz-server-side-encryption-customer-key-md5", &computed_key_md5));
}
if let Some(ref v) = self.sse_customer_algorithm {
    extra_headers.push(("x-amz-server-side-encryption-customer-algorithm", v.as_str()));
}
```

出現ファイル: `put_object.rs`, `copy_object.rs`, `create_multipart_upload.rs`, `complete_multipart_upload.rs`, `get_object.rs`, `head_object.rs`, `list_parts.rs`, `upload_part.rs`, `upload_part_copy.rs`

## 設計方針

`src/api/mod.rs` に `pub(crate) fn add_sse_c_headers<'a>(extra_headers: &mut Vec<(&'a str, &'a str)>, sse_customer_algorithm: Option<&'a str>, sse_customer_key: Option<&'a str>) -> Result<Option<String>, Error>` のようなヘルパー関数を追加し、全ファイルから呼び出す。

戻り値の `Option<String>` は MD5 計算結果文字列のライフタイム管理用。

## 完了条件

- SSE-C ヘッダー構築が 1 つのヘルパー関数に集約されていること
- 全 9 ファイルがヘルパー関数を呼び出す形に変更されていること
- 挙動が変更前後で同一であること
- 既存のテストが全て通過すること

## 解決方法

1. `src/api/mod.rs` に `add_sse_c_headers` 関数を実装する
2. 全 9 ファイルの重複コードをヘルパー関数呼び出しに置き換える
3. CHANGES.md の `## develop` の `### misc` にエントリを追加する
