# GetBucketCors / PutBucketCors / DeleteBucketCors の実装

Created: 2026-04-04
Model: Composer 2 Fast

## 根拠

- ブラウザから直接 S3 互換エンドポイントにアクセスする構成では **CORS 設定が必須**になりやすい。
- aws-sdk-rust と同等のバケットサブリソース操作を揃えるため（`AGENTS.md` の shiguredo_s3 方針）。

## 概要

バケットの CORS（クロスオリジン）設定の取得・設定・削除を行う API を `S3Client` に追加する。

## 対象 API

| 操作 | 説明 |
|------|------|
| GetBucketCors | バケットに設定された CORS ルールを取得する |
| PutBucketCors | CORS ルールを設定する |
| DeleteBucketCors | CORS 設定を削除する |

## AWS 公式ドキュメント（API リファレンス）

- [GetBucketCors](https://docs.aws.amazon.com/AmazonS3/latest/API/API_GetBucketCors.html)
- [PutBucketCors](https://docs.aws.amazon.com/AmazonS3/latest/API/API_PutBucketCors.html)
- [DeleteBucketCors](https://docs.aws.amazon.com/AmazonS3/latest/API/API_DeleteBucketCors.html)

## 補足

- リクエスト・レスポンスボディは XML（CORSConfiguration）。

## 優先度

高
