# GetBucketReplication / PutBucketReplication / DeleteBucketReplication の実装

Created: 2026-04-04
Model: Composer 2 Fast

## 根拠

- 冗長化・災害対策で **レプリケーションルール**をバケットに設定する需要がある（ルール表現は実装と IAM 前提）。
- aws-sdk-rust と同等のバケットサブリソース操作を揃えるため（`AGENTS.md` の shiguredo_s3 方針）。

## 概要

バケットのレプリケーション設定の取得・設定・削除を行う API を `S3Client` に追加する。

## 対象 API

| 操作 | 説明 |
|------|------|
| GetBucketReplication | レプリケーション設定を取得する |
| PutBucketReplication | レプリケーションルールを設定する |
| DeleteBucketReplication | レプリケーション設定を削除する |

## AWS 公式ドキュメント（API リファレンス）

- [GetBucketReplication](https://docs.aws.amazon.com/AmazonS3/latest/API/API_GetBucketReplication.html)
- [PutBucketReplication](https://docs.aws.amazon.com/AmazonS3/latest/API/API_PutBucketReplication.html)
- [DeleteBucketReplication](https://docs.aws.amazon.com/AmazonS3/latest/API/API_DeleteBucketReplication.html)

## 補足

- リクエスト・レスポンスは XML（ReplicationConfiguration）。S3 互換実装ではルールの表現が AWS と完全一致しない場合がある。

## pending 理由

レプリケーションはバージョニング有効が前提であり、ルール表現は IAM ポリシーとバックエンド実装に強く依存する。S3 互換実装間でのルール互換性も不明確なため、設計判断が必要な issue として pending とする。

## 優先度

高
