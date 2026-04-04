# GetBucketOwnershipControls / PutBucketOwnershipControls / DeleteBucketOwnershipControls の実装

Created: 2026-04-04
Completed: 2026-04-05
Model: Opus 4.6

## 根拠

- **Bucket owner enforced** 等により ACL を無効化し、ポリシー中心で権限を統一する運用が推奨されるため、設定 API が必要になる。
- aws-sdk-rust と同等のバケットサブリソース操作を揃えるため（`AGENTS.md` の shiguredo_s3 方針）。

## 概要

バケットの Object Ownership（オブジェクト所有者の扱い・ACL 無効化等）の取得・設定・削除を行う API を `S3Client` に追加する。

## 優先度

高

## 解決方法

- `GetBucketOwnershipControlsFluentBuilder` / `PutBucketOwnershipControlsFluentBuilder` / `DeleteBucketOwnershipControlsFluentBuilder` を新規作成
- `OwnershipControlsRule` 型を `types.rs` に追加
- OwnershipControls XML は単純な構造のため `for_each_element` でパース
- `S3Client` にファクトリメソッドを追加
