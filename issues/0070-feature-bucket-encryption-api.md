# バケット暗号化 API を追加する

Created: 2026-03-27
Model: Opus 4.6

## 概要

GetBucketEncryption / PutBucketEncryption / DeleteBucketEncryption の 3 API を追加する。

## 根拠

バケットのデフォルト暗号化設定を管理する S3 標準 API。2023 年 1 月以降 S3 は全オブジェクトをデフォルトで SSE-S3 (AES256) 暗号化するようになったが、SSE-KMS への変更や S3 Bucket Key の有効化など、暗号化方式のカスタマイズに必要。

Cloudflare R2 では GetBucketEncryption のみサポート（読み取り専用、常に AES256）。

## 対象 API

### GetBucketEncryption

- `GET /{Bucket}?encryption`
- デフォルト暗号化設定を取得する
- レスポンス: `<ServerSideEncryptionConfiguration>` XML

### PutBucketEncryption

- `PUT /{Bucket}?encryption`
- デフォルト暗号化設定を作成・更新する
- リクエスト: `<ServerSideEncryptionConfiguration>` XML

### DeleteBucketEncryption

- `DELETE /{Bucket}?encryption`
- カスタム暗号化設定を削除し、デフォルト (SSE-S3) に戻す

## 追加する型

- `ServerSideEncryptionRule`: SSEAlgorithm, KMSMasterKeyID, BucketKeyEnabled
- `ServerSideEncryptionConfiguration`: Rule のリスト
- `GetBucketEncryptionOutput`
- `PutBucketEncryptionOutput`
- `DeleteBucketEncryptionOutput`

## 実装上の注意

- XML のネスト構造が深い: `Rule > ApplyServerSideEncryptionByDefault > SSEAlgorithm`
- `for_each_element` は直接の子要素 (depth=2) のみ取得するため、ネストした要素のパースに工夫が必要
- 対処案: `extract_element` でドキュメント全体からフラットに取得する、または XML リーダーで直接パースする専用関数を書く

## 参考

- https://docs.aws.amazon.com/AmazonS3/latest/API/API_GetBucketEncryption.html
- https://docs.aws.amazon.com/AmazonS3/latest/API/API_PutBucketEncryption.html
- https://docs.aws.amazon.com/AmazonS3/latest/API/API_DeleteBucketEncryption.html

## 優先度

中
