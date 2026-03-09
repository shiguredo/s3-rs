# CompleteMultipartUpload と CopyObject が 200 OK のエラーレスポンスを無視する

## 優先度

P1

## 概要

S3 は CompleteMultipartUpload と CopyObject で HTTP 200 OK を返しつつ、ボディに `<Error>` を含めることがある。
現実装は `is_success()` (2xx) だけで成功判定しているため、このケースでエラーを検出できない。

## 影響

- マルチパートアップロードの完了失敗を黙殺する
- オブジェクトコピーの失敗を黙殺する
- 呼び出し側にはフィールドが `None` の成功値が返り、データロスに気づけない

## 該当箇所

- `src/api/complete_multipart_upload.rs:78` - `parse_response`
- `src/api/copy_object.rs:143` - `parse_response`

## 修正方針

`parse_response` で 2xx の場合もボディに `<Error>` タグが含まれるかチェックし、含まれていたら `Err` を返す。

## 参考

- https://docs.aws.amazon.com/AmazonS3/latest/API/API_CompleteMultipartUpload.html
- https://docs.aws.amazon.com/AmazonS3/latest/API/API_CopyObject.html

## 完了

- `api/mod.rs` に `check_body_error()` ヘルパーを追加。2xx レスポンスのボディに `<Code>` タグが含まれる場合にエラーを返す
- `CompleteMultipartUploadFluentBuilder::parse_response` で `check_body_error()` を呼び出すよう修正
- `CopyObjectFluentBuilder::parse_response` で同様に修正
