# GetBucketWebsite / PutBucketWebsite / DeleteBucketWebsite の実装

Created: 2026-04-04
Model: Composer 2 Fast

## 根拠

- 静的サイト配信・ドキュメント公開で **ウェブサイトエンドポイント設定**をバケット単位で管理する需要がある。
- aws-sdk-rust と同等のバケットサブリソース操作を揃えるため（`AGENTS.md` の shiguredo_s3 方針）。

## 概要

静的ウェブサイトホスティング（インデックスドキュメント・エラードキュメント等）の取得・設定・無効化を行う API を `S3Client` に追加する。

## 対象 API

| 操作 | 説明 |
|------|------|
| GetBucketWebsite | ウェブサイト設定を取得する |
| PutBucketWebsite | ウェブサイト設定を有効化・更新する |
| DeleteBucketWebsite | ウェブサイト設定を削除する |

## AWS 公式ドキュメント（API リファレンス）

- [GetBucketWebsite](https://docs.aws.amazon.com/AmazonS3/latest/API/API_GetBucketWebsite.html)
- [PutBucketWebsite](https://docs.aws.amazon.com/AmazonS3/latest/API/API_PutBucketWebsite.html)
- [DeleteBucketWebsite](https://docs.aws.amazon.com/AmazonS3/latest/API/API_DeleteBucketWebsite.html)

## 補足

- リクエスト・レスポンスは XML（WebsiteConfiguration）。

## 優先度

高
