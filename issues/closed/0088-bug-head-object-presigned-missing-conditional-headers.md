# HeadObject の presigned が条件ヘッダーと partNumber 検証を欠落する

- Priority: High
- Created: 2026-07-07
- Completed: 2026-07-07
- Model: hy3-free
- Branch: feature/fix-head-object-presigned-conditional-headers

## AWS S3 API Reference

- <https://docs.aws.amazon.com/AmazonS3/latest/API/API_HeadObject.html>

> The HEAD operation retrieves metadata from an object without returning the object itself.

リクエスト構文（同ページ）より、HeadObject は以下の条件パラメータをサポートする:

> `If-Match` / `If-Modified-Since` / `If-Unmodified-Since` / `Range` / `x-amz-checksum-mode` およびクエリ `partNumber`

> **partNumber** — Part number of the object being read. This is a positive integer between 1 and 10,000.

## 目的

`head_object` の `presigned` が `build_request` と非対称であり、条件付き HEAD の presigned URL を使って利用者が手動で条件ヘッダーを付与すると署名不一致で 403 になる不具合を解消する。

## 優先度根拠

- 署名不一致により利用者が意図した条件付き HEAD が常に失敗する実害がある。
- `get_object` の `presigned` は条件ヘッダーを署名対象に含めており、同じ HEAD 系 API での非対称は aws-sdk-rust 互換性（AGENTS.md 最優先事項）を損なう。
- 仕様上 HeadObject がこれらのパラメータをサポートしているため、実装は仕様に従うべき。

## 現状

- `src/api/head_object.rs:340-398`（`presigned`）の `extra_headers` は `sse_customer_algorithm` / `sse_customer_key` のみ。
- `build_request`（同:184-258）が付与する `range` / `if-match` / `if-none-match` / `if-modified-since` / `if-unmodified-since` / `x-amz-checksum-mode` が `presigned` では署名対象に含まれない。
- `presigned` の `part_number`（同:352-355）は `pn.to_string()` で範囲検証なしにクエリに放り込まれる。`build_request`（同:230-238）および `get_object::presigned` は `if !(1..=10000).contains(&pn)` で検証している。

## 設計方針

- `get_object::presigned`（`src/api/get_object.rs:488-526`）を参照し、同じ条件ヘッダー群を `head_object::presigned` の `extra_headers` に追加する。
- `part_number` は `build_request` と同一の `1..=10000` 範囲検証を `presigned` にも追加する（重複回避のため `src/api/mod.rs` に `validate_part_number(pn: i32) -> Result<(), Error>` を切り出し、全 API から呼ぶ）。

## 完了条件

- `head_object::presigned` が `build_request` と同一の条件ヘッダーを署名対象に含めること。
- `head_object::presigned` が `part_number` に対し `1..=10000` の範囲検証を行うこと。
- `tests/` または `pbt/` で `head_object::presigned` の署名対象ヘッダーと `part_number` 検証を検証するテストを追加すること。
- `CHANGES.md` の `## develop` セクションに `[FIX]` エントリを記載すること。

## 解決方法

commit `cf179f2` で修正済み。

- `head_object::presigned` の `extra_headers` に `range` / `if-match` / `if-none-match` / `if-modified-since` / `if-unmodified-since` / `x-amz-checksum-mode` を追加し、`build_request` と署名対象を一致させた
- `presigned` の `part_number` に `validate_part_number` による `1..=10000` 範囲検証を追加した
