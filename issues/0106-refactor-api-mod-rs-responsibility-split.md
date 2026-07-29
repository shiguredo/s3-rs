# api/mod.rs の責務を分離する

- Priority: Medium
- Created: 2026-07-12
- Model: Composer 2.5 Fast
- Branch: feature/refactor-api-mod-rs-responsibility-split
- Polished: 2026-07-29

## 目的

`src/api/mod.rs` (884 行) に複数の責務が混在しており、保守性・可読性が低下している。責務ごとにファイルを分離する。

## 優先度根拠

型定義、リクエスト構築、ホスト計算、エラー解析、ユーティリティ、テストが 1 ファイルに詰め込まれており、変更時の影響範囲の把握が困難。中長期的な技術的負債として対応が必要。

## 現状

`src/api/mod.rs` に以下の責務が混在:

| 責務 | 内容 | 行番号 |
|------|------|--------|
| 型定義 | `PresignedRequest`, `S3Request`, `S3Response` | 133-227 |
| 設定参照 | `ClientConfig`, `parse_endpoint_scheme`, `impl Client { config_ref }` | 234-280 |
| リクエスト構築 | `build_signed_request`, `build_signed_service_request`, `build_signed_request_inner`, `build_presigned_url` | 291-487 |
| ホスト・パス計算 | `service_host`, `use_path_style_for_bucket`, `host_for_bucket`, `extract_connect_host`, `extract_port`, `parse_port_from_authority`, `path_for_key` | 493-576 |
| バリデーション | `required`, `validate_presign_expires`, `validate_part_number`, `PRESIGN_MIN/MAX_EXPIRES_SECS` | 585-623 |
| エラー解析 | `check_body_error`, `xml_body_text`, `MAX_XML_BODY_SIZE`, `parse_error_response`, `parse_error_response_with_status`, `parse_s3_error_xml`, `head_error_from_status` | 630-718 |
| ユーティリティ | `base64_md5`, `compute_sse_c_key_md5` | 720-738 |
| テスト | `required_tests`, `sans_io_tests` | 740-884 |

## 設計方針

以下の分離を提案する:

1. `S3Request`, `S3Response`, `PresignedRequest` は `src/request.rs` に移動する（`src/types.rs` は 0107 で分割予定のため避け、リクエスト・レスポンスのワイヤー型として独立させる。`S3Response` が `request.rs` に入るのは命名上やや不自然だが、リクエスト・レスポンスの対として 1 ファイルにまとめる方が利用者の認知負荷が低いと判断する）
2. ホスト・パス計算（`service_host` 等）と設定参照（`ClientConfig`, `parse_endpoint_scheme`）は `src/api/endpoint.rs` に分離する（`ClientConfig` を引数に取る関数群と一体で移動する。`impl Client { config_ref }` は `Client` に対する impl ブロックであるため `api/mod.rs` に残す）。移動後、`api/mod.rs` から呼び出される関数（`service_host`, `host_for_bucket`, `path_for_key`, `extract_connect_host`, `extract_port`, `parse_endpoint_scheme`）は `pub(super)` に可視性を引き上げる。endpoint.rs 内部完結の関数（`use_path_style_for_bucket`, `parse_port_from_authority`）はプライベートを維持する
3. バリデーション・エラー解析・ユーティリティは `src/api/util.rs` に分離する。0108 実装後に `add_sse_c_headers` もここに移動する
4. テストは移動先のモジュール内に `#[cfg(test)]` モジュールとして追従する（`required_tests` は `util.rs` へ、`sans_io_tests` は `api/mod.rs` に残す）
5. 分離後、56 個の api サブモジュールの import を更新する。`use super::{...}` 文だけでなく、関数シグネチャや本体内のインライン `super::` 修飾参照（`super::S3Response`, `super::compute_sse_c_key_md5`, `super::xml_body_text` 等）も書き換え対象とする

`pub use` による再エクスポートについて: shiguredo-rust の「re-export は基本的にやらないこと」規約に対し、tests/fuzz が `shiguredo_s3::api::{S3Response, ...}` パスで import しているため（`tests/test_get_bucket_cors.rs:6` 等 7 ファイル + `fuzz/fuzz_targets/fuzz_xml_parse.rs:9`）、`api/mod.rs` に `pub use crate::request::{S3Request, S3Response, PresignedRequest};` を残す。`lib.rs` の既存の `pub use api::{...}` は変更不要。本 issue ではこの例外を正当化する。

## 完了条件

- `src/api/mod.rs` から責務が分離され、リクエスト構築・`impl Client { config_ref }`・モジュール宣言・再エクスポート・`sans_io_tests` のみが残ること
- 外部クレートからの公開 API パス（`shiguredo_s3::S3Request` 等）およびモジュールパス（`shiguredo_s3::api::S3Response` 等）が変更されていないこと
- 既存のテストが全て通過すること
- `cargo clippy --workspace --all-targets -- -D warnings` が通過すること

## 解決方法

1. `src/api/endpoint.rs` を作成し、`ClientConfig`・ホスト・パス計算関連のコードを移動する（`api/mod.rs` に `mod endpoint;` 宣言を追加し、移動した関数の可視性を調整する）
2. `src/request.rs` を作成し、`S3Request`, `S3Response`, `PresignedRequest` を移動する（`lib.rs` に `mod request;` 宣言を追加する）
3. `src/api/util.rs` を作成し、バリデーション・エラー解析・ユーティリティを移動する（`api/mod.rs` に `mod util;` 宣言を追加する）
4. テストを移動先のモジュールに追従させる
5. `api/mod.rs` に `pub use crate::request::{S3Request, S3Response, PresignedRequest};` を残し、56 個のサブモジュールの import（`use super::{...}` とインライン `super::` 修飾参照の両方）を更新する
6. CHANGES.md の `## develop` の `### misc` にエントリを追加する

## 他 issue との依存関係

- 0108（SSE-C ヘッダー重複解消）は `src/api/mod.rs` に `add_sse_c_headers` を追加する設計。0108 を先に実装し、0106 で移動する方が手戻りが少ない
- 0107（types.rs 分割）は `src/types.rs` が対象。本 issue は `src/request.rs` を使うため干渉しない
