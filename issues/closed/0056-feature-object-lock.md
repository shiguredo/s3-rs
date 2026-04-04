# Object Lock 関連 API の実装

Created: 2026-04-04
Completed: 2026-04-05
Model: Opus 4.6

## 根拠

- 改ざん防止・保持義務のあるデータでは **リテンションとリーガルホールド**が必須になり、バケット既定ロックとオブジェクト単位 API の両方が必要になる。
- aws-sdk-rust と同等の操作を揃えるため（`AGENTS.md` の shiguredo_s3 方針）。

## 概要

WORM 運用のための Object Lock 関連 API を `S3Client` に追加する。

## 優先度

高

## 解決方法

- `GetObjectLegalHoldFluentBuilder` / `PutObjectLegalHoldFluentBuilder` を新規作成
- `GetObjectRetentionFluentBuilder` / `PutObjectRetentionFluentBuilder` を新規作成
- `GetObjectLockConfigurationFluentBuilder` / `PutObjectLockConfigurationFluentBuilder` を新規作成
- `ObjectLockLegalHold`, `ObjectLockRetention`, `DefaultRetention`, `ObjectLockRule`, `ObjectLockConfiguration` 型を `types.rs` に追加
- `PutObjectRetention` に `bypass_governance_retention` ヘッダー対応を追加
- `PutObjectLockConfiguration` に `token` (x-amz-bucket-object-lock-token) ヘッダー対応を追加
- オブジェクト単位の API には `version_id` パラメータを追加
- `S3Client` にファクトリメソッドを追加
