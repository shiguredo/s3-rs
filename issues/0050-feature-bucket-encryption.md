# GetBucketEncryption / PutBucketEncryption / DeleteBucketEncryption の実装

Created: 2026-04-04
Model: Composer 2 Fast

## 根拠

- コンプライアンス・セキュリティ要件で **バケット既定の SSE** をコードや IaC で管理する需要が高い。
- aws-sdk-rust と同等のバケットサブリソース操作を揃えるため（`AGENTS.md` の shiguredo_s3 方針）。

## 概要

バケットのデフォルト暗号化（サーバーサイド暗号化の既定）の取得・設定・削除を行う API を `S3Client` に追加する。

## 対象 API

| 操作 | 説明 |
|------|------|
| GetBucketEncryption | バケットのデフォルト暗号化設定を取得する |
| PutBucketEncryption | デフォルト暗号化ルールを設定する |
| DeleteBucketEncryption | デフォルト暗号化設定を削除する |

## AWS 公式ドキュメント（API リファレンス）

- [GetBucketEncryption](https://docs.aws.amazon.com/AmazonS3/latest/API/API_GetBucketEncryption.html)
- [PutBucketEncryption](https://docs.aws.amazon.com/AmazonS3/latest/API/API_PutBucketEncryption.html)
- [DeleteBucketEncryption](https://docs.aws.amazon.com/AmazonS3/latest/API/API_DeleteBucketEncryption.html)

## 補足

- リクエスト・レスポンスは XML (`ServerSideEncryptionConfiguration`) でやり取りする。ルール構造は以下の通り:
  - `ServerSideEncryptionRule`
    - `ApplyServerSideEncryptionByDefault`
      - `SSEAlgorithm` (必須): `aws:kms` / `AES256` / `aws:kms:dsse`
      - `KMSMasterKeyID` (任意): SSE-KMS 使用時の KMS キー ID
    - `BucketKeyEnabled` (任意): S3 Bucket Key の有効化

## 優先度

高
