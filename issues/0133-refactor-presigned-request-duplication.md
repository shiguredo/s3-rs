# build_presigned_url の署名関連の重複計算を解消する

- Priority: Low
- Created: 2026-08-06
- Completed: {YYYY-MM-DD}
- Branch: feature/refactor-presigned-duplication
- Polished: {YYYY-MM-DD}

## 目的

`src/api/mod.rs` の `build_presigned_url` 関数に署名関連の重複計算が残っており、片方だけが修正されてもう片方が取り残されると署名不一致による 403 エラーが発生する。共通関数を利用して解消する。

## 現状

`build_presigned_url` 関数に以下の重複・二重計算が残っている:

1. `build_presigned_url` 内の `X-Amz-SignedHeaders` の構築（`signed_headers_value`）が `src/signing.rs` の `build_canonical_headers` 関数の署名対象ヘッダー名の構築と同型で重複している。`compute_presigned_signature` 関数が同じ `headers` に対して `build_canonical_headers` を再実行するため、同じ SignedHeaders 文字列が同じ関数内で 2 度計算されている
2. `build_presigned_url` がカノニカルクエリ文字列を 2 回計算している。`compute_presigned_signature` 関数が内部で `build_canonical_query_string` を実行し、その後に `build_presigned_url` が同じ `query_params` で `build_canonical_query_string` をもう一度実行する

## 設計方針

1. `src/signing.rs` の `build_canonical_headers` を `pub(crate)` に引き上げ、`build_presigned_url` の `X-Amz-SignedHeaders` 構築に利用する
2. カノニカルクエリ文字列の計算を 1 箇所に集約する（例: `compute_presigned_signature` の戻り値を署名とカノニカルクエリ文字列の組にする、または `build_presigned_url` 側で計算した値を署名計算に引き渡す構成にする）
3. 生成される Presigned URL の文字列が変更前後で同一であることを検証するテストを追加する

## 完了条件

- `build_presigned_url` の署名関連の重複計算が解消されていること
- 生成される Presigned URL が変更前後で同一であること（回帰テストで検証可能であること）
- 既存のテストが全て通過すること
- `cargo clippy --workspace --all-targets -- -D warnings` が通過すること
