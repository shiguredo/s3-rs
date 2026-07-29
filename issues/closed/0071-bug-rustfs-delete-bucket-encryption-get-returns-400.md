# RustFS が DeleteBucketEncryption 後の GetBucketEncryption で 400 を返す

Created: 2026-03-27
Completed: 2026-07-29
Model: Opus 4.6

## 概要

RustFS で DeleteBucketEncryption 実行後に GetBucketEncryption を呼ぶと 400 (Bad Request) が返る。
当初は S3 の仕様では 404 を返すべきと誤認していた。

## 参照

- GetBucketEncryption: <https://docs.aws.amazon.com/AmazonS3/latest/API/API_GetBucketEncryption.html>
- DeleteBucketEncryption: <https://docs.aws.amazon.com/AmazonS3/latest/API/API_DeleteBucketEncryption.html>
- Error Responses: <https://docs.aws.amazon.com/AmazonS3/latest/API/ErrorResponses.html>

`API_GetBucketEncryption` / `API_DeleteBucketEncryption` 個別ページには本エラーの Errors 節は無い。
HTTP ステータスの根拠は Error Responses の List of error codes である。

> **ServerSideEncryptionConfigurationNotFoundError**
> Description: The server-side encryption configuration was not found.
> HTTP status code: 400 Bad Request

## 再現手順

1. バケットを作成する
2. PutBucketEncryption で SSE-S3 (AES256) を設定する
3. DeleteBucketEncryption で暗号化設定を削除する
4. GetBucketEncryption を呼ぶ → **400 が返る**

## 解決方法

誤認だったため closed にする。

Error Responses では `ServerSideEncryptionConfigurationNotFoundError` の HTTP status code は **400 Bad Request** と明記されている。
RustFS / kikyo-local の 400 は仕様どおり。MinIO が 404 を返す方が仕様から外れている。

`tests/rustfs.rs` の `test_bucket_encryption` は期待値を 400 に固定した。
