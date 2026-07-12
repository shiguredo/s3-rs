# api/mod.rs の責務を分離する

- Priority: Medium
- Created: 2026-07-12
- Model: Composer 2.5 Fast
- Branch: feature/refactor-api-mod-rs-responsibility-split

## 目的

`src/api/mod.rs` (884 行) に複数の責務が混在しており、保守性・可読性が低下している。責務ごとにファイルを分離する。

## 優先度根拠

型定義、リクエスト構築、ホスト計算、エラー解析、ユーティリティ、テストが 1 ファイルに詰め込まれており、変更時の影響範囲の把握が困難。中長期的な技術的負債として対応が必要。

## 現状

`src/api/mod.rs` に以下の 6 種の責務が混在:

| 責務 | 内容 |
|------|------|
| 型定義 | `PresignedRequest`, `S3Request`, `S3Response` |
| リクエスト構築 | `build_signed_request`, `build_presigned_url` |
| ホスト・パス計算 | `service_host`, `host_for_bucket`, `path_for_key`, `extract_connect_host` 等 |
| バリデーション | `required`, `validate_presign_expires`, `validate_part_number` |
| エラー解析 | `check_body_error`, `parse_error_response`, `head_error_from_status` |
| ユーティリティ | `base64_md5`, `compute_sse_c_key_md5` |

## 設計方針

以下の分離を提案する:

1. `S3Request`, `S3Response`, `PresignedRequest` は利用者が直接 import する公開型であるため `src/types.rs` または `src/request.rs` に移動
2. ホスト・パス計算は `src/endpoint.rs` に分離
3. バリデーション・エラー解析・ユーティリティは `src/util.rs` に分離

## 完了条件

- `src/api/mod.rs` から責務が分離され、ファイルサイズが適切になっていること
- 公開 API の import パスが変更されていないこと（後方互換）
- 既存のテストが全て通過すること
- `cargo clippy --workspace --all-targets -- -D warnings` が通過すること

## 解決方法

1. `src/endpoint.rs` を作成し、ホスト・パス計算関連のコードを移動する
2. `src/request.rs` を作成し、`S3Request`, `S3Response`, `PresignedRequest` を移動する
3. `src/util.rs` を作成し、バリデーション・エラー解析・ユーティリティを移動する
4. `src/api/mod.rs` から移動したコードを削除し、`pub use` で再エクスポートする
5. CHANGES.md の `## develop` の `### misc` にエントリを追加する
