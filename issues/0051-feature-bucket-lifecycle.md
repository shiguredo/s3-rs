# GetBucketLifecycleConfiguration / PutBucketLifecycleConfiguration / DeleteBucketLifecycle の実装

Created: 2026-04-04
Model: Composer 2 Fast

## 根拠

- ログ・バックアップ等の **自動失効や階層移動**は実運用の基本であり、バケットライフサイクル API が必要になる。
- aws-sdk-rust と同等のバケットサブリソース操作を揃えるため（`AGENTS.md` の shiguredo_s3 方針）。

## 概要

バケットのライフサイクル設定（期限切れ・ストレージクラス遷移等）の取得・設定・削除を行う API を `S3Client` に追加する。

## 対象 API

| 操作 | 説明 |
|------|------|
| GetBucketLifecycleConfiguration | ライフサイクルルールを取得する |
| PutBucketLifecycleConfiguration | ライフサイクルルールを設定する |
| DeleteBucketLifecycle | ライフサイクル設定を削除する |

## AWS 公式ドキュメント（API リファレンス）

- [GetBucketLifecycleConfiguration](https://docs.aws.amazon.com/AmazonS3/latest/API/API_GetBucketLifecycleConfiguration.html)
- [PutBucketLifecycleConfiguration](https://docs.aws.amazon.com/AmazonS3/latest/API/API_PutBucketLifecycleConfiguration.html)
- [DeleteBucketLifecycle](https://docs.aws.amazon.com/AmazonS3/latest/API/API_DeleteBucketLifecycle.html)

## 補足

- リクエスト・レスポンスは XML（LifecycleConfiguration）。
- **`transition_default_minimum_object_size`**: `PutBucketLifecycleConfiguration` のリクエストヘッダー `x-amz-transition-default-minimum-object-size` に対応する builder フィールドを含めること。`GetBucketLifecycleConfiguration` のレスポンスにも同名フィールドが返される。aws-sdk-rust では Put の input / Get の output の両方に `transition_default_minimum_object_size` が存在する。

## 優先度

高
