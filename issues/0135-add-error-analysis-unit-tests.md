# エラー解析とユーティリティ関数の単体テストを追加する

- Priority: Medium
- Created: 2026-08-06
- Completed: {YYYY-MM-DD}
- Branch: feature/add-error-analysis-unit-tests
- Polished: {YYYY-MM-DD}

## 目的

`src/api/util.rs` のエラー解析・ユーティリティ関数群に単体テストがなく、エラーパス（破損 XML、サイズ上限超過、非 UTF-8 ボディ等）が統合テストでのみ間接的にしか検証されていない。単体テストで主要エラーパスを固定する。

## 現状

`src/api/util.rs` の以下の関数に単体テストがない:

- `check_body_error` — 2xx レスポンスボディの `<Error>` ルートタグ検出（10MB 以下の XML パース / 10MB 超の先頭 8KB スキャンの 2 経路）
- `xml_body_text` — XML ボディのサイズ上限チェック (10MB) と UTF-8 検証
- `parse_error_response` / `parse_error_response_with_status` — ステータスコードとボディからのエラー構築
- `parse_s3_error_xml` — 破損 XML 時の `UnknownError` フォールバック

`required` / `validate_*` / `head_error_from_status` / `base64_md5` / `compute_sse_c_key_md5` は 0106 で単体テストを追加済み。

## 設計方針

`src/api/util.rs` の `#[cfg(test)] mod tests` に以下を追加する:

1. `xml_body_text` — 10MB 超ボディの `Error::InvalidResponse` 拒否、非 UTF-8 ボディの拒否、正常ボディの通過
2. `check_body_error` — `<Error>` ルートタグを含むボディのエラー検出、含まないボディの通過
3. `parse_error_response_with_status` — ステータスコードとエラー XML からの `Error::S3` 構築
4. `parse_s3_error_xml` — 破損 XML の `UnknownError` フォールバック

## 完了条件

- 上記 4 関数の主要エラーパスが単体テストで検証されていること
- 既存のテストが全て通過すること
- `cargo clippy --workspace --all-targets -- -D warnings` が通過すること
