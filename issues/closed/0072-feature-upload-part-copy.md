# UploadPartCopy の実装

Created: 2026-04-04
Completed: 2026-04-05
Model: Opus 4.6
Renumbered: 2026-05-04 (旧番号 0047、`closed/0047-feature-object-tagging-api.md` と重複していたため)

## 根拠

- 大容量オブジェクトをクライアントで再アップロードせず、**サーバー側で範囲コピーしてマルチパートを構成**できる。実運用の大ファイル効率化に直結する。
- aws-sdk-rust と同等のマルチパート操作を揃えるため（`AGENTS.md` の shiguredo_s3 方針）。

## 概要

マルチパートアップロードの 1 パートとして、別オブジェクトのバイト範囲をコピーする API を `S3Client` に追加する。

## 優先度

高

## 解決方法

- `UploadPartCopyFluentBuilder` を新規作成
- `UploadPartCopyOutput` 型を `types.rs` に追加（`e_tag`, `last_modified`, `copy_source_version_id`）
- builder に以下のフィールドを実装:
  - `copy_source`, `copy_source_range`
  - 条件付きコピー: `copy_source_if_match`, `copy_source_if_none_match`, `copy_source_if_modified_since`, `copy_source_if_unmodified_since`
  - コピー元 SSE-C: `copy_source_sse_customer_algorithm`, `copy_source_sse_customer_key` (MD5 自動計算)
  - コピー先 SSE-C: `sse_customer_algorithm`, `sse_customer_key` (MD5 自動計算)
- `S3Client` にファクトリメソッドを追加
- MinIO 統合テスト `test_upload_part_copy` でサーバー側コピーのラウンドトリップを検証
