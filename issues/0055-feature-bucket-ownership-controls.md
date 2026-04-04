# GetBucketOwnershipControls / PutBucketOwnershipControls / DeleteBucketOwnershipControls の実装

Created: 2026-04-04
Model: Composer 2 Fast

## 根拠

- **Bucket owner enforced** 等により ACL を無効化し、ポリシー中心で権限を統一する運用が推奨されるため、設定 API が必要になる。
- aws-sdk-rust と同等のバケットサブリソース操作を揃えるため（`AGENTS.md` の shiguredo_s3 方針）。

## 概要

バケットの Object Ownership（オブジェクト所有者の扱い・ACL 無効化等）の取得・設定・削除を行う API を `S3Client` に追加する。

## 対象 API

| 操作 | 説明 |
|------|------|
| GetBucketOwnershipControls | Object Ownership 設定を取得する |
| PutBucketOwnershipControls | Object Ownership ルールを設定する |
| DeleteBucketOwnershipControls | Object Ownership 設定を削除する |

## AWS 公式ドキュメント（API リファレンス）

- [GetBucketOwnershipControls](https://docs.aws.amazon.com/AmazonS3/latest/API/API_GetBucketOwnershipControls.html)
- [PutBucketOwnershipControls](https://docs.aws.amazon.com/AmazonS3/latest/API/API_PutBucketOwnershipControls.html)
- [DeleteBucketOwnershipControls](https://docs.aws.amazon.com/AmazonS3/latest/API/API_DeleteBucketOwnershipControls.html)

## 補足

- リクエスト・レスポンスは XML (`OwnershipControls`)。`ObjectOwnership` の値域は以下の 3 つ:
  - `BucketOwnerEnforced`: ACL を無効化し、バケットオーナーが全オブジェクトを所有する
  - `BucketOwnerPreferred`: バケットオーナーが `bucket-owner-full-control` ACL 付きオブジェクトを所有する
  - `ObjectWriter`: アップロード者がオブジェクトを所有する（従来の挙動）

## 優先度

高
