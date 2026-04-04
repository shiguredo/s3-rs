# GetBucketNotificationConfiguration / PutBucketNotificationConfiguration の実装

Created: 2026-04-04
Model: Composer 2 Fast

## 根拠

- パイプライン連携・監査で **オブジェクトイベントの通知設定**をコード化する需要がある（宛先は実装依存）。
- aws-sdk-rust と同等のバケットサブリソース操作を揃えるため（`AGENTS.md` の shiguredo_s3 方針）。

## 概要

バケットイベント通知（オブジェクト作成・削除等の通知先）の取得・設定を行う API を `S3Client` に追加する。

## 対象 API

| 操作 | 説明 |
|------|------|
| GetBucketNotificationConfiguration | 通知設定を取得する |
| PutBucketNotificationConfiguration | 通知設定を登録・更新する |

## AWS 公式ドキュメント（API リファレンス）

- [GetBucketNotificationConfiguration](https://docs.aws.amazon.com/AmazonS3/latest/API/API_GetBucketNotificationConfiguration.html)
- [PutBucketNotificationConfiguration](https://docs.aws.amazon.com/AmazonS3/latest/API/API_PutBucketNotificationConfiguration.html)

## 補足

- 通知先は SNS / SQS / Lambda / EventBridge 等を XML の NotificationConfiguration で表現する。S3 互換実装ではサポートする宛先種別が異なる場合がある。
- **通知設定の削除**: S3 には `DeleteBucketNotificationConfiguration` API は存在しない。通知を無効化するには空の `NotificationConfiguration` を PUT する。実装・テストではこの挙動を前提とすること。
- **宛先検証の原子性**（参考: サーバー側の挙動）: `PutBucketNotificationConfiguration` はサーバー側で設定適用前に全ての宛先の到達可能性を検証し、失敗すると設定全体を拒否する。これはクライアント SDK の責務ではなくバックエンド依存の挙動である。
- **宛先検証スキップ**: `PutBucketNotificationConfiguration` の builder に `skip_destination_validation` (bool) を含めること。HTTP ヘッダー `x-amz-skip-destination-validation` に対応する。aws-sdk-rust でも同名の builder メソッドが公開されている。

## 優先度

高
