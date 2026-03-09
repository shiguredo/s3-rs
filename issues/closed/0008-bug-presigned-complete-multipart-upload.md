# Presigned CompleteMultipartUpload が API として完結していない

## 優先度

P2

## 概要

`CompleteMultipartUploadFluentBuilder::presigned()` は URL のみを返すが、
CompleteMultipartUpload は POST ボディに completed parts の XML が必須である。
また `multipart_upload` フィールドを完全に無視しており、Builder で設定したパート情報が捨てられる。

## 影響

- Presigned CompleteMultipartUpload が実行不可能

## 該当箇所

- `src/api/complete_multipart_upload.rs` - `presigned` メソッド

## 修正方針

`presigned` の戻り値を `PresignedRequest` 構造体にし、URL と body を両方返す。
他の presigned メソッドも同じ型を返すように統一する。

## 完了

- `PresignedRequest` 構造体を `api/mod.rs` に導入 (url, method, body を保持)
- 全 8 API の `presigned` メソッドの戻り値を `Result<PresignedRequest, Error>` に統一
- CompleteMultipartUpload の `presigned` で `multipart_upload` から XML body を生成して返すように修正

## 参考

- https://docs.aws.amazon.com/AmazonS3/latest/API/API_CompleteMultipartUpload.html
