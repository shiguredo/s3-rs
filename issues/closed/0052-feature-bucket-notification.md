# GetBucketNotificationConfiguration / PutBucketNotificationConfiguration の実装

Created: 2026-04-04
Completed: 2026-04-05
Model: Opus 4.6

## 根拠

- パイプライン連携・監査で **オブジェクトイベントの通知設定**をコード化する需要がある（宛先は実装依存）。
- aws-sdk-rust と同等のバケットサブリソース操作を揃えるため（`AGENTS.md` の shiguredo_s3 方針）。

## 概要

バケットイベント通知（オブジェクト作成・削除等の通知先）の取得・設定を行う API を `S3Client` に追加する。

## 優先度

高

## 解決方法

- `GetBucketNotificationConfigurationFluentBuilder` / `PutBucketNotificationConfigurationFluentBuilder` を新規作成
- `TopicConfiguration`, `QueueConfiguration`, `LambdaFunctionConfiguration`, `EventBridgeConfiguration`, `NotificationConfigurationFilter`, `S3KeyFilter`, `FilterRule` 型を `types.rs` に追加
- 通知設定は削除 API がなく、空の NotificationConfiguration を PUT して無効化する仕様
- `skip_destination_validation` ヘッダー対応を追加
- `S3Client` にファクトリメソッドを追加
- fuzz ターゲットに `GetBucketNotificationConfigurationFluentBuilder` を追加
