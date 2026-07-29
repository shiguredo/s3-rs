# kikyo-local が DeleteBucketEncryption 後の GetBucketEncryption で 400 を返す

- Priority: Low
- Created: 2026-07-29
- Model: Cursor Grok 4.5
- Branch: feature/fix-kikyo-delete-bucket-encryption-get-returns-400

## 概要

kikyo-local で DeleteBucketEncryption 実行後に GetBucketEncryption を呼ぶと 400 (Bad Request) が返る。
当初は S3 の仕様では 404 を返すべきと誤認して起票した。

## 参照

- GetBucketEncryption: <https://docs.aws.amazon.com/AmazonS3/latest/API/API_GetBucketEncryption.html>
- DeleteBucketEncryption: <https://docs.aws.amazon.com/AmazonS3/latest/API/API_DeleteBucketEncryption.html>
- Error Responses: <https://docs.aws.amazon.com/AmazonS3/latest/API/ErrorResponses.html>

`API_GetBucketEncryption` / `API_DeleteBucketEncryption` 個別ページには本エラーの Errors 節は無い。
HTTP ステータスの根拠は Error Responses の List of error codes である。

> **ServerSideEncryptionConfigurationNotFoundError**
> Description: The server-side encryption configuration was not found.
> HTTP status code: 400 Bad Request
