# ListObjectVersions の実装

Created: 2026-04-04
Completed: 2026-04-05
Model: Opus 4.6

## 根拠

- バージョニング運用では **全バージョンと削除マーカーの列挙**が必要で、`ListObjectsV2` では代替できない。
- aws-sdk-rust と同等の一覧操作を揃えるため（`AGENTS.md` の shiguredo_s3 方針）。

## 概要

バージョニング有効バケット内のオブジェクトバージョンおよび削除マーカーを一覧する API を `S3Client` に追加する。

## 優先度

高

## 解決方法

- `ListObjectVersionsFluentBuilder` を新規作成
- `ListObjectVersionsOutput`, `ObjectVersion`, `DeleteMarkerEntry` 型を `types.rs` に追加
- builder フィールド: `prefix`, `delimiter`, `key_marker`, `version_id_marker`, `max_keys`, `encoding_type`
- output フィールド: `is_truncated`, `next_key_marker`, `next_version_id_marker`, `versions`, `delete_markers`, `common_prefixes`, `name`, `prefix`, `delimiter`, `max_keys`, `key_marker`, `version_id_marker`, `encoding_type`
- `S3Client` にファクトリメソッドを追加
- MinIO 統合テスト `test_list_object_versions` でバージョン一覧と削除マーカーの取得を検証
