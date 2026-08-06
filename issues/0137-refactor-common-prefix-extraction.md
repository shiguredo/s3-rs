# CommonPrefixes の XML パース関数の複製を解消する

- Priority: Low
- Created: 2026-08-06
- Completed: {YYYY-MM-DD}
- Branch: feature/refactor-common-prefix-extraction
- Polished: {YYYY-MM-DD}

## 目的

`extract_xml_common_prefixes` 関数が 3 ファイルに複製されており、修正漏れによるバグの温床となっている。0110 (ListObjects v1 追加) で `src/api/util.rs` に共通関数を置いたため、残りの複製も置き換えて 1 箇所に集約する。

## 現状

以下の 3 ファイルに同一の `extract_xml_common_prefixes` が存在する:

- `src/api/util.rs` (0110 で追加した共通関数。`pub(crate)`)
- `src/api/list_object_versions.rs` — `extract_xml_common_prefixes` の private 複製
- `src/api/list_multipart_uploads.rs` — `extract_xml_common_prefixes` の private 複製

3 箇所はコードが完全に同一で、`crate::xml::for_each_element(text, "CommonPrefixes", ...)` で `<CommonPrefixes>` 要素をパースする。

## 設計方針

1. `list_object_versions.rs` と `list_multipart_uploads.rs` の private 複製を削除し、`super::extract_xml_common_prefixes` (util.rs の共通関数、api.rs の再エクスポート経由) を呼び出す形に置き換える
2. 挙動は変更しない (同値変換のみ)

## 完了条件

- `extract_xml_common_prefixes` の複製が解消され、`src/api/util.rs` の 1 箇所のみになっていること
- 既存のテストが全て通過すること
- `cargo clippy --workspace --all-targets -- -D warnings` が通過すること
